#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use eframe::egui;
mod config;
mod logger;
mod pubsub;
mod store;

use egui::epaint::RectShape;
use egui::Color32;
use egui::Layout;
use egui::RichText;
use egui_gauge::Gauge;
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

use config::*;
use logger::*;
use pubsub::mqtt_bridge::mqtt;
use pubsub::redis_bridge::redis;
use pubsub::*;
use store::sub_table::*;
use store::sub_timeseries::*;

pub struct DashboardApp {
    config: Box<Tag>,
}

impl DashboardApp {
    fn new(config: Box<Tag>) -> Self {
        Self { config: config }
    }
}

fn rect_border(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0),
        egui::Pos2::new(rect.max.x - 1.0, rect.max.y - 1.0),
    )
}

fn show_config(ui: &mut egui::Ui, rect: egui::Rect, cfg: &mut Tag) {
    let mut rect = rect;
    if cfg.name == "Dashboard" {
        rect.min.y = 0.0;
        rect.min.x = 0.0;
    }
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
        "Dashboard" => {
            cfg.children.iter_mut().for_each(|child| {
                show_config(ui, rect, child);
                if child.width.is_some() {
                    rect.min.x += child.width.unwrap() as f32;
                }
                if child.height.is_some() {
                    rect.min.y += child.height.unwrap() as f32;
                }
            });
        }
        "Row" => {
            cfg.children.iter_mut().for_each(|child| {
                show_config(ui, rect, child);
                if child.width.is_some() {
                    rect.min.x += child.width.unwrap() as f32;
                }
            });
        }
        "Col" => {
            cfg.children.iter_mut().for_each(|child| {
                show_config(ui, rect, child);
                if child.height.is_some() {
                    rect.min.y += child.height.unwrap() as f32;
                }
            });
        }
        "Status" => {
            let mut style = egui::Style::default();
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 0, 255);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0, 255, 0);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(255, 0, 0);
            style.visuals.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 255));
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0, 255, 0);
            ui.set_style(style);

            let label_text = cfg.label.as_ref().unwrap();
            ui.painter()
                .rect_filled(rect_border(rect), 0.0, Color32::from_rgb(0, 255, 0));
            ui.put(rect_border(rect), egui::Label::new(label_text.as_str()));
            ui.reset_style();
            // ui.put(rect, egui::Button::new("Example button Row"));
        }
        "Gauge" => {
            let mut value = 70.736 as f64;
            let mut range = cfg.min.unwrap() as f64..=cfg.max.unwrap() as f64;
            let square = rect.width().min(rect.height());
            let g = egui_gauge::Gauge::new(value, range, square, Color32::RED)
                .text(cfg.label.as_ref().unwrap());
            ui.put(rect, g);
        }
        "Label" => {
            ui.label(cfg.label.as_ref().unwrap());
        }
        "Progress" => {
            let mut value = 3.5f32;
            let s = format!("{}{}",value, cfg.unit.as_ref().unwrap());
            let rect = rect_border(rect);
            info!("Progress: {:?},{:?}, {}", rect,cfg, s);
            if cfg.min.is_some() && cfg.max.is_some() {
                ui.put(
                    rect_border(rect),
                    egui::ProgressBar::new(value)
                        .desired_height(rect.height())
                        .desired_width(rect.width())
                        .text(s),
                );
            } else {
                ui.add(egui::ProgressBar::new(value).text(cfg.label.as_ref().unwrap()));
            }
        }
        "ValueOutput" => {
            let mut value = 3.5f32;
            let s = format!("{} : {} {}",cfg.label.as_ref().unwrap(),value, cfg.unit.as_ref().unwrap());
            let rect = rect_border(rect);
            info!("Progress: {:?},{:?}, {}", rect,cfg, s);
                ui.put(
                    rect,
                    egui::Label::new(s),
                );
        }
        "Button" => {


            ui.put(rect_border(rect), egui::Button::new(cfg.label.as_ref().unwrap()).fill(Color32::from_rgb(0,0,255)));
            // if ui.button(cfg.label.as_ref().unwrap()).clicked() {};
        }
        "Slider" => {
            let mut value = 0.0f32;
            if cfg.min.is_some() && cfg.max.is_some() {
                let mut style = egui::Style::default();
                style.spacing.slider_width = rect.width();
                ui.set_style(style);
                ui.add(
                    egui::Slider::new(
                        &mut value,
                        cfg.min.unwrap() as f32..=cfg.max.unwrap() as f32,
                    )
                    .text(cfg.label.as_ref().unwrap()),
                );
            } else {
                ui.add(egui::Slider::new(&mut value, 0.0..=1.0).text(cfg.label.as_ref().unwrap()));
            }
        }

        _ => {}
    }
}

impl eframe::App for DashboardApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());


        egui::CentralPanel::default().show(ctx, |ui| {
            let mut style = egui::Style::default();
            style.visuals.override_text_color = Some(Color32::from_rgb(255, 255,255));
            ui.set_style(style);
            let rect = egui::Rect::EVERYTHING;
            show_config(ui, rect, self.config.as_mut());
            /*           ui.heading("Dashboard");
            let rect = egui::Rect::from_min_max(
                egui::Pos2::new(0.0, 0.0),
                egui::Pos2::new(
                    if self.config.width.is_some() {
                        self.config.width.unwrap() as f32
                    } else {
                        100.0
                    },
                    if self.config.height.is_some() {
                        self.config.height.unwrap() as f32
                    } else {
                        100.0
                    }
                ),
            );
            info!("Main window: {:?}", rect);
            self.config.children.iter_mut().for_each(|child| {
                info!("Config: {:?}", child.name);
                show_config(ui, rect, child);
            });*/
        });
        //    ctx.request_repaint();
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
        default_value = "/Users/mg61dd/Developer/egui-dashboard/config.xml"
    )]
    config: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    let args = Args::parse();
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    info!("Starting up. Reading config file {}.", &args.config);

    let config = Box::new(load_xml_file(&args.config).unwrap());
    info!("Config: {:?}", config);
    let mut app = DashboardApp::new(config);

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app)));
}
