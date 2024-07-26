use crate::file_xml::WidgetParams;
use crate::payload_decode;
use crate::payload_display;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use epaint::RectShape;
use log::info;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
pub struct Label {
    rect: Rect,
    label: String,
    text: String,
    text_size: i32,
    src_topic: String,
    expire_time: Instant,
    expire_duration: Duration,
}

impl PubSubWidget for Label {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.src_topic != *topic {
                    WidgetResult::NoEffect
                } else {
                    self.text = payload_display(payload);
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
        info!("Drawing label rect: {:?}", self.rect);
        let id = Id::new(self.label.clone());
        ui.painter().add(RectShape::stroke(
            self.rect,
            Rounding::ZERO,
            Stroke::new(1.0, Color32::BLACK),
        ));
        ui.put(
            self.rect,
            egui::Label::new(format!("{} {}", self.label.clone(), self.text.clone())),
        );
    }
}

impl Label {
    pub fn new(rect: Rect, config: &WidgetParams) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            text: String::new(),
            text_size: config.text_size.unwrap_or(20),
            src_topic: config
                .src_topic
                .as_ref()
                .unwrap_or(&String::from(""))
                .clone(),
            expire_time: Instant::now() + expire_duration,
            expire_duration,
        }
    }
}
