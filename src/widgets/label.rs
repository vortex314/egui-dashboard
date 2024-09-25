use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
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
    codec: dyn PayloadCodec,
}

impl Label {
    pub fn new(rect: Rect, cfg: &WidgetParams) -> Self {
        Self {
            rect,
            label: cfg.get_or("label", &cfg.name),
            text: String::new(),
            text_size: cfg.get_or_default("text_size", 16),
            src_topic: cfg.get_or("src", "undefined").clone(),
            expire_time: Instant::now()
                + Duration::from_millis(cfg.get_or_default("timeout", u64::MAX / 2)),
            expire_duration: Duration::from_millis(cfg.get_or_default("timeout", u64::MAX / 2)),
            eval: get_eval_or(cfg, "eval", "msg_str"),
            codec: PayloadCodec::from(cfg.get_or("codec", "json")),
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
                    self.text = match self.eval.eval_to_string(payload) {
                        Ok(value) => value,
                        Err(e) => {
                            // info!("Error evaluating expression: {}:{} =>  {:?} for widget Label ",&topic, payload_display(payload),e);
                            self.codec.to_string(payload)
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
        let rect = inside_rect(self.rect, 3.0);
        draw_border(rect, ui);

        if self.expired() {
            ui.painter().add(RectShape::filled(
                rect,
                Rounding::ZERO,
                Color32::LIGHT_GRAY,
            ));
        } else {
            ui.painter()
                .add(RectShape::filled(rect, Rounding::ZERO, Color32::WHITE));
        }

        ui.put(
            rect,
            egui::Label::new(format!("{} {}", self.label.clone(), self.text.clone())),
        );
    }
}
