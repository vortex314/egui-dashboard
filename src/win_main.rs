// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use eframe::egui;
use log;
mod logger;
use logger::*;
use std::env;

mod pubsub;
use pubsub::payload_decode;

mod win_status;
use win_status::*;

fn main() -> eframe::Result<()> {
    env::set_var("RUST_LOG", "info");
    let _ = logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Custom Keypad App",
        options,
        Box::new(|cc| {
            // Use the dark theme
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let status_window = WinStatus::new();
            let mut app = Box::<MyApp>::default();
            app.windows.push(Box::new(status_window));

            app
        }),
    )
}

trait PubSubWindow {
    fn show(&mut self, ctx: &egui::Context);
    fn on_message(&mut self, topic: &str, payload: &Vec<u8>);
}

struct MyApp {
    name: String,
    age: u32,
    windows: Vec<Box<dyn PubSubWindow>>,
}

impl MyApp {}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            windows: vec![],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for window in self.windows.iter_mut() {
            window.on_message("Hi", &Vec::<u8>::new());
            window.show(ctx);
        }
    /*     egui::Window::new("Custom Keypad")
            .default_pos([100.0, 100.0])
            .title_bar(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Your name: ");
                    ui.text_edit_singleline(&mut self.name);
                });
                ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if ui.button("Increment").clicked() {
                    self.age += 1;
                }
                ui.label(format!("Hello '{}', age {}", self.name, self.age));
                
            });*/

    }

}
