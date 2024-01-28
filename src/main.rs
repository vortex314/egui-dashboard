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
use widget::slider::Slider;
use widget::progress::Progress;
use widget::status::Status;
use widget::tag::load_xml_file;
use widget::tag::Tag;
use widget::Widget;

pub struct DashboardApp {
    widgets: Vec<Box<dyn Widget>>,
}

impl DashboardApp {
    fn new(widgets: Vec<Box<dyn Widget>>) -> Self {
        Self { widgets }
    }
}

fn show_config(ui: &mut egui::Ui, widgets: &mut Vec<Box<dyn Widget>>) {
    info!("Drawing widgets [{}]", widgets.len());
    let mut rect = egui::Rect::EVERYTHING;
    widgets.iter_mut().for_each(|widget| {
        let _r = widget.draw(ui);
    });
}

fn load_dashboard(cfg: & Tag) -> Vec<Box<dyn Widget>> {
    if cfg.name != "Dashboard" {
        warn!("Invalid config file. Missing Dashboard tag.");
        return Vec::new();
    }
    let mut widgets = Vec::new();
    let mut rect = Rect::EVERYTHING;
    rect.min.y = 0.0;
    rect.min.x = 0.0;
    cfg.children.iter().for_each(|child| {
        info!("Loading widget {}", child.name   );
        let mut sub_widgets = load_config(rect, child);
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

fn load_config(rect: egui::Rect, cfg: & Tag) -> Vec<Box<dyn Widget>> {
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
                let mut sub_widgets = load_config(rect, child);
                widgets.append(&mut sub_widgets);
                if child.width.is_some() {
                    rect.min.x += child.width.unwrap() as f32;
                }
            });
        }
        "Col" => {
            cfg.children.iter().for_each(|child| {
                let mut sub_widgets = load_config(rect, child);
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
            let widget = Button::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Slider" => {
            let widget = Slider::new(rect, cfg);
            widgets.push(Box::new(widget));
        }

        _ => {
            warn!("Unknown widget: {}", cfg.name);
        }
    }
    widgets
}

impl eframe::App for DashboardApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());

        egui::CentralPanel::default().show(ctx, |ui| {
            /*let mut style = egui::Style::default();
            style.visuals.override_text_color = Some(Color32::from_rgb(255, 255, 255));
            ui.set_style(style);*/
            let rect = egui::Rect::EVERYTHING;
            show_config(ui, &mut self.widgets);
        });
        for widget in self.widgets.iter_mut() {
            widget.on_tick();
        }
        // ctx.request_repaint();
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

    #[clap(short, long, default_value = "./config.xml")]
    config: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    let args = Args::parse();
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    info!("Starting up. Reading config file {}.", &args.config);

    let mut config = Box::new(load_xml_file(&args.config).unwrap());
    info!("Config: {:?}", config);
    let widgets = load_dashboard(&mut config);
    let mut app = DashboardApp::new(widgets);

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app)));
}
