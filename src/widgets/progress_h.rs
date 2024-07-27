use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::payload_as_f64;
use crate::payload_decode;
use crate::payload_display;
use crate::store::timeseries;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use egui_plot::PlotPoints;
use epaint::ColorMode;
use epaint::PathStroke;
use epaint::RectShape;
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct ProgressH {
    rect: Rect,
    label: String,
    suffix: String,
    src_topic: String,
    value: f64,
    expire_time: Instant,
    expire_duration: Duration,
    min: f64,
    max: f64,
}

impl PubSubWidget for ProgressH {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.src_topic != *topic {
                    WidgetResult::NoEffect
                } else {
                    self.value = payload_as_f64(payload).unwrap_or(payload_decode::<u64>(payload).unwrap_or(self.min as u64 ) as f64);
                    self.expire_time = Instant::now() + self.expire_duration;
                        WidgetResult::Update
                    
                }
            }
            WidgetMsg::Tick => {
                if Instant::now() > self.expire_time {
                    return WidgetResult::Update;
                }
                WidgetResult::NoEffect
            }
        }

    }



    fn draw(&mut self, ui: &mut egui::Ui) {
        let s = format!("{} {}", self.value, self.suffix);
        let rect = inside_rect(self.rect,2.0);
        ui.put(
            rect,
            egui::ProgressBar::new(self.fraction(self.value))
                .fill(Color32::RED)
                .rounding(Rounding::ZERO)
                .desired_height(rect.height())
                .desired_width(rect.width())
                .text(s),
        );
    }
}

impl ProgressH {
    pub fn new(rect: Rect, config: &WidgetParams) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);

        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            suffix: config.suffix.as_ref().unwrap_or(&String::from("")).clone(),
            src_topic: config.src_topic.as_ref().unwrap_or(&String::from("")).clone(),
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min: config.min.unwrap_or(0.0),
            max: config.max.unwrap_or(1.0),
            value: config.min.unwrap_or(0.0),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }

    pub fn fraction(&self, value: f64) -> f32 {
        let mut value = if value < self.min { self.min } else { value };
        value = if value > self.max { self.max } else { value };
        ((value - self.min) / (self.max - self.min)) as f32
    }
}

