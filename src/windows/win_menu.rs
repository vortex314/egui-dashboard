use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use msg::payload_display;
use crate::PubSubWindow;
use crate::{ MyAppCmd, WinProgress};
use egui::*;
use egui_modal::Modal;
use log::info;

use crate::WinStatus;

pub struct WinMenu {
    rect: Rect,
    pub title: String,
    pub regexp: String,
    pub status: u32,
    pub window_types: Vec<String>,
    windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>,
}

impl WinMenu {
    pub fn new(windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>) -> Self {
        Self {
            rect: Rect::from_min_size([100.0, 100.0].into(), [200.0, 200.0].into()),
            title: "Topics".to_owned(),
            regexp: ".*".to_owned(),
            status: 0,
            window_types: vec!["status".to_owned(), "progress".to_owned()],
            windows,
        }
    }

    fn create_new_window(&self, name: &str) -> Box<dyn PubSubWindow + Send> {
        if name == "status" {
            return Box::new(WinStatus::new());
        }
        if name == "progress" {
            return Box::new(WinProgress::new());
        }

        Box::new(WinStatus::new())
    }
}

impl PubSubWindow for WinMenu {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut cmd = None;
        let window_id = Id::new("win_menu");
        let popup_id = Id::new("win_menu_popup");
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE)
            .stroke(Stroke {
                width: 4.0,
                color: Color32::LIGHT_BLUE,
            })
            .inner_margin(5.0);
        let mut win = egui::Window::new(self.title.clone())
            .id(window_id)
            .default_pos(self.rect.min)
            .frame(frame)
            .title_bar(true)
            .resizable(true)
            .collapsible(false)
            .constrain(false)
            //           .max_width(300.0)
            //           .max_height(600.0)
            .anchor(Align2::RIGHT_TOP, Vec2::new(0.0, 0.0));
        win.show(ctx, |ui| {
            for window_type in self.window_types.iter() {
                ui.horizontal(|ui| {
                    let label1 = Button::new(window_type).sense(Sense::click());
                    if ui.add(label1).clicked() {
                        info!("Clicked on {}", window_type);
                        cmd = Some(MyAppCmd::AddWindow(self.create_new_window(&window_type)));
                    };
                });
            }
            //    ui.allocate_space(ui.available_size());
        });
        self.rect = ctx.memory(|mem| mem.area_rect(window_id).map(|rect| rect).unwrap());
        cmd
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {}
}
