use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::pubsub::payload_decode;
use crate::pubsub::payload_display;
use crate::pubsub::payload_encode;
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
pub struct Space {
    rect: Rect,
}

impl PubSubWidget for Space {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        WidgetResult::NoEffect
    }

    fn draw(&mut self, ui: &mut egui::Ui) {}
}

impl Space {
    pub fn new(rect: Rect, config: &WidgetParams) -> Self {
        Self { rect }
    }
}
