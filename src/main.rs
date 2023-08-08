#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
mod measurements;

use crate::measurements::MeasurementWindow;
use eframe::egui;
mod pubsub;
mod logger;
mod config;

use std::io::BufRead;
use std::sync::*;
use std::thread;
use std::env;
use log::{error, info, warn};

use tokio::task::block_in_place;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::sync::mpsc::{Sender,Receiver,channel};
use tokio::task;
use tokio_stream::StreamExt;


use pubsub::*;
use logger::*;
use config::*;
use pubsub::mqtt_bridge::mqtt;
use pubsub::redis_bridge::redis;

pub struct MonitorApp {
    include_y: Vec<f64>,
    measurements: Arc<Mutex<MeasurementWindow>>,
}

impl MonitorApp {
    fn new(look_behind: usize) -> Self {
        Self {
            measurements: Arc::new(Mutex::new(MeasurementWindow::new_with_look_behind(
                look_behind,
            ))),
            include_y: Vec::new(),
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
        egui::CentralPanel::default().show(ctx, |ui| {
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

    #[clap(short, long,default_value = "/Users/mg61dd/Developer/egui-dashboard/config.yaml")]
    config: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    let args = Args::parse();
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    info!("Starting up. Reading config file {}.", &args.config );

    let config = Box::new(load_yaml_file(&args.config));

    let (mut tx_publish, mut rx_publish) = broadcast::channel::<PubSubEvent>(16);
    let (mut tx_redis_cmd, mut rx_redis_cmd) = channel::<PubSubCmd>(16);

    let redis_config = config["redis"].clone();
    let mqtt_config = config["mqtt"].clone();
    let bc = tx_publish.clone();

    tokio::spawn(async move {
        redis(redis_config, tx_publish).await;
    });
    tokio::spawn(async move {
        mqtt(mqtt_config, bc).await;
    });

    let mut app = MonitorApp::new(args.window_size);
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app)));
}