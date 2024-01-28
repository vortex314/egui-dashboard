
use egui::containers::Frame;
use egui::*;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use crate::widget::tag::Tag;
use crate::widget::rect_border;
use std::time::Duration;
use std::time::Instant;

pub struct Button {
    rect : Rect,
    label: String,
    src_topic: String,
    expire_time: Instant   ,
    expire_duration: Duration,
}

impl Widget for Button {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        WidgetResult::Update
    }
    fn draw(&self, ui: &mut Ui) -> Result<(), String> {
        ui.put(rect_border(self.rect), egui::Button::new(&self.label));
        Ok(())
    }
}

impl Button {
    pub fn new(rect:Rect,config: &Tag) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label:config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic:config.src.as_ref().unwrap_or(&String::from("")).clone(),
            expire_time : Instant::now()+expire_duration,
            expire_duration,
        }
    }    
}
