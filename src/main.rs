#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
mod measurements;

use crate::measurements::MeasurementWindow;
use eframe::egui;
mod config;
mod logger;
mod pubsub;
mod store;

use log::{error, info, warn};
use std::env;
use std::io::BufRead;
use std::sync::*;
use std::thread;

use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tokio::task::block_in_place;
use tokio::time::{self, Duration};
use tokio_stream::StreamExt;

use config::*;
use logger::*;
use pubsub::mqtt_bridge::mqtt;
use pubsub::redis_bridge::redis;
use pubsub::*;
use store::sub_table::*;

pub struct MonitorApp {
    include_y: Vec<f64>,
    measurements: Arc<Mutex<MeasurementWindow>>,
    tx_publish: broadcast::Sender<PubSubEvent>,
    rx_publish: broadcast::Receiver<PubSubEvent>,
    tx_redis_cmd: Sender<PubSubCmd>,
    rx_redis_cmd: Receiver<PubSubCmd>,
    entry_list: EntryList,
}

impl MonitorApp {
    fn new(look_behind: usize) -> Self {
        let (mut tx_publish, mut rx_publish) = broadcast::channel::<PubSubEvent>(16);
        let (mut tx_redis_cmd, mut rx_redis_cmd) = channel::<PubSubCmd>(16);
        Self {
            measurements: Arc::new(Mutex::new(MeasurementWindow::new_with_look_behind(
                look_behind,
            ))),
            include_y: Vec::new(),
            tx_publish,
            rx_publish,
            tx_redis_cmd,
            rx_redis_cmd,
            entry_list: EntryList::new(),
        }
    }
}

impl eframe::App for MonitorApp {
    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(msg) = self.rx_publish.try_recv().ok() {
            match msg {
                PubSubEvent::Publish { topic, message } => {
                    info!("Added {} {}", topic, message);
                    self.entry_list.add(topic, message);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monitor");
            if let Some(e) = self.entry_list.get("src/USB0/system/alive"){
                ui.label(format!("test: {:?}", e));
            }
            let mut plot = egui::plot::Plot::new("measurements");
            for y in self.include_y.iter() {
                plot = plot.include_y(*y);
            }

            plot.show(ui, |plot_ui| {
                plot_ui.line(egui::plot::Line::new(
                    self.measurements.lock().unwrap().plot_values(),
                ));
            });
        });
        // make it always repaint. TODO: can we slow down here?
        ctx.request_repaint();
    }
}

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Look-behind window size
    #[clap(short, long, default_value_t = 1000)]
    window_size: usize,

    #[clap(
        short,
        long,
        default_value = "/home/lieven/workspace/egui-dashboard/config.yaml"
    )]
    config: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    let args = Args::parse();
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    info!("Starting up. Reading config file {}.", &args.config);

    match load_yaml_file(&args.config) {
        Err(e) => error!("Error loading config file {} : {:?}", &args.config, e),
        Ok(val) => {
            let config = Box::new(val);
            let mut app = MonitorApp::new(args.window_size);

            //        let (mut tx_publish, mut rx_publish) = broadcast::channel::<PubSubEvent>(16);
            //        let (mut tx_redis_cmd, mut rx_redis_cmd) = channel::<PubSubCmd>(16);

            let redis_config = config["redis"].clone();
            let mqtt_config = config["mqtt"].clone();
            let tx_publish_clone1 = app.tx_publish.clone();
            let tx_publish_clone2 = app.tx_publish.clone();

            tokio::spawn(async move {
                let _ = redis(redis_config, tx_publish_clone1).await;
            });
            tokio::spawn(async move {
                mqtt(mqtt_config, tx_publish_clone2).await;
            });

            //           let mut app = MonitorApp::new(args.window_size);
            let native_options = eframe::NativeOptions::default();
            let _ = eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app)));
        }
    }
}
