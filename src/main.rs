#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use eframe::egui;
mod logger;
mod pubsub;
mod store;
mod widget;

use egui::epaint::RectShape;
use egui::Color32;
use egui::Layout;
use egui::Rect;
use egui::RichText;
use eframe::egui::Ui;
use log::{error, info, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::io::BufRead;
use std::sync::*;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use widget::WidgetResult;

use tokio::runtime::Builder;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tokio::task::block_in_place;
use tokio::time::{self, Duration};
use tokio_stream::StreamExt;

use logger::*;
//use pubsub::mqtt_bridge::mqtt;
//use pubsub::redis_bridge::redis;
use pubsub::*;

use widget::button::Button;
use widget::gauge::Gauge;
use widget::label::Label;
use widget::plot::Plot;
use widget::progress::Progress;
use widget::slider::Slider;
use widget::status::Status;
use widget::table::Table;
use widget::tag::load_xml_file;
use widget::tag::Tag;
use widget::Widget;

use clap::Parser;
mod zenoh_pubsub;
use zenoh_pubsub::*;
mod limero;
use limero::*;
use limero::ActorTrait;
use limero::SinkRef;
use limero::SinkTrait;

struct MessageHandler {
    dashboard: Arc<Mutex<Dashboard>>,
    cmds : Sink<PubSubEvent>,
}

impl MessageHandler {
    fn new(dashboard: Arc<Mutex<Dashboard>>) -> Self {
        Self {
            dashboard,
            cmds : Sink::new(100)
        }
    }
}


impl ActorTrait<PubSubEvent, ()> for MessageHandler {
    async fn run(&mut self) {
        loop {
            let x = self.cmds.next().await;
            match x {
                Some(cmd) => {
                    match cmd {
                        PubSubEvent::Publish { topic, message } => {
                            self.dashboard.lock().unwrap().on_message(PubSubEvent::Publish { topic, message });
                        }
                        _ => {}
                    }
                }
                None => {
                    warn!("Error in recv : None ");
                }
            }
        }
    }
    fn sink_ref(&self) -> SinkRef<PubSubEvent> {
        self.cmds.sink_ref()
    }
}


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Look-behind window size
    #[clap(short, long, default_value_t = 1000)]
    window_size: usize,

    #[clap(short, long, default_value = "./config.xml")]
    config: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> () {
    let args = Args::parse();
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    info!("Starting up. Reading config file {}.", &args.config);

    let mut pubsub = PubSubActor::new();
    pubsub.sink_ref().push(PubSubCmd::Subscribe { topic : "**".to_string()} );

    let mut dashboard_config = Box::new(load_xml_file(&args.config).unwrap());
    let dashboard_title = dashboard_config.label.clone().unwrap_or(String::from("Dashboard"));
    let dashboard = Arc::new(Mutex::new(Dashboard::new()));
    let _r = dashboard.lock().unwrap().load(&mut dashboard_config, pubsub.sink_ref()).unwrap();
    let mut db_clone = dashboard.clone();

    let mut dashboard_message_handler = MessageHandler::new(dashboard.clone());



    pubsub.add_listener(dashboard_message_handler.sink_ref());

    tokio::spawn(async move {
        pubsub.run().await;
    }); 
    tokio::spawn(async move {
        dashboard_message_handler.run().await;
    });


    let mut app = DashboardApp::new(dashboard);
    let native_options: eframe::NativeOptions = eframe::NativeOptions::default();
    let _r = eframe::run_native(dashboard_title.as_str(), native_options, Box::new(|_| Box::new(app)));
    info!("Exiting.");
}

pub struct Dashboard {
    widgets: Vec<Box<dyn Widget + Send>>,
}
pub struct DashboardApp {
    dashboard : Arc<Mutex<Dashboard>>
}

impl DashboardApp {
    fn new(dashboard: Arc<Mutex<Dashboard>>) -> Self {
        Self {
            dashboard
        }
    }
}

impl eframe::App for DashboardApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        ctx.set_visuals(egui::Visuals::light());

        egui::CentralPanel::default().show(ctx, |ui| {
            self.dashboard.lock().unwrap().draw(ui);
        });

        ctx.request_repaint_after(Duration::from_millis(10)); // update timed out widgets
    }
}

impl Dashboard {
    fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.widgets.iter_mut().for_each(|widget| {
            let _r = widget.draw(ui);
        });
    }

    fn on_message(&mut self, message: PubSubEvent) -> bool {
        let mut repaint = false;
        match message {
            PubSubEvent::Publish { topic, message } => {
                for widget in self.widgets.iter_mut() {
                    if widget.on_message(topic.as_str(), &message) == WidgetResult::Update {
                        repaint = true
                    };
                }
            }
            _ => {}
        }
        repaint
    }

    fn load(&mut self, cfg: &Tag, cmd_sender: SinkRef<PubSubCmd>) -> Result<(), String> {
        if cfg.name != "Dashboard" {
            return Err("Invalid config file. Missing Dashboard tag.".to_string());
        }
        let mut rect = Rect::EVERYTHING;
        rect.min.y = 0.0;
        rect.min.x = 0.0;
        rect.max.x = cfg.width.unwrap_or(1025) as f32;
        rect.max.y = cfg.height.unwrap_or(769) as f32;
        cfg.children.iter().for_each(|child| {
            info!("Loading widget {}", child.name);
            let mut sub_widgets = load_widgets(rect, child, cmd_sender.clone());
            self.widgets.append(&mut sub_widgets);
            if child.width.is_some() {
                rect.min.x += child.width.unwrap() as f32;
            }
            if child.height.is_some() {
                rect.min.y += child.height.unwrap() as f32;
            }
        });
        Ok(())
    }
}

fn load_widgets(
    rect: egui::Rect,
    cfg: &Tag,
    cmd_sender: SinkRef<PubSubCmd>,
) -> Vec<Box<dyn Widget + Send >> {
    let mut widgets: Vec<Box<dyn Widget+Send>> = Vec::new();
    let mut rect = rect;

    if cfg.height.is_some() {
        rect.max.y = rect.min.y + cfg.height.unwrap() as f32;
    }
    if cfg.width.is_some() {
        rect.max.x = rect.min.x + cfg.width.unwrap() as f32;
    }
    info!(
        "{} : {} {:?}",
        cfg.name,
        cfg.label.as_ref().get_or_insert(&String::from("NO_LABEL")),
        rect
    );

    match cfg.name.as_str() {
        "Row" => {
            cfg.children.iter().for_each(|child| {
                let mut sub_widgets = load_widgets(rect, child, cmd_sender.clone());
                widgets.append(&mut sub_widgets);
                if child.width.is_some() {
                    rect.min.x += child.width.unwrap() as f32;
                }
            });
        }
        "Col" => {
            cfg.children.iter().for_each(|child| {
                let mut sub_widgets = load_widgets(rect, child, cmd_sender.clone());
                widgets.append(&mut sub_widgets);
                if child.height.is_some() {
                    rect.min.y += child.height.unwrap() as f32;
                }
            });
        }
        "Status" => {
            let mut status = Status::new(rect, cfg);
            widgets.push(Box::new(status));
        }
        "Gauge" => {
            let widget = Gauge::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Label" => {
            let widget = Label::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Progress" => {
            let widget = Progress::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Button" => {
            let widget = Button::new(rect, cfg, cmd_sender);
            widgets.push(Box::new(widget));
        }
        "Slider" => {
            let widget = Slider::new(rect, cfg, cmd_sender);
            widgets.push(Box::new(widget));
        }
        "Table" => {
            let widget = Table::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Plot" => {
            let widget = Plot::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        _ => {
            warn!("Unknown widget: {}", cfg.name);
        }
    }
    widgets
}
