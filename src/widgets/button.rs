use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::limero::SinkRef;
use crate::limero::SinkTrait;
use crate::payload_decode;
use crate::payload_display;
use crate::payload_encode;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::PubSubCmd;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use epaint::RectShape;
use evalexpr::Value;
use log::info;
use std::time::Duration;
use std::time::Instant;
use std::u64;

use super::get_eval_or;
use super::get_value_or;
use super::get_values_or;
use super::Eval;
use super::EvalError;
use super::Payload;

struct OnOff {
    on: Vec<u8>,
    off: Vec<u8>,
}

pub struct Button {
    rect: Rect,
    margin: f32,
    label: String,
    text: String,
    text_size: i32,
    dst_topic: String,
    dst_val: Payload,
    enabled: bool,
    sinkref_cmd: SinkRef<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
}

impl Button {
    pub fn new(rect: Rect, cfg: &WidgetParams, sinkref_cmd: SinkRef<PubSubCmd>) -> Self {
        Self {
            rect,
            margin: cfg.margin.unwrap_or(5) as f32,
            label: cfg.get_or("label", &cfg.name).clone(),
            text_size: cfg.get_or_default("text_size", 16),
            dst_topic: cfg.get_or("dst", "undefined").clone(),
            dst_val: get_value_or(cfg, "dst_val", "true"),
            text: String::new(),
            enabled: true,
            sinkref_cmd,
            expire_time: Instant::now()
                + Duration::from_millis(cfg.get_or_default("timeout", u64::MAX / 2)),
            expire_duration: Duration::from_millis(cfg.get_or_default("timeout", u64::MAX / 2)),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}

impl PubSubWidget for Button {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        WidgetResult::NoEffect
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let id = Id::new(self.label.clone());
        draw_border(self.rect, ui);
        let rect = inside_rect(self.rect, self.margin);

        if ui
            .put(
                rect,
                egui::Button::new(self.label.clone())
                    .rounding(5.0)
                    .fill(Color32::LIGHT_GREEN)
                    .stroke(Stroke {
                        width: 1.0,
                        color: Color32::BLACK,
                    }),
            )
            .clicked()
        {
            self.sinkref_cmd.push(PubSubCmd::Publish {
                topic: self.dst_topic.clone(),
                payload: self.dst_val.clone(),
            });
        }
    }
}
