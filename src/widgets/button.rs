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
    src_topic: String,
    dst_topic: String,
    src_val: Eval,
    dst_val: Vec<Payload>,
    on_state: bool,
    enabled: bool,
    sinkref_cmd: SinkRef<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
    eval: super::Eval,
}

impl Button {
    pub fn new(rect: Rect, cfg: &WidgetParams, sinkref_cmd: SinkRef<PubSubCmd>) -> Self {
        Self {
            rect,
            margin: cfg.margin.unwrap_or(5) as f32,
            label: cfg.get_or("label", &cfg.name).clone(),
            text_size: cfg.get_or_default("text_size", 16),
            src_topic: cfg.get_or("src_topic", "undefined").clone(),
            dst_topic: cfg.get_or("dst_topic", "undefined").clone(),
            text: String::new(),
            src_val: get_eval_or(&cfg, "src_eval", "msg_bool"),
            dst_val: get_values_or(&cfg, "dst_val", "true,false"),
            on_state: if cfg.get_or("dst_topic", "").len() == 0 {
                true
            } else {
                false
            },
            enabled: true,
            sinkref_cmd,
            expire_time: Instant::now()
                + Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            expire_duration: Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            eval: get_eval_or(cfg, "eval", "msg_str"),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}

impl PubSubWidget for Button {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if *topic == self.src_topic {
                    let _ = self.eval.eval_bool(payload).map(|value| {
                        self.on_state = value;
                    });

                    self.expire_time = Instant::now() + self.expire_duration;
                    self.enabled = true;
                    WidgetResult::Update
                } else {
                    WidgetResult::NoEffect
                }
            }
            WidgetMsg::Tick => {
                if self.expired() && self.src_topic.len() > 0 {
                    self.enabled = false;
                    WidgetResult::Update
                } else {
                    WidgetResult::NoEffect
                }
            }
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let id = Id::new(self.label.clone());
        draw_border(self.rect, ui);
        let rect = inside_rect(self.rect, self.margin);
        let mut enabled = true;
        if self.src_topic.len() > 0 {
            enabled = self.enabled;
        }

        ui.add_enabled_ui(enabled, |ui| {
            if ui
                .put(
                    rect,
                    egui::Button::new(self.label.clone())
                        .rounding(5.0)
                        .fill(if self.on_state {
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::LIGHT_RED
                        })
                        .stroke(Stroke {
                            width: 1.0,
                            color: Color32::BLACK,
                        }),
                )
                .clicked()
            {
                let _r = self.sinkref_cmd.push(PubSubCmd::Publish {
                    topic: self.dst_topic.clone(),
                    payload: self.dst_val[0].clone(),
                });
            }
        });
    }
}
