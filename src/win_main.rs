// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use clap::ColorChoice;
use egui::*;
use egui_extras::install_image_loaders;
use log::{self, info};
use pubsub::mock_pubsub::MockPubSubActor;
use std::sync::Mutex;
use std::{env, sync::Arc};

use limero::ActorExt;
use limero::*;

mod widgets;
use widgets::*;

mod windows;
use windows::*;

mod config;
use config::*;

mod store;
use store::*;

mod pubsub;
use msg::{payload_display, PubSubCmd, PubSubEvent};
use pubsub::mqtt_pubsub::MqttPubSubActor;
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
    let windows = Arc::new(Mutex::new(Vec::<Box<dyn PubSubWindow + Send>>::new()));
//   let mut pubsub = MqttPubSubActor::new("mqtt://test.mosquitto.org", "homeassistant/#");
    let mut pubsub = MockPubSubActor::new();
    pubsub.handler().handle(&PubSubCmd::Subscribe {
        topic: "**".to_string(),
    });
    let windows_clone = windows.clone();
    pubsub.for_each_event(Box::new(move |event: &PubSubEvent| {
        match event {
            PubSubEvent::Publish { topic, payload } => {
                //  info!("Publish {} {}", topic, payload_display(&payload));
                windows_clone
                    .lock()
                    .map(|mut windows| {
                        for window in windows.iter_mut() {
                            window.on_message(&topic, &payload);
                        }
                    })
                    .unwrap();
            }
            _ => {}
        }
    }));

    tokio::spawn(async move {
        pubsub.run().await;
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MyApp",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, windows)))),
    )
}

pub trait PubSubWindow {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd>;
    fn on_message(&mut self, topic: &str, payload: &Vec<u8>);
}

struct MyApp {
    windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>,
}

pub enum MyAppCmd {
    AddWindow(Box<dyn PubSubWindow + Send>),
}

impl MyApp {
    fn new(
        _cc: &eframe::CreationContext,
        windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>,
    ) -> Self {
        let mut db = Self::default();
        db = Self { windows, ..db };
        //    db.windows.lock().unwrap().push(Box::new(WinStatus::new()));
        db.windows
            .lock()
            .unwrap()
            .push(Box::new(WinMenu::new(db.windows.clone())));
        db.windows
            .lock()
            .unwrap()
            .push(Box::new(WinTopics::new(db.windows.clone())));
        db
    }
    fn windows(&self) -> Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>> {
        self.windows.clone()
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            windows: Arc::new(Mutex::new(Vec::<Box<dyn PubSubWindow + Send>>::new())),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //     info!("update windows count {}", self.windows.lock().unwrap().len());
        install_image_loaders(ctx);
        let l = self.windows.lock();
        if let Ok(mut windows) = l {
            let mut cmds = Vec::new();
            for window in windows.iter_mut() {
                let cmd = window.show(ctx);
                if let Some(cmd) = cmd {
                    cmds.push(cmd);
                }
            }
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
