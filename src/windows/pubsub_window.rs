use std::sync::Arc;

use crate::payload_decode;
use crate::pubsub::payload_as_f64;
use crate::MyAppCmd;
use crate::PubSubWindow;
use egui::*;
use log::info;
use minicbor::data::Int;
use rand::Rng;

pub struct WinLabel {
    rect: Rect,
    widget: PubSubWidget,
}

impl WinLabel {
    pub fn new(rect:Rect,widget:PubSubWidget) -> Self {
        Self {
            rect,
            widget,
        }
    }
    fn context_menu(&mut self, ui: &mut Ui) {
        let topics = vec![
            "src/esp32/sys/uptime",
            "src/esp32/sys/latency",
            "src/esp32/sys/heap_free",
            "src/esp32/sys/heap_used",
        ];
        ui.label("Context menu");
        ui.horizontal(|ui| {
            ui.label("Title: ");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.horizontal(|ui| {
            ui.label("Suffix: ");
            ui.text_edit_singleline(&mut self.suffix);
        });
        let mut topic_selected = self.src_topic.clone();
        ui.horizontal(|ui| {
            ui.label("Source topic: ");
            egui::ComboBox::from_id_source("Source topic")
                .selected_text(format!("{:?}", topic_selected))
                .show_ui(ui, |ui| {
                    for topic in topics.iter() {
                        ui.selectable_value(
                            &mut topic_selected,
                            topic.to_string(),
                            topic.to_string(),
                        );
                    }
                })
        });
        if topic_selected != self.src_topic {
            self.title = topic_selected.clone();
            self.src_topic = topic_selected;
            self.current_value = None;
            self.min_value = None;
            self.max_value = None;
        }
        ui.allocate_space(ui.available_size());
    }
}

fn get_opt(v: &Option<f64>) -> String {
    match v {
        Some(value) => format!("{}", value),
        None => "--".to_owned(),
    }
}

fn round_rect_to_multiple(rect: Rect, multiple: f32) -> Rect {
    let min = rect.min;
    let max = rect.max;
    let min_x = (min.x / multiple).round() * multiple;
    let min_y = (min.y / multiple).round() * multiple;
    let max_x = (max.x / multiple).round() * multiple;
    let max_y = (max.y / multiple).round() * multiple;
    Rect::from_min_max([min_x, min_y].into(), [max_x, max_y].into())
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
