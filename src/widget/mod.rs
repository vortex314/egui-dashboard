use egui::Ui;
pub mod status;
pub mod gauge;
pub mod tag;
pub mod label;
pub mod progress;
pub mod button;
pub mod slider;

use tag::Tag;

use crate::PubSubCmd;

pub enum WidgetResult {
    Update,
    NoEffect,
}

pub trait Widget {
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String>;
    fn on_message(&mut self, topic:&str,payload:&str) -> WidgetResult;
    fn on_tick(&mut self) -> WidgetResult {
        WidgetResult::NoEffect
    }
}

pub fn rect_border(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0),
        egui::Pos2::new(rect.max.x - 1.0, rect.max.y - 1.0),
    )
}

