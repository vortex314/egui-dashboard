use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::pubsub::payload_decode;
use crate::pubsub::payload_display;
use crate::pubsub::payload_encode;
use crate::pubsub::payload_as_f64;
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
        let s = format!("{:.3} {}", self.value, self.suffix);
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
    pub fn new(rect: Rect, cfg: &WidgetParams) -> Self {

        Self {
            rect,
            label: cfg.get_or("label", &cfg.name).clone(),
            suffix: cfg.get_or("label","").clone(),
            src_topic: cfg.get_or("src", "undefined").clone(),
            expire_time: Instant::now()
                + Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            expire_duration: Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            min: cfg.get_or_default("min", 0.0),
            max: cfg.get_or_default("max", 1.0),
           value: 0.0,
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

