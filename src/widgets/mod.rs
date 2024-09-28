use std::collections::VecDeque;
use std::time::Duration;

use egui::{epaint::RectShape, Color32, Rounding, Stroke, Ui};
/*pub mod status;
pub mod gauge;*/
pub mod label;
use evalexpr::ValueType;
use evalexpr::{ContextWithMutableFunctions, ContextWithMutableVariables, Value};
pub use label::Label;
pub mod broker_alive;
pub use broker_alive::BrokerAlive;
pub mod button;
pub use button::Button;
pub mod space;
use log::error;
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
mod dial;
pub use dial::Dial;
/*pub mod progress;
pub mod button;
pub mod slider;
pub mod table;
pub mod plot;*/

use msg::{payload_as_f64, payload_decode, payload_encode, PubSubCmd};

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
        let m_bool = payload_decode::<bool>(payload);
        self.context.clear_variables();
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
        if let Ok(v) = m_bool {
            self.context
                .set_value("msg_bool".into(), evalexpr::Value::Boolean(v))
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

fn value_to_payload_1(value: &Value) -> Vec<u8> {
    match value {
        Value::Int(x) => payload_encode(x),
        Value::Float(x) => payload_encode(x),
        Value::Boolean(x) => payload_encode(x),
        Value::String(x) => payload_encode(x),
        Value::Tuple(x) => {
            let mut v = Vec::new();
            for i in x {
                v.extend(value_to_payload_1(i));
            }
            v
        }
        _ => vec![],
    }
}

use crate::config::WidgetParams;
pub type Payload = Vec<u8>;

pub fn get_eval_or(cfg: &WidgetParams, key: &str, default: &str) -> Eval {
    let default_str = default.to_string();
    let eval = cfg.get(key).unwrap_or(&default_str);
    let r = Eval::create(eval.clone());
    match r {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to create eval for {} : {:?}", key, e);
            Eval::create(default.to_string()).unwrap()
        }
    }
}

pub fn get_values_or(cfg: &WidgetParams, key: &str, default: &str) -> Vec<Payload> {
    let default_str = default.to_string();
    let values_str = cfg.get(key).unwrap_or(&default_str);
    let values_default = evalexpr::eval(default).unwrap();
    match evalexpr::eval(values_str.as_str()) {
        Ok(value) => match value_to_payload_array(&value) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed value_to_payload_array {}.{} : {:?} default {} value : {}", &cfg.name,key, e, default,value);
                value_to_payload_array(&values_default).unwrap()
            }
        },
        Err(e) => {
            error!("Failed to create values for {} : {:?} default {}", key, e, default);
            let default_value = evalexpr::eval(default).unwrap();
            value_to_payload_array(&default_value).unwrap()
        }
    }
}

pub fn get_value_or(cfg: &WidgetParams, key: &str, default: &str) -> Payload {
    let default_str = default.to_string();
    let value_str = cfg.get(key).unwrap_or(&default_str);
    let value_default = evalexpr::eval(default).unwrap();

    match evalexpr::eval(value_str.as_str()) {
        Ok(value) => value_to_payload(&value).unwrap(),
        Err(e) => {
            error!("Failed to create value for {} : {}", key, e);
            let default_value = evalexpr::eval(default).unwrap();
            value_to_payload(&default_value).unwrap()
        }
    }
}

pub fn value_to_payload_array(value: &Value) -> Result<Vec<Payload>, EvalError> {
    match value {
        Value::Tuple(a) => {
            let mut v: Vec<Payload> = Vec::new();
            for value in a {
                match value_to_payload(&value) {
                    Ok(p) => v.push(p),
                    Err(e) => {
                        error!("Failed to create values for {:?} : {} type {:?} ", v, value, ValueType::from(value));
                        return Err(e)
                    }
                }
            }
            Ok(v)
        }
        Value::String(s) => Ok(vec![payload_encode(s)]),
        Value::Int(i) => Ok(vec![payload_encode(i)]),
        Value::Float(f) => Ok(vec![payload_encode(f)]),
        Value::Boolean(b) => Ok(vec![payload_encode(b)]),
        Value::Empty => Ok(Vec::new()),
    }
}

pub fn value_to_payload(value: &Value) -> Result<Payload, EvalError> {
    match value {
        Value::String(s) => Ok(payload_encode(s)),
        Value::Int(i) => Ok(payload_encode(i)),
        Value::Float(f) => Ok(payload_encode(f)),
        Value::Boolean(b) => Ok(payload_encode(b)),
        Value::Tuple(a) => Err(EvalError::ParseError),
        Value::Empty => Ok(Vec::new()),
    }
}

pub fn expr_to_payload(default_value: &str) -> Result<Payload, EvalError> {
    let values = evalexpr::eval(default_value).map_err(|_| EvalError::ParseError)?;
    value_to_payload(&values)
}

pub fn expr_to_payload_with_default(
    val_str: &Option<String>,
    default_value: &str,
) -> Result<Payload, EvalError> {
    let default_payloads = expr_to_payload(default_value).unwrap();
    match val_str {
        Some(val) => expr_to_payload(&val),
        None => Ok(default_payloads),
    }
}

fn major_ticks(min: f64, max: f64) -> VecDeque<f64> {
    let log_max = max.log10().ceil();
    println!(" min {} max {} log_max : {} ", min, max, log_max);
    let start = 10.0_f64.powf(log_max);
    let mut step = 10.0_f64.powf(log_max - 1.0);
    let mut v = VecDeque::new();
    loop {
        let mut cursor = start;
        v.clear();
        while cursor >= min {
            if cursor <= max {
                v.push_front(cursor);
            }
            cursor -= step;
        }
        if v.len() > 3 { break; }
        step = step / 10.0;
    }
    v
}
