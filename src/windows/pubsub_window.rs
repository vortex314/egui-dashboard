use std::sync::Arc;

use crate::payload_decode;
use crate::pubsub::payload_as_f64;
use crate::MyAppCmd;
use egui::*;
use log::info;
use minicbor::data::Int;
use rand::Rng;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetMsg;

pub struct PubSubWindow {
    rect: Rect,
    widget: Box<dyn PubSubWidget>,
}

impl PubSubWindow {
    pub fn new(rect:Rect,widget:Box<dyn PubSubWidget>) -> Self {
        Self {
            rect,
            widget,
        }
    }
    fn context_menu(&mut self, ui: &mut Ui) {
        ui.allocate_space(ui.available_size());
    }
}


impl PubSubWindow  {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE);
        let id = Id::new(rand::thread_rng().gen::<u64>().to_string());
        let mut win = egui::Window::new("No Title")
            .id(id)
            .default_pos(self.rect.min)
            .current_pos(self.rect.min)
            .frame(frame)
            .title_bar(false)
            .resizable(true)
            .collapsible(false)
            .constrain(false);
        win.show(ctx, |ui| {
            self.widget.draw(ui);
        });
        None
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        self.widget.update(&WidgetMsg::Pub {
            topic: topic.to_string(),
            payload: payload.clone(),
        });
    }
}
