use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::payload_as_f64;
use crate::payload_decode;
use crate::payload_display;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use epaint::RectShape;
use evalexpr::ContextWithMutableVariables;
use log::info;
use std::time::Duration;
use std::time::Instant;

use super::get_eval_or;
use super::Eval;

pub struct Label {
    rect: Rect,
    label: String,
    text: String,
    text_size: i32,
    src_topic: String,
    expire_time: Instant,
    expire_duration: Duration,
    eval: Eval,
}

impl Label {
    pub fn new(rect: Rect, cfg: &WidgetParams) -> Self {
        Self {
            rect,
            label: cfg.get_or("label", &cfg.name),
            text: String::new(),
            text_size: cfg.get_or_default("text_size", 16),
            src_topic: cfg.get_or("src_topic","undefined").clone(),
            expire_time: Instant::now() + Duration::from_millis(cfg.get_or_default("timeout",3000)),
            expire_duration:Duration::from_millis(cfg.get_or_default("timeout",3000)),
            eval:get_eval_or(cfg,"eval","msg_str"),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}

impl PubSubWidget for Label {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        let previous_text = self.text.clone();
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.src_topic == *topic {
                    self.text = match self
                        .eval
                        .eval_to_string(payload) {
                            Ok(value) => value,
                            Err(e) => {
                                info!("Error evaluating expression: {:?}", e);
                                payload_display(payload)
                            }
                        };
                    self.expire_time = Instant::now() + self.expire_duration;
                    WidgetResult::Update
                } else {
                    WidgetResult::NoEffect
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
        let id = Id::new(self.label.clone());
        draw_border(self.rect, ui);

        if self.expired() {
            ui.painter().add(RectShape::filled(
                self.rect,
                Rounding::ZERO,
                Color32::LIGHT_GRAY,
            ));
        } else {
            ui.painter()
                .add(RectShape::filled(self.rect, Rounding::ZERO, Color32::WHITE));
        }

        ui.put(
            self.rect,
            egui::Label::new(format!("{} {}", self.label.clone(), self.text.clone())),
        );
    }
}
