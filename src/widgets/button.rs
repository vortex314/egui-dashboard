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

use super::value_to_payload;
use super::Eval;

pub struct Button {
    rect: Rect,
    margin: f32,
    label: String,
    text: String,
    text_size: i32,
    src_topic: String,
    dst_topic: String,
    dst_msg: Vec<u8>,
    on_state: bool,
    enabled: bool,
    sinkref_cmd: SinkRef<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
    eval: Option<super::Eval>,
}

impl Button {
    pub fn new(rect: Rect, config: &WidgetParams, sinkref_cmd: SinkRef<PubSubCmd>) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        let eval = match &config.eval {
            None => None,
            Some(evals) => Eval::create(evals.clone()).ok(),
        };
        info!(
            "Value : {:?} = {:?}",
            config.dst_msg,
            value_to_payload(&Value::from(
                config.dst_msg.as_ref().unwrap_or(&String::from("")).clone()
            ))
        );
        info!("Button params : {:?}", config);
        Self {
            rect,
            margin: config.margin.unwrap_or(5) as f32,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            text: String::new(),
            text_size: config.text_size.unwrap_or(20),
            src_topic: config
                .src_topic
                .as_ref()
                .unwrap_or(&String::from(""))
                .clone(),
            dst_topic: config
                .dst_topic
                .as_ref()
                .unwrap_or(&String::from(""))
                .clone(),
            dst_msg: value_to_payload(&Value::from(
                config.dst_msg.as_ref().unwrap_or(&String::from("")).clone(),
            )),
            on_state: if config.src_topic.is_none() {
                true
            } else {
                false
            },
            enabled: true,
            sinkref_cmd,
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            eval,
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
                    self.eval.as_mut().map(|ev| {
                        let r = ev.eval_bool(payload).map(|value| {
                            self.on_state = value;
                        });
                        info!("Eval result : {:?}", r);
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
                    payload: self.dst_msg.clone(),
                });
            }
        });
    }
}
