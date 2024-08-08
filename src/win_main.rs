// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use clap::ColorChoice;
use egui::*;
use log::{self, info};
use std::{env, sync::Arc};
use std::sync::Mutex;


mod limero;
use limero::*;

mod windows;
use windows::*;

mod pubsub;
use pubsub::{payload_decode, payload_display,payload_as_f64};
use pubsub::{PubSubCmd, PubSubEvent};
use pubsub::zenoh_pubsub::ZenohPubSubActor;
mod logger;



#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> eframe::Result<()> {
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();

    let my_frame = egui::containers::Frame::default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    let mut app = Box::<MyApp>::default();
    app.windows.try_lock().ok().unwrap().push(Box::new(WinStatus::new()));
    let mut windows = app.windows().clone();

    app.windows.try_lock().ok().unwrap().push(Box::new(WinMenu::new(windows)));
    let mut windows = app.windows().clone();

    let mut pubsub = ZenohPubSubActor::new();
    pubsub.sink_ref().push(PubSubCmd::Subscribe {
        topic: "**".to_string(),
    });
    pubsub.for_all( Box::new(
       move  |event| {
            match event {
                PubSubEvent::Publish { topic, payload } => {
                    info!("Publish {} {}", topic, payload_display(&payload));
                    windows.lock().map(|mut windows| {
                        for window in windows.iter_mut() {
                            window.on_message(&topic, &payload);
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

        let native_options = eframe::NativeOptions::default();
        eframe::run_native("MyApp", options, Box::new(|cc| Ok(Box::new(MyApp::new(cc)))))

    /*eframe::run_native(
        "PubSub Dashboard",
        options,
        Box::new(|cc| {
            // Use the dark theme
            let mut visuals = egui::Visuals::light();
            visuals.window_fill = Color32::LIGHT_BLUE;
            visuals.panel_fill = Color32::LIGHT_BLUE;

            cc.egui_ctx.set_visuals(visuals);
            // This gives us image support:
       //     egui_extras::install_image_loaders(&cc.egui_ctx);



            app
        }),
    )*/
}

pub trait PubSubWindow {
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
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self::default()
    }
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
