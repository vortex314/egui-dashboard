use crate::widget::rect_border;
use crate::widget::tag::Tag;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use egui::containers::Frame;
use egui::*;
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Progress {
    rect: Rect,
    label: String,
    src_topic: String,
    expire_time: Instant,
    expire_duration: Duration,
    min: f64,
    max: f64,
    value: f64,
    unit: String,
}

impl Widget for Progress {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        info!(
            "Progress {} : {} = {}",
            topic,
            payload,
            payload.parse().unwrap_or(0.0)
        );
        self.value = payload.parse().unwrap_or(self.min);
        WidgetResult::Update
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        let s = format!("{} {}", self.value, self.unit);
        let rect = rect_border(self.rect);
        ui.put(
            rect_border(rect),
            egui::ProgressBar::new(self.fraction(self.value))
                .fill(Color32::BLUE)
                .rounding(Rounding::ZERO)
                .desired_height(rect.height())
                .desired_width(rect.width())
                .text(s),
        );

        Ok(())
    }
}

impl Progress {
    pub fn new(rect: Rect, config: &Tag) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        let min = config.min.unwrap_or(0.0);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic: config.src.as_ref().unwrap_or(&String::from("")).clone(),
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min,
            max: config.max.unwrap_or(1.0),
            value: min,
            unit: config.unit.as_ref().unwrap_or(&String::from("")).clone(),
        }
    }
    pub fn fraction(&self, value: f64) -> f32 {
        let mut value = if value < self.min { self.min } else { value };
        value = if value > self.max { self.max } else { value };
        ((value - self.min) / (self.max - self.min)) as f32
    }
}
