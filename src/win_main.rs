// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use clap::ColorChoice;
use eframe::egui;
use egui::Color32;
use log::{self, info};
mod logger;
use logger::*;
use std::{env, sync::Arc};
use std::sync::Mutex;
mod pubsub;
use pubsub::{payload_decode, payload_display};
mod zenoh_pubsub;
use zenoh_pubsub::PubSubActor;
use crate::pubsub::{PubSubCmd, PubSubEvent};

mod limero;
use limero::*;

mod win_status;
use win_status::*;

mod win_menu;
use win_menu::*;

mod win_topics;
use win_topics::*;


#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> eframe::Result<()> {
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();

    let my_frame = egui::containers::Frame {
        inner_margin: egui::style::Margin { left: 10., right: 10., top: 10., bottom: 10. },
        outer_margin: egui::style::Margin { left: 10., right: 10., top: 10., bottom: 10. },
        rounding: egui::Rounding { nw: 1.0, ne: 1.0, sw: 1.0, se: 1.0 },
        shadow: eframe::epaint::Shadow { extrusion: 1.0, color: Color32::YELLOW },
        fill: Color32::LIGHT_BLUE,
        stroke: egui::Stroke::new(2.0, Color32::GOLD),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    let mut app = Box::<MyApp>::default();
    app.windows.try_lock().ok().unwrap().push(Box::new(WinStatus::new()));
    let mut windows = app.windows().clone();

    app.windows.try_lock().ok().unwrap().push(Box::new(WinMenu::new(windows)));
    let mut windows = app.windows().clone();

    let mut pubsub = PubSubActor::new();
    pubsub.sink_ref().push(PubSubCmd::Subscribe {
        topic: "**".to_string(),
    });
    pubsub.for_all( Box::new(
       move  |event| {
            match event {
                PubSubEvent::Publish { topic, message } => {
                    info!("Publish {} {}", topic, payload_display(&message));
                    windows.lock().map(|mut windows| {
                        for window in windows.iter_mut() {
                            window.on_message(&topic, &message);
                        }
                    }).unwrap();

                },
                _  => {},
            }
        }
    ));

    tokio::spawn(async move {
        pubsub.run().await;
    });

    eframe::run_native(
        "PubSub Dashboard",
        options,
        Box::new(|cc| {
            // Use the dark theme
            let mut visuals = egui::Visuals::light();
            visuals.window_fill = Color32::LIGHT_BLUE;
            visuals.panel_fill = Color32::LIGHT_BLUE;

            cc.egui_ctx.set_visuals(visuals);
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);



            app
        }),
    )
}

trait PubSubWindow {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd>;
    fn on_message(&mut self, topic: &str, payload: &Vec<u8>);
}

struct MyApp {
    windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send >>>>,
}

pub enum MyAppCmd {
    AddWindow(Box<dyn PubSubWindow + Send>),
}

impl MyApp {
    fn windows(&self) -> Arc<Mutex<Vec<Box<dyn PubSubWindow + Send >>>> {
        self.windows.clone()
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            windows: Arc::new(Mutex::new(Vec::<Box<dyn PubSubWindow + Send >>::new())),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       let l = self.windows.lock();
        if let Ok(mut windows) = l {
            let mut cmds = Vec::new();
            for window in windows.iter_mut() {
                let cmd =  window.show(ctx);
                if let Some(cmd) = cmd {
                    cmds.push(cmd);
                }
            };
            for cmd in cmds {
                match cmd {
                    MyAppCmd::AddWindow(window) => {
                        windows.push(window);
                    }
                }
            }
        }
    }
}
