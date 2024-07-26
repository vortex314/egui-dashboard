use egui::Ui;
/*pub mod status;
pub mod gauge;*/
pub mod label;
pub use label::Label as Label;
/*pub mod progress;
pub mod button;
pub mod slider;
pub mod table;
pub mod plot;*/


use crate::PubSubCmd;

#[derive(PartialEq)]
pub enum WidgetResult {
    Update,
    NoEffect,
}

pub enum WidgetMsg {
    Pub { topic : String, payload : Vec<u8>},
    Tick ,
}

pub trait PubSubWidget : Send {
    fn update(&mut self, event:& WidgetMsg) -> WidgetResult;
    fn draw(&mut self,ui:&mut Ui);
}

pub fn rect_border(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0),
        egui::Pos2::new(rect.max.x - 1.0, rect.max.y - 1.0),
    )
}

