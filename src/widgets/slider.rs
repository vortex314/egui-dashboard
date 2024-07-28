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
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Slider {
    rect: Rect,
    label: String,
    src_topic: String,
    dst_topic: String,
    sinkref_cmd: SinkRef<PubSubCmd>,
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
            .drag_stopped()
        {
            let _r = self.sinkref_cmd.push(PubSubCmd::Publish {
                topic: self.dst_topic.clone(),
                payload: payload_encode(self.value),
            });
        }
    }
}

impl Slider {
    pub fn new(rect: Rect, config: &WidgetParams, sinkref_cmd: SinkRef<PubSubCmd>) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
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
            sinkref_cmd: sinkref_cmd,
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min: config.min.unwrap_or(0.0),
            max: config.max.unwrap_or(100.0),
            unit: config.unit.as_ref().unwrap_or(&String::from("")).clone(),
            value: 0.0,
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}
