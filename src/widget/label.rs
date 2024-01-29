use crate::widget::tag::Tag;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use egui::containers::Frame;
use egui::*;
use log::info;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
pub struct Label {
    rect: Rect,
    label: String,
    text_size: i32,
    src_topic: String,
    expire_time: Instant,
    expire_duration: Duration,
}

impl Widget for Label {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        self.label = payload.to_string();
        WidgetResult::Update
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        ui.put(
            self.rect,
            egui::widgets::Label::new(
                egui::RichText::new(self.label.clone())
                    .size(self.text_size as f32)
                    .color(Color32::BLACK),
            ),
        );
        Ok(())
    }
}

impl Label {
    pub fn new(rect: Rect, config: &Tag) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            text_size: config.text_size.unwrap_or(20),
            src_topic: config.src.as_ref().unwrap_or(&String::from("")).clone(),
            expire_time: Instant::now() + expire_duration,
            expire_duration,
        }
    }
}
