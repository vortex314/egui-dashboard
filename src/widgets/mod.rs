use egui::{epaint::RectShape, Color32, Rounding, Stroke, Ui};
/*pub mod status;
pub mod gauge;*/
pub mod label;
pub use label::Label as Label;
pub mod broker_alive;
pub use broker_alive::BrokerAlive as BrokerAlive;
pub mod button;
pub use button::Button as Button;
pub mod space;
pub use space::Space as Space;
mod plot;
pub use plot::Plot as Plot;
mod table;
pub use table::Table as Table;
mod gauge;
pub use gauge::Gauge as Gauge;
mod progress_h;
pub use progress_h::ProgressH as ProgressH;
mod slider;
pub use slider::Slider as Slider;
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

pub fn inside_rect(rect: egui::Rect,margin : f32) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + margin, rect.min.y + margin),
        egui::Pos2::new(rect.max.x - margin, rect.max.y - margin),
    )
}

pub fn draw_border(rect: egui::Rect,ui : &egui::Ui )  {
    ui.painter().add(RectShape::stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(1.0, Color32::LIGHT_GRAY),
    ));
}

