
use egui::epaint::RectShape;
use egui::Color32;
use egui::Id;
use egui::Rect;
use egui::Rounding;
use egui::Widget;
use log::info;
use rand::random;
use serde_yaml::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::time::Duration;

use crate::config::file_xml::WidgetParams;
use crate::limero::{SinkRef, SinkTrait};
use crate::pubsub::{PayloadCodec, PubSubCmd, PubSubEvent};
use crate::WidgetMsg;
use crate::WidgetResult;
use tokio::sync::mpsc;



use super::PubSubWidget;

#[derive(Clone)]
pub struct BrokerAlive {
    rect:Rect,
    label: String,
    src_topic: String,
    sinkref_cmd : SinkRef<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
}

impl BrokerAlive {
    pub fn new(rect: Rect, config: &WidgetParams,cmd_sink_ref:SinkRef<PubSubCmd>) -> Self {
        // get random topic
        let topic = format!("dst/broker/alive/{}", random::<u32>());
        Self {
            rect,
            label: config.get_or("label",&config.name).clone(),
            src_topic: topic,
            sinkref_cmd : cmd_sink_ref,
            expire_time: Instant::now() + Duration::from_millis(config.get_or_default("timeout",3000)),
            expire_duration: Duration::from_millis(config.get_or_default("timeout",3000)),
        }
    }
    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}

impl PubSubWidget for BrokerAlive {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let id = Id::new(self.label.clone());
        if !self.expired() {
            ui.painter().add(RectShape::filled(
                self.rect,
                Rounding::ZERO,
                Color32::from_rgb(0, 255, 0),
            ));
        } else {
            ui.painter().add(RectShape::filled(
                self.rect,
                Rounding::ZERO,
                Color32::from_rgb(255, 0, 0),
            ));
        }
        ui.put(
            self.rect,
            egui::Label::new(format!("{}", self.label.clone())),
        );
    }

    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        let old_alive_state = ! self.expired();
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if *topic == self.src_topic {
                    self.expire_time = Instant::now() + self.expire_duration;
                }
            }
            WidgetMsg::Tick => {
                self.sinkref_cmd.push(PubSubCmd::Publish {
                    topic: self.src_topic.clone(),
                    payload: payload_encode("OK"),
                });

            }
        }
        if old_alive_state != self.expired() {
            WidgetResult::Update
        } else {
            WidgetResult::NoEffect
        }
    }
}
