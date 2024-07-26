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
use std::sync::*;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
/*
struct MessageHandler {
    dashboard: Arc<Mutex<Dashboard>>,
    cmds: Sink<PubSubEvent>,
}

impl MessageHandler {
    fn new(dashboard: Arc<Mutex<Dashboard>>) -> Self {
        Self {
            dashboard,
            cmds: Sink::new(100),
        }
    }
}

impl ActorTrait<PubSubEvent, ()> for MessageHandler {
    async fn run(&mut self) {
        loop {
            let x = self.cmds.next().await;
            match x {
                Some(cmd) => match cmd {
                    PubSubEvent::Publish { topic, payload } => {
                        self.dashboard
                            .lock()
                            .unwrap()
                            .update(WidgetMsg::Pub { topic, payload });
                    }
                    _ => {}
                },
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
*/
fn start_pubsub_mqtt(
    cfg: &Element,
    event_sink: SinkRef<PubSubEvent>,
) -> Result<SinkRef<PubSubCmd>, String> {
    let mut mqtt_actor = MqttPubSubActor::new();
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

    let mut event_sink = limero::Sink::new(100);

    let root_config = load_xml_file("./config.xml").map_err(MyError::Xml)?;

    let pubsub_config = root_config
        .get_child("PubSub", "")
        .ok_or(MyError::Str("PubSub section not found"))?;
    let pubsub_cmd =
        start_pubsub_mqtt(&pubsub_config, event_sink.sink_ref()).map_err(MyError::String)?;

    let dashboard_config = root_config
        .get_child("Dashboard", "")
        .ok_or(MyError::Str("Dashboard section not found"))?;
    let widgets_params = load_dashboard(&dashboard_config).map_err(MyError::String)?;
    let mut widgets = Vec::new();
    for widget_params in widgets_params {
        let widget = create_widget(&widget_params, pubsub_cmd.clone()).map_err(MyError::String)?;
        widgets.push(widget);
    }

    let dashboard = Dashboard {
        widgets:Arc::new(Mutex::new(widgets)),
        pubsub_cmd,
    };


    let db = dashboard.clone();
    tokio::spawn(async move {
        loop {
            let m = event_sink.next().await;
            match m {
                Some(PubSubEvent::Publish { topic, payload }) => {
                    info!("Publishing topic {} payload {:?}", topic, payload);
                    db.update(WidgetMsg::Pub { topic, payload });
                }
                _ => {}
            }
        }
    });

    let native_options: eframe::NativeOptions = eframe::NativeOptions::default();
    let _r = eframe::run_native(
        "Dashboard",
        native_options,
        Box::new(|x| Ok(Box::new(DashboardApp::new(dashboard)))),
    );
    info!("Exiting.");
    Ok(())
}

#[derive(Clone) ]
pub struct Dashboard {
    widgets: Arc<Mutex<Vec<Box<dyn PubSubWidget + Send>>>>,
    pubsub_cmd: SinkRef<PubSubCmd>,
}
#[derive(Clone)]

pub struct DashboardApp {
    dashboard: Dashboard,
}

impl DashboardApp {
    fn new(dashboard: Dashboard) -> Self {
        Self { dashboard }
    }

    fn update(&mut self, widget_msg: WidgetMsg) {
        self.dashboard.update(widget_msg);
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

        ctx.request_repaint_after(Duration::from_millis(10000)); // update timed out widgets
    }
}

impl Dashboard {
    fn new(pubsub_cmd: SinkRef<PubSubCmd>) -> Self {
        Self {
            widgets: Vec::new(),
            pubsub_cmd,
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.widgets.iter_mut().for_each(|widget| {
            widget.draw(ui);
        });
    }

    fn update(&mut self, widget_msg: WidgetMsg) -> bool {
        let mut repaint = false;
        self.widgets.lock().iter_mut().for_each(|widget| {
            if widget.update(&widget_msg) == WidgetResult::Update {
                repaint = true
            };
        });
        if repaint {
            self.context.as_ref().unwrap().request_repaint();
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
        _ => Ok(Box::new(Label::new(rect, cfg))), //Err(format!("Unknown widget: {}", cfg.name)),
    }
}
