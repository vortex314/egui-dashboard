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

pub struct GaugeR {
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

impl PubSubWidget for GaugeR {
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
                            "GaugeR::update {} failed to decode payload {:?}",
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


    fn draw(&mut self, ui: &mut egui::Ui) {
        let painter = ui.painter();
        let rect = self.rect;
        let desired_size = self.rect.size();
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            self.paint(painter, rect);
        }

        
    }
}

impl GaugeR {
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

    fn paint(&self, painter: &egui::Painter, rect: Rect) {
        let center = rect.center() + Vec2::new(0.0, rect.height() * 0.25);
        let radius = rect.width().min(rect.height()) / 2.0 * 0.8;

        let start_angle = std::f32::consts::PI;
        let end_angle = 2.0 * std::f32::consts::PI;
        let angle_range = end_angle - start_angle;

        // Draw background arc
     //   painter.add(Shape::circle_stroke(center, radius, Stroke::new(4.0, Color32::DARK_GRAY)));

        // Draw colored gauge arc
        let value_normalized = (self.value - self.min) / (self.max - self.min);
        let value_angle = start_angle + angle_range * value_normalized;
        
        let mut points = Vec::new();
        let num_points = 50;
        for i in 0..=num_points {
            let t = i as f32 / num_points as f32;
            let angle = start_angle + t * (angle_range);
            let point = center + Vec2::angled(angle) * radius;
            points.push(point);
        }
        
        painter.add(Shape::line(points, Stroke::new(2.0, Color32::BLACK)));

        // Draw tick marks
        for i in 0..=10 {
            let angle = start_angle + angle_range * (i as f32 / 10.0);
            let start = center + Vec2::angled(angle) * (radius * 0.8);
            let end = center + Vec2::angled(angle) * radius;
            painter.line_segment([start, end], Stroke::new(2.0, Color32::DARK_GRAY));
        }

        // Draw needle
        let needle_angle = start_angle + angle_range * value_normalized;
        let needle_length = radius * 0.9;
        let needle_end = center + Vec2::angled(needle_angle) * needle_length;

        painter.line_segment([center, needle_end], Stroke::new(3.0, Color32::RED));

        // Draw needle pivot
        painter.circle_filled(center, 5.0, Color32::RED);

        // Draw value text
        let value_text = format!("{:.1}", self.value);
        painter.text(
            center + Vec2::new(0.0, -radius * 0.5),
            egui::Align2::CENTER_CENTER,
            &value_text,
            egui::FontId::proportional(20.0),
            Color32::BLACK,
        );

        // Draw min and max labels
        painter.text(
            center + Vec2::angled(start_angle) * (radius * 1.1),
            egui::Align2::CENTER_CENTER,
            format!("{:.1}", self.min),
            egui::FontId::proportional(16.0),
            Color32::DARK_GRAY,
        );
        painter.text(
            center + Vec2::angled(end_angle) * (radius * 1.1),
            egui::Align2::CENTER_CENTER,
            format!("{:.1}", self.max),
            egui::FontId::proportional(16.0),
            Color32::DARK_GRAY,
        );
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
