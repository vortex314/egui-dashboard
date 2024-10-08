use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::store::timeseries;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use egui_plot::PlotPoints;
use epaint::ColorMode;
use epaint::PathShape;
use epaint::PathStroke;
use epaint::RectShape;
use log::error;
use log::info;
use msg::payload_as_f64;
use msg::payload_decode;
use msg::payload_display;
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

pub struct GaugeH {
    rect: Rect,
    label: String,
    suffix: String,
    src_topic: String,
    value: f32,
    expire_time: Instant,
    expire_duration: Duration,
    min: f32,
    max: f32,
    major_ticks: Vec<f32>,
    minor_ticks: Vec<f32>,
}

impl PubSubWidget for GaugeH {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.src_topic != *topic {
                    WidgetResult::NoEffect
                } else {
                    if let Ok(new_value) = msg::cbor::as_f64(payload) {
                        self.value = new_value as f32;
                        self.expire_time = Instant::now() + self.expire_duration;
                        return WidgetResult::Update;
                    } else {
                        error!(
                            "GaugeH::update {} failed to decode payload {:?}",
                            topic, payload
                        );
                        return WidgetResult::NoEffect;
                    }
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
    /*
                   Joystick X-axis : -50.0 Volt
    |-----+------|------V-----|-----+------|--------------|
    -128        -100           0           100            128

    */

    fn draw(&mut self, ui: &mut egui::Ui) {
        let _ = major_ticks(self.min, self.max);
        let s = format!("{:.3} {}", self.value, self.suffix);
        let rect = inside_rect(self.rect, 1.0);
        {
            // horizontal line through center
            ui.painter().hline(
                Rangef::new(rect.min.x, rect.max.x),
                rect.left_center().y,
                Stroke::new(1.0, Color32::BLACK),
            );
            // tick marks
            for marker in self.minor_ticks.iter() {
                let x = scale(*marker, self.min, self.max, rect.min.x, rect.max.x);
                ui.painter().vline(
                    x,
                    Rangef::new(rect.center().y, rect.center().y + 5.0),
                    Stroke::new(1.0, Color32::BLACK),
                );
            }
        }
        {
            // tick texts
            for marker in self.minor_ticks.iter() {
                let x = scale(*marker, self.min, self.max, rect.min.x, rect.max.x);
                let text_rect = Rect::from_min_max(
                    Pos2::new(x - 20.0, rect.max.y - 12.0),
                    Pos2::new(x + 20.0, rect.max.y - 12.0),
                );
                ui.put(text_rect, egui::Label::new(format!("{}", marker)));
            }
            // value text
            let text_rect = Rect::from_min_max(rect.left_top(), rect.right_center());
            ui.put(
                text_rect,
                egui::Label::new(format!("{} {}",  self.value, self.suffix))
            );
        }
        {
            // triangle needle - V shape
            let y = rect.center().y; // bottom y coord needle
            let x = scale(self.value, self.min, self.max, rect.min.x, rect.max.x);
            let pos1 = Pos2::new(x - 5.0, y - 15.0);
            let pos2 = Pos2::new(x, y);
            let pos3 = Pos2::new(x + 5.0, y - 15.0);
            let points = vec![pos1, pos2, pos3];
            let path_shape =
                PathShape::convex_polygon(points, Color32::RED, Stroke::new(2.0, Color32::RED));

            ui.painter().add(path_shape);
        }
    }
}

impl GaugeH {
    pub fn new(rect: Rect, cfg: &WidgetParams) -> Self {
        let min = cfg.get_or_default("min", 0.0);
        let max = cfg.get_or_default("max", 1.0);
        let (major_step, major_ticks) = major_ticks(min, max);
        let minor_ticks = minor_ticks(major_step, min, max);

        Self {
            rect,
            label: cfg.get_or("label", &cfg.name).clone(),
            suffix: cfg.get_or("label", "").clone(),
            src_topic: cfg.get_or("src", "undefined").clone(),
            expire_time: Instant::now()
                + Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            expire_duration: Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            min,
            max,
            value: 0.0,
            major_ticks,
            minor_ticks,
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }

    pub fn fraction(&self, value: f32) -> f32 {
        let mut value = if value < self.min { self.min } else { value };
        value = if value > self.max { self.max } else { value };
        ((value - self.min) / (self.max - self.min)) as f32
    }
}

pub fn major_ticks(min: f32, max: f32) -> (f32, Vec<f32>) {
    let mut m = VecDeque::new();
    let mut start_pow = (max - min).log10().ceil();
    let mut current_pow = (max - min).log10().floor();
    let mut step: f32 = 10f32.powf(current_pow);
    loop {
        m.clear();
        let mut value = 10f32.powf(start_pow);
        loop {
            if in_range(value, min, max) {
                m.push_front(value);
            }
            value -= step;
            if value < min {
                break;
            };
        }
        if m.len() > 2 {
            break;
        }
        current_pow -= 1f32;
        step = 10f32.powf(current_pow);
    }
    m.push_back(max);
    m.push_front(min);
    (step, m.into())
}

fn minor_ticks(major_step: f32, min: f32, max: f32) -> Vec<f32> {
    let mut m = VecDeque::new();
    let mut start_pow = (max - min).log10().ceil();
    let mut current_pow = (max - min).log10().floor();
    let mut minor_step = major_step / 2.0;
    loop {
        m.clear();
        let mut value = 10f32.powf(start_pow);
        loop {
            if in_range(value, min, max) {
                m.push_front(value);
            }
            value -= minor_step;
            if value < min {
                break;
            };
        }
        if m.len() > 7 {
            break;
        }
        current_pow -= 1f32;
        minor_step = minor_step / 2.0;
    }
    m.into()
}

fn scale(value: f32, min: f32, max: f32, x1: f32, x2: f32) -> f32 {
    if value < min {
        x1
    } else if value > max {
        x2
    } else {
        x1 + ((value - min) / (max - min)) * (x2 - x1)
    }
}

fn count_ticks(step: f32, min: f32, max: f32) -> u32 {
    let mut cursor = max;
    let mut count = 0;
    loop {
        cursor -= step;
        if !in_range(cursor, min, max) {
            break;
        };
        count += 1;
    }
    count
}

fn in_range(value: f32, min: f32, max: f32) -> bool {
    value >= min && value <= max
}

/*
examples : 35.1 - 36.1 -> 36
10.0 -> 15.0 ->
*/
