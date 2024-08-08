use std::sync::Arc;

use crate::payload_decode;
use crate::pubsub::payload_as_f64;
use crate::MyAppCmd;
use crate::PubSubWindow;
use egui::*;
use log::info;
use minicbor::data::Int;
use rand::Rng;

pub struct PubSubWindow {
    rect: Rect,
    widget: PubSubWidget,
}

impl PubSubWindow {
    pub fn new(rect:Rect,widget:PubSubWidget) -> Self {
        Self {
            rect,
            widget,
        }
    }
    fn context_menu(&mut self, ui: &mut Ui) {
        
        ui.allocate_space(ui.available_size());
    }
}


impl PubSubWindow for WinLabel {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE);
        let mut win = egui::Window::new(self.title.clone())
            .id(self.window_id)
            .default_pos(self.rect.min)
            .current_pos(self.rect.min)
            .frame(frame)
            .title_bar(false)
            .resizable(true)
            .collapsible(false)
            .constrain(false);
        win.show(ctx, |ui| {
            self.widget.show(ui);
        });
        None
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        self.widget.on_message(topic, payload);
    }
}
