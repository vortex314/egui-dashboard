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
use pubsub::mqtt_bridge::mqtt;
use pubsub::redis_bridge::redis;
use pubsub::*;

use widget::button::Button;
use widget::gauge::Gauge;
use widget::label::Label;
use widget::progress::Progress;
use widget::slider::Slider;
use widget::status::Status;
use widget::tag::load_xml_file;
use widget::tag::Tag;
use widget::table::Table;
use widget::plot::Plot;
use widget::Widget;

use clap::Parser;

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

//#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
fn main() {
    let args = Args::parse();
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    info!("Starting up. Reading config file {}.", &args.config);
    let (mut publish_sender, mut publish_receiver) = channel::<PubSubEvent>(16);
    let (mut cmd_sender, mut cmd_receiver) = channel::<PubSubCmd>(16);

    let mut config = Box::new(load_xml_file(&args.config).unwrap());
    let widgets = load_dashboard(&mut config, cmd_sender);
    let mut app = DashboardApp::new(widgets, publish_receiver);
    let native_options: eframe::NativeOptions = eframe::NativeOptions::default();

    thread::spawn(move || {
        let result = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                redis(
                    "redis://limero.ddns.net:6379",
                    publish_sender,
                    &mut cmd_receiver,
                )
                .await
            });
    });

    let _ = eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app)));
    info!("Exiting.");
}

pub struct DashboardApp {
    widgets: Vec<Box<dyn Widget>>,
    receiver_events: Receiver<PubSubEvent>,
}

impl DashboardApp {
    fn new(widgets: Vec<Box<dyn Widget>>, receiver_events: Receiver<PubSubEvent>) -> Self {
        Self {
            widgets,
            receiver_events,
        }
    }
}

impl eframe::App for DashboardApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = egui::Rect::EVERYTHING;
            show_dashboard(ui, &mut self.widgets);
        });
        let mut repaint = false;
        for widget in self.widgets.iter_mut() {
            if widget.on_tick() == WidgetResult::Update {
                repaint = true
            };
        }
        if repaint {
            ctx.request_repaint();
            repaint=false;
        };
        let x = self.receiver_events.try_recv();
        match x {
            Ok(m) => match m {
                PubSubEvent::Publish { topic, message } => {
                    for widget in self.widgets.iter_mut() {
                        if widget.on_message(topic.as_str(), message.as_str())
                            == WidgetResult::Update
                        {
                            repaint = true
                        };
                    }
                    if repaint { ctx.request_repaint()};
                }
            },
            Err(e) => {
                // warn!("Error in recv : {}", e);
            }
        }
        ctx.request_repaint_after(Duration::from_millis(1000)); // updqte timed out widgets
    }
}

fn show_dashboard(ui: &mut egui::Ui, widgets: &mut Vec<Box<dyn Widget>>) {
    // info!("Drawing widgets [{}]", widgets.len());
    let mut rect = egui::Rect::EVERYTHING;
    widgets.iter_mut().for_each(|widget| {
        let _r = widget.draw(ui);
    });
}

fn load_dashboard(cfg: &Tag, cmd_sender: Sender<PubSubCmd>) -> Vec<Box<dyn Widget>> {
    if cfg.name != "Dashboard" {
        warn!("Invalid config file. Missing Dashboard tag.");
        return Vec::new();
    }
    let mut widgets = Vec::new();
    let mut rect = Rect::EVERYTHING;
    rect.min.y = 0.0;
    rect.min.x = 0.0;
    rect.max.x = cfg.width.unwrap_or(1025) as f32;
    rect.max.y = cfg.height.unwrap_or(769) as f32;
    cfg.children.iter().for_each(|child| {
        info!("Loading widget {}", child.name);
        let mut sub_widgets = load_widgets(rect, child, cmd_sender.clone());
        widgets.append(&mut sub_widgets);
        if child.width.is_some() {
            rect.min.x += child.width.unwrap() as f32;
        }
        if child.height.is_some() {
            rect.min.y += child.height.unwrap() as f32;
        }
    });
    widgets
}

fn load_widgets(
    rect: egui::Rect,
    cfg: &Tag,
    cmd_sender: Sender<PubSubCmd>,
) -> Vec<Box<dyn Widget>> {
    let mut widgets: Vec<Box<dyn Widget>> = Vec::new();
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
        cfg.label.as_ref().get_or_insert(&String::from("-")),
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
        _ => {
            warn!("Unknown widget: {}", cfg.name);
        }
    }
    widgets
}
