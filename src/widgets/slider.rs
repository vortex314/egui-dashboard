use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use limero::Endpoint;
use msg::payload_encode;
use msg::PubSubCmd;

use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use epaint::RectShape;
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Slider {
    rect: Rect,
    label: String,
    src_topic: String,
    dst_topic: String,
    sinkref_cmd: Endpoint<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
    min: f64,
    max: f64,
    unit: String,
    value: f32,
}

impl PubSubWidget for Slider {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        WidgetResult::NoEffect
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let id = Id::new(self.label.clone());
        draw_border(self.rect, ui);
        let rect = inside_rect(self.rect, 2.0);

        if ui
            .put(
                rect,
                egui::Slider::new(&mut self.value, self.min as f32..=self.max as f32)
                    .text(self.label.clone()),
            )
            .dragged()
        {
            let _r = self.sinkref_cmd.handle(&PubSubCmd::Publish {
                topic: self.dst_topic.clone(),
                payload: msg::cbor::encode(&self.value),
            });
        }
    }
}

impl Slider {
    pub fn new(rect: Rect, cfg: &WidgetParams, sinkref_cmd: Endpoint<PubSubCmd>) -> Self {
        let expire_duration = Duration::from_millis(cfg.get_or_default("timeout",3000) as u64);
        Self {
            rect,
            label: cfg.get_or("label", cfg.name.as_str()).clone(),
            src_topic: cfg.get_or("src","undefined").clone(),
            dst_topic: cfg.get_or("dst", "undefined").clone(),
            sinkref_cmd: sinkref_cmd,
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min: cfg.get_or_default("min",0.0),
            max: cfg.get_or_default("max",1.0),
            unit: cfg.get_or("unit","").clone(),
            value: 0.0,
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}
