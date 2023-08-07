#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
mod measurements;

use crate::measurements::MeasurementWindow;
use eframe::egui;
mod redis_bridge;
mod my_logger;
mod sub_table;

use sub_table::EntryList;
use std::io::BufRead;
use std::sync::*;
use std::thread;
use log::{error, info, warn};

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

    #[clap(short, long)]
    include_y: Vec<f64>,
}

fn main() {
    let args = Args::parse();

    let mut app = MonitorApp::new(args.window_size);

    app.include_y = args.include_y;

    let native_options = eframe::NativeOptions::default();


    info!("Main thread started");
    let _ = eframe::run_native("Monitor app", native_options, Box::new(|_| Box::new(app)));
}