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

pub struct Button {
    rect: Rect,
    margin: f32,
    label: String,
    text: String,
    text_size: i32,
    dst_topic: String,
    sinkref_cmd: SinkRef<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
}

impl PubSubWidget for Button {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        WidgetResult::NoEffect
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let id = Id::new(self.label.clone());
        draw_border(self.rect, ui);
        let rect = inside_rect(self.rect,self.margin);


        if ui
            .put(
                rect,
                egui::Button::new(self.label.clone())
    //                .fill(Color32::BLUE)
                    .rounding(5.0)
                    .stroke(Stroke {
                        width: 1.0,
                        color: Color32::BLACK,
                    }),
            )
            .clicked()
        {
            let _r = self.sinkref_cmd.push(PubSubCmd::Publish {
                topic: self.dst_topic.clone(),
                payload: payload_encode("TEST".to_string()),
            });
        }
    }
}

impl Button {
    pub fn new(rect: Rect, config: &WidgetParams, sinkref_cmd: SinkRef<PubSubCmd>) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            margin: config.margin.unwrap_or(5) as f32,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            text: String::new(),
            text_size: config.text_size.unwrap_or(20),
            dst_topic: config
                .dst_topic
                .as_ref()
                .unwrap_or(&String::from(""))
                .clone(),
            sinkref_cmd,
            expire_time: Instant::now() + expire_duration,
            expire_duration,
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}
