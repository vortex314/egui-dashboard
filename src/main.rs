#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use eframe::egui;
mod logger;
mod pubsub;
mod store;

use eframe::egui::Ui;
use egui::epaint::RectShape;
use egui::Color32;
use egui::Layout;
use egui::Pos2;
use egui::Rect;
use egui::RichText;
use file_change::FileChangeActor;
use file_change::FileChangeEvent;
use file_xml::load_dashboard;
use file_xml::load_xml_file;
use file_xml::WidgetParams;
use log::{error, info, warn};
use minidom::Element;
use mqtt_pubsub::MqttPubSubActor;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::io::BufRead;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use tokio::runtime::Builder;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tokio::task::block_in_place;
use tokio::time::{self, sleep, Duration};
use tokio_stream::StreamExt;

use logger::*;
//use pubsub::mqtt_bridge::mqtt;
//use pubsub::redis_bridge::redis;
use pubsub::*;
mod widgets;
use widgets::*;

use widgets::Label;

use clap::Parser;
use zenoh_pubsub::*;
mod limero;
use limero::ActorTrait;
use limero::SinkRef;
use limero::SinkTrait;
use limero::SourceTrait;
use limero::*;
mod config;
use config::*;

fn start_pubsub_mqtt(
    cfg: &Element,
    event_sink: SinkRef<PubSubEvent>,
) -> Result<SinkRef<PubSubCmd>, String> {
    let url = cfg.attr("url").unwrap_or("mqtt://pcthink.local:1883/");
    let pattern = cfg.attr("pattern").unwrap_or("#");
    let mut mqtt_actor = MqttPubSubActor::new(url, pattern);
    let pubsub_cmd = mqtt_actor.sink_ref();
    mqtt_actor.add_listener(event_sink);
    tokio::spawn(async move {
        mqtt_actor.run().await;
        error!("Mqtt actor exited");
    });
    /*   pubsub_cmd.push(PubSubCmd::Connect);
    pubsub_cmd.push(PubSubCmd::Subscribe {
        topic: "**".to_string(),
    });*/
    Ok(pubsub_cmd)
}

fn start_pubsub_zenoh(
    cfg: &Element,
    event_sink: SinkRef<PubSubEvent>,
) -> Result<SinkRef<PubSubCmd>, String> {
    let zenoh = cfg
        .get_child("Zenoh", "")
        .ok_or("Zenoh section not found")?;
    let mut zenoh_actor = ZenohPubSubActor::new();
    let pubsub_cmd = zenoh_actor.sink_ref();
    zenoh_actor.add_listener(event_sink);
    tokio::spawn(async move {
        zenoh_actor.run().await;
    });
    pubsub_cmd.push(PubSubCmd::Connect);
    pubsub_cmd.push(PubSubCmd::Subscribe {
        topic: "**".to_string(),
    });
    Ok(pubsub_cmd)
}

pub fn start_file_change_actor(db: Arc<Mutex<Dashboard>>) -> Result<(), String> {
    let file_name = "./config.xml".to_string();
    let mut file_change_actor = FileChangeActor::new(file_name);
    // File Change Actor

    let mut file_change_actor = FileChangeActor::new("./config.xml".to_string());
    file_change_actor.for_all(Box::new(move |x: FileChangeEvent| {
        let mut binding = db.lock();
        let res = binding.as_mut().map(|mut db| {
            db.clear();
            let file_name = "./config.xml".to_string();
            let root_config = load_xml_file(&file_name).map_err(MyError::Xml).unwrap();
            let dashboard_config = root_config
                .get_child("Dashboard", "")
                .ok_or(MyError::Str("Dashboard section not found"))
                .unwrap();
            let widgets_params = load_dashboard(&dashboard_config)
                .map_err(MyError::String)
                .unwrap();
            for widget_params in widgets_params {
                let widget = create_widget(&widget_params, db.pubsub_cmd.clone())
                    .map_err(MyError::String)
                    .unwrap();
                db.add_widget(widget);
            }
        });
        if res.is_err() {
            error!("Error reloading config file {:?}", res);
        }
    }));

    file_change_actor.trigger_file_change();


    tokio::spawn(async move {
        file_change_actor.run().await;
    });
    Ok(())
}

#[derive(Debug)]
enum MyError<'a> {
    Io(std::io::Error),
    Xml(minidom::Error),
    Yaml(serde_yaml::Error),
    Str(&'a str),
    String(String),
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
async fn main() -> Result<(), MyError<'static>> {
    let args = Args::parse();
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    info!("Starting up. Reading config file {}.", &args.config);

    let mut event_sink = limero::Sink::new(1000);

    let root_config = load_xml_file("./config.xml").map_err(MyError::Xml)?;
    let pubsub_config = root_config
        .get_child("PubSub", "")
        .ok_or(MyError::Str("PubSub section not found"))?;
    /*  let pubsub_cmd =
    start_pubsub_mqtt(&pubsub_config, event_sink.sink_ref()).map_err(MyError::String)?;*/

    let pubsub_cmd =
        start_pubsub_zenoh(&pubsub_config, event_sink.sink_ref()).map_err(MyError::String)?;

    let dashboard = Arc::new(Mutex::new(Dashboard {
        widgets: Vec::new(),
        pubsub_cmd,
        context: None,
    }));

    start_file_change_actor(dashboard.clone()).map_err(MyError::String)?;

    // update widgets with PubSub event
    let db = dashboard.clone();
    tokio::spawn(async move {
        loop {
            let m = event_sink.next().await;
            match m {
                Some(PubSubEvent::Publish { topic, payload }) => {
                    db.lock().unwrap().update(WidgetMsg::Pub { topic, payload });
                }
                _ => {}
            }
        }
    });
    // 1 second Ticker
    let db = dashboard.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            db.lock().unwrap().update(WidgetMsg::Tick);
        }
    });

    let native_options: eframe::NativeOptions = eframe::NativeOptions::default();
    let _r = eframe::run_native(
        "Dashboard",
        native_options,
        Box::new(|cc| {
            let _ = dashboard
                .try_lock()
                .unwrap()
                .set_context(cc.egui_ctx.clone());
            Ok(Box::new(DashboardApp::new(dashboard)))
        }),
    );
    info!("Exiting.");
    Ok(())
}

pub struct Dashboard {
    widgets: Vec<Box<dyn PubSubWidget + Send>>,
    pubsub_cmd: SinkRef<PubSubCmd>,
    context: Option<egui::Context>,
}
#[derive(Clone)]

pub struct DashboardApp {
    dashboard: Arc<Mutex<Dashboard>>,
}

impl DashboardApp {
    fn new(dashboard: Arc<Mutex<Dashboard>>) -> Self {
        Self { dashboard }
    }
}

impl eframe::App for DashboardApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());

        egui::CentralPanel::default().show(ctx, |ui| {
            self.dashboard.lock().unwrap().draw(ui); // not in an async context
        });
    }
}

impl Dashboard {
    fn new(pubsub_cmd: SinkRef<PubSubCmd>, context: egui::Context) -> Self {
        Self {
            widgets: Vec::new(),
            pubsub_cmd,
            context: Some(context),
        }
    }

    fn clear(&mut self) {
        self.widgets.clear();
    }

    fn add_widget(&mut self, widget: Box<dyn PubSubWidget + Send>) {
        self.widgets.push(widget);
    }

    fn set_context(&mut self, context: egui::Context) {
        self.context = Some(context);
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.widgets.iter_mut().for_each(|widget| {
            widget.draw(ui);
        });
    }

    fn update(&mut self, widget_msg: WidgetMsg) -> bool {
        let mut repaint = false;
        self.widgets.iter_mut().for_each(|widget| {
            if widget.update(&widget_msg) == WidgetResult::Update {
                repaint = true
            };
        });
        if repaint && self.context.is_some() {
            // self.context.as_ref().unwrap().request_repaint();
            self.context
                .as_mut()
                .map(|ctx| ctx.request_repaint_after(Duration::from_millis(50)));
            // update timed out widgets
        }
        repaint
    }

    /*fn load(&mut self, cfg: &Tag) -> Result<(), String> {
        if cfg.name != "Dashboard" {
            return Err("Invalid config file. Missing Dashboard tag.".to_string());
        }
        let mut rect = Rect::EVERYTHING;
        rect.min.y = 0.0;
        rect.min.x = 0.0;
        rect.max.x = cfg.width.unwrap_or(1025) as f32;
        rect.max.y = cfg.height.unwrap_or(769) as f32;
        self.widgets.clear(); // clear existing widgets for reload
        cfg.children.iter().for_each(|child| {
            info!("Loading widget {}", child.name);
            let mut sub_widgets = load_widgets(rect, child, self.pubsub_cmd.clone());
            self.widgets.append(&mut sub_widgets);
            if child.width.is_some() {
                rect.min.x += child.width.unwrap() as f32;
            }
            if child.height.is_some() {
                rect.min.y += child.height.unwrap() as f32;
            }
        });
        Ok(())
    }*/
}

fn create_widget(
    cfg: &WidgetParams,
    cmd_sender: SinkRef<PubSubCmd>,
) -> Result<Box<dyn PubSubWidget + Send>, String> {
    let name = cfg.name.as_str();
    let rect = Rect {
        min: Pos2 {
            x: cfg.rect.x as f32,
            y: cfg.rect.y as f32,
        },
        max: Pos2 {
            x: cfg.rect.x as f32 + cfg.rect.w as f32,
            y: cfg.rect.y as f32 + cfg.rect.h as f32,
        },
    };
    match name {
        "Label" => Ok(Box::new(Label::new(rect, cfg))),
        "Plot" => Ok(Box::new(Plot::new(rect, cfg))),
        "Gauge" => Ok(Box::new(Gauge::new(rect, cfg))),
        "Table" => Ok(Box::new(Table::new(rect, cfg))),
        "ProgressH" => Ok(Box::new(ProgressH::new(rect, cfg))),
        "ProgressV" => Ok(Box::new(ProgressV::new(rect, cfg))),
        "BrokerAlive" => Ok(Box::new(BrokerAlive::new(rect, cfg, cmd_sender))),
        "Button" => Ok(Box::new(Button::new(rect, cfg, cmd_sender))),
        "Slider" => Ok(Box::new(Slider::new(rect, cfg, cmd_sender))),
        "Space" => Ok(Box::new(Space::new(rect, cfg))),
        "Switch" => Ok(Box::new(Switch::new(rect, cfg, cmd_sender))),
        _ => Ok(Box::new(Label::new(rect, cfg))), //Err(format!("Unknown widget: {}", cfg.name)),
    }
}
