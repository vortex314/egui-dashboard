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

pub struct ProgressV {
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

impl PubSubWidget for ProgressV {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.src_topic != *topic {
                    WidgetResult::NoEffect
                } else {
                    self.value =
                        payload_as_f64(payload).unwrap_or(
                            payload_decode::<u64>(payload).unwrap_or(self.min as u64) as f64,
                        );
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
     //   let s = format!("{:.3} {}", self.value, self.suffix);
        let line2_rect = Rect::from_min_max(
            Pos2 {
                x: self.rect.min.x,
                y: self.rect.max.y - 20.0,
            },
            self.rect.max,
        );
        let line1_rect = Rect::from_min_max(
            Pos2 {
                x: self.rect.min.x,
                y: self.rect.max.y - 40.0,
            },
            Pos2 {
                x: self.rect.max.x,
                y: self.rect.max.y - 20.0,
            },
        );
        let bar_rect = Rect::from_min_max(
            Pos2 {
                x: self.rect.min.x+10.0,
                y: self.rect.min.y,
            },
            Pos2 {
                x: self.rect.max.x,
                y: self.rect.max.y - 40.0,
            },
        );
        let (progress_bar_rect, ticks_rect) = bar_rect.split_left_right_at_fraction(0.5);
        let (empty_progress_bar_rect, filled_progress_bar_rect) =
            progress_bar_rect.split_top_bottom_at_fraction(1.0 - self.fraction(self.value));
        let rect = inside_rect(self.rect, 2.0);
        ui.put(line1_rect, egui::Label::new(self.label.clone()));
        ui.put(
            line2_rect,
            egui::Label::new(format!(
                "{:.3} {}",
                self.value,
                self.suffix.clone()
            )),
        );
        ui.painter().with_clip_rect(progress_bar_rect).rect_filled(
            filled_progress_bar_rect,
            Rounding::default(),
            Color32::RED,
        );
        ui.painter().rect_stroke(
            empty_progress_bar_rect,
            Rounding::default(),
            Stroke::new(1.0, Color32::LIGHT_GRAY),
        );
    }
}

impl ProgressV {
    pub fn new(rect: Rect, config: &WidgetParams) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);

        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            suffix: config.suffix.as_ref().unwrap_or(&String::from("")).clone(),
            src_topic: config
                .src_topic
                .as_ref()
                .unwrap_or(&String::from(""))
                .clone(),
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
