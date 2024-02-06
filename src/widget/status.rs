
use egui::containers::Frame;
use egui::*;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use crate::widget::tag::Tag;
use std::time::Duration;
use std::time::Instant;

use crate::widget::rect_border;

#[derive(PartialEq)]
enum StatusValue {
    Ok,
    Failed,
    Timeout,
}

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

    fn on_tick(&mut self) -> WidgetResult {
        if self.expired() && self.value!=StatusValue::Timeout {
            self.value = StatusValue::Timeout;
            return WidgetResult::Update;
        }
        WidgetResult::NoEffect
    }

    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        self.expire_time = Instant::now()+self.expire_duration;
        self.value = StatusValue::Ok;
        WidgetResult::Update
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        if self.expired() && self.value!=StatusValue::Timeout {
            self.value = StatusValue::Timeout;
        }
        let mut color = match self.value {
            StatusValue::Ok => Color32::from_rgb(0,255,0),
            StatusValue::Failed => Color32::from_rgb(255,0,0),
            StatusValue::Timeout => Color32::from_rgb(180,180,180),
        };
        let mut painter = ui.painter();
        let rect = rect_border(self.rect);
        painter.rect_filled(rect, 0.0, color);

        ui.put(rect_border(self.rect), egui::Label::new(&self.label));
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
            value: StatusValue::Ok,
            expire_time : Instant::now()+expire_duration,
            expire_duration,
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
    
}
