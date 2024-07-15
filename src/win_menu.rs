use std::collections::HashMap;
use std::hash::Hash;

use crate::payload_decode;
use crate::pubsub::payload_display;
use crate::PubSubWindow;
use egui::*;
use egui_modal::Modal;
use log::info;

pub struct WinMenu {
    rect: Rect,
    pub title: String,
    pub regexp: String,
    pub status: u32,
    pub topics: HashMap<String, String>,
}

impl WinMenu {
    pub fn new() -> Self {
        Self {
            rect: Rect::from_min_size([200.0, 200.0].into(), [300.0, 300.0].into()),
            title: "Topics".to_owned(),
            regexp: ".*".to_owned(),
            status: 0,
            topics: HashMap::new(),
        }
    }
}

impl Default for WinMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl PubSubWindow for WinMenu {
    fn show(&mut self, ctx: &egui::Context) {
        let window_id = Id::new("win_menu");
        let popup_id = Id::new("win_menu_popup");
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE)
            .inner_margin(5.0);
        let mut win = egui::Window::new(self.title.clone())
            .id(window_id)
            .default_pos(self.rect.min)
            .frame(frame)
            .title_bar(true)
            .resizable(false)
            .collapsible(false)
            .constrain(false)
            //           .max_width(300.0)
            //           .max_height(600.0)
            .anchor(Align2::RIGHT_TOP, Vec2::new(0.0, 0.0));
        win.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter");
                ui.add(egui::TextEdit::singleline(&mut self.regexp));
            });
            for (topic, payload) in self.topics.iter() {
                ui.horizontal(|ui| {
                    let label1 = Button::new(topic).sense(Sense::click());
                    if ui.add(label1).clicked() {
                        info!("Clicked on {}", topic);
                    };
                    let line = format!("{}", payload);
                    let label2 = Label::new(line.as_str());
                    ui.add(label2);
                });
            }
            ui.separator();
            //    ui.allocate_space(ui.available_size());
        });
        self.rect = ctx.memory(|mem| mem.area_rect(window_id).map(|rect| rect).unwrap());
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        let value_string = payload_display(payload);

        self.topics.insert(topic.to_string(), value_string);
    }
}
