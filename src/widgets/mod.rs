use std::time::Duration;

use egui::{epaint::RectShape, Color32, Rounding, Stroke, Ui};
/*pub mod status;
pub mod gauge;*/
pub mod label;
use evalexpr::{ContextWithMutableFunctions, ContextWithMutableVariables, Value};
pub use label::Label;
pub mod broker_alive;
pub use broker_alive::BrokerAlive;
pub mod button;
pub use button::Button;
pub mod space;
use log::info;
use minicbor::decode;
pub use space::Space;
mod plot;
pub use plot::Plot;
mod table;
pub use table::Table;
mod gauge;
pub use gauge::Gauge;
mod progress_h;
pub use progress_h::ProgressH;
mod slider;
pub use slider::Slider;
mod progress_v;
pub use progress_v::ProgressV;
mod switch;
pub use switch::Switch;
/*pub mod progress;
pub mod button;
pub mod slider;
pub mod table;
pub mod plot;*/

use crate::{payload_as_f64, payload_decode, payload_display, payload_encode, PubSubCmd};

#[derive(PartialEq)]
pub enum WidgetResult {
    Update,
    NoEffect,
}

pub enum WidgetMsg {
    Pub { topic: String, payload: Vec<u8> },
    Tick,
}

pub trait PubSubWidget: Send {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult;
    fn draw(&mut self, ui: &mut Ui);
}

pub fn inside_rect(rect: egui::Rect, margin: f32) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::Pos2::new(rect.min.x + margin, rect.min.y + margin),
        egui::Pos2::new(rect.max.x - margin, rect.max.y - margin),
    )
}

pub fn draw_border(rect: egui::Rect, ui: &egui::Ui) {
    ui.painter().add(RectShape::stroke(
        rect,
        Rounding::ZERO,
        Stroke::new(1.0, Color32::LIGHT_GRAY),
    ));
}

#[derive(Debug)]
pub enum EvalError {
    DecodeError(decode::Error),
    EvalError(evalexpr::EvalexprError),
    ParseError,
}

pub struct Eval {
    node: evalexpr::Node,
    context: evalexpr::HashMapContext,
}

impl Eval {
    pub fn create(str: String) -> Result<Eval, EvalError> {
        let node = evalexpr::build_operator_tree(&str).map_err(EvalError::EvalError)?;
        let context = evalexpr::HashMapContext::new();
        let mut ev = Eval { node, context };
        ev.load_context();
        Ok(ev)
    }

    pub fn load_context(&mut self) {
        let eval_humantime = evalexpr::Function::new(|arg| {
            Ok(match arg {
                Value::Int(x) => Value::String(
                    humantime::format_duration(Duration::from_millis(*x as u64)).to_string(),
                ),
                Value::Float(x) => Value::String(
                    humantime::format_duration(Duration::from_millis(*x as u64)).to_string(),
                ),
                _ => Value::String("Invalid input to humantime".to_string()),
            })
        });
        self.context
            .set_function("humantime".into(), eval_humantime)
            .unwrap();
    }

    pub fn common_eval(&mut self, payload: &Vec<u8>) -> Result<Value, EvalError> {
        let v64 = payload_as_f64(&payload);
        let vstr = payload_decode::<String>(payload);
        if let Ok(v) = v64 {
            self.context
                .set_value("msg_f64".into(), evalexpr::Value::Float(v))
                .map_err(EvalError::EvalError)?;
        };
        if let Ok(v) = vstr {
            self.context
                .set_value("msg_str".into(), evalexpr::Value::String(v))
                .map_err(EvalError::EvalError)?;
        };

        let value: Value = self
            .node
            .eval_with_context(&self.context)
            .map_err(EvalError::EvalError)?;
        Ok(value)
    }

    pub fn eval_to_string(&mut self, payload: &Vec<u8>) -> Result<String, EvalError> {
        let value = self.common_eval(payload)?;
        Ok(value.as_string().map_err(EvalError::EvalError)?)
    }

    pub fn eval_to_f64(&mut self, payload: &Vec<u8>) -> Result<f64, EvalError> {
        let value = self.common_eval(payload)?;
        Ok(value.as_float().map_err(EvalError::EvalError)?)
    }

    pub fn eval_bool(&mut self, payload: &Vec<u8>) -> Result<bool, EvalError> {
        let value = self.common_eval(payload)?;
        Ok(value.as_boolean().map_err(EvalError::EvalError)?)
    }
}

fn value_to_payload(value: &Value) -> Vec<u8> {
    match value {
        Value::Int(x) => payload_encode(x),
        Value::Float(x) => payload_encode(x),
        Value::Boolean(x) => payload_encode(x),
        Value::String(x) => payload_encode(x),
        _ => vec![],
    }
}
