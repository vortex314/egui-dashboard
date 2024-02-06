use crate::widget::tag::Tag;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use egui::containers::Frame;
use egui::*;
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Gauge {
    rect: Rect,
    label: String,
    src_topic: String,
    value: f64,
    expire_time: Instant,
    expire_duration: Duration,
    min: f64,
    max: f64,
}

impl Widget for Gauge {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        self.expire_time = Instant::now() + self.expire_duration;
        self.value = payload.parse().unwrap_or(0.0);
        WidgetResult::Update
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        let mut range = self.min..=self.max;
        let square = self.rect.width().min(self.rect.height());
        let g = egui_gauge::Gauge::new(self.value, range, square, Color32::RED)
            .text(self.label.clone());
        ui.put(self.rect, g);
        Ok(())
    }
}

impl Gauge {
    pub fn new(rect: Rect, config: &Tag) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic: config.src.as_ref().unwrap_or(&String::from("")).clone(),
            value: 0.0,
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min: config.min.unwrap_or(0.0),
            max: config.max.unwrap_or(100.0),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}
