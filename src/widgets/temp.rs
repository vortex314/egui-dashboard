
use egui::containers::Frame;
use egui::*;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use crate::widget::tag::Tag;
use std::time::Duration;
use std::time::Instant;




pub struct Status {
    rect : Rect,
    label: String,
    src_topic: String,
    src_value_ok: String,
    src_value_nok: String,
    value: StatusValue,
    expire_time: Instant   ,
    expire_duration: Duration,
}

impl Widget for Status {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        WidgetResult::Update
    }
    fn draw(&self, ui: &mut Ui) -> Result<(), String> {
        
        Ok(())
    }
}

impl Status {
    pub fn new(rect:Rect,config: &Tag) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label:config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic:config.src.as_ref().unwrap_or(&String::from("")).clone(),
            src_value_ok: config.on.as_ref().unwrap_or(&String::from("ok")).clone(),
            src_value_nok: config.off.as_ref().unwrap_or(&String::from("nok")).clone(),
            value: StatusValue::Timeout,
            expire_time : Instant::now()+expire_duration,
            expire_duration,
        }
    }    
}
