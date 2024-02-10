use crate::widget::tag::Tag;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use egui::containers::Frame;
use egui::*;
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Gauge {
    rect: Rect,
    label: String,
    src_topic: String,
    value: f64,
    expire_time: Instant,
    expire_duration: Duration,
    min: f64,
    max: f64,
}

impl Widget for Gauge {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        self.expire_time = Instant::now() + self.expire_duration;
        self.value = payload.parse().unwrap_or(0.0);
        WidgetResult::Update
    }
    fn draw(&mut self, ui: &mut egui::Ui) -> Result<(), String> {
//        info!("Gauge draw {:?}",self.major_ticks());
        let mut range = self.min..=self.max;
        let square = self.rect.width().min(self.rect.height());
        let g = egui_gauge::Gauge::new(self.value, range, square,Color32::RED)
            .text(self.label.clone());
        ui.put(self.rect, g);
        Ok(())
    }
}

impl Gauge {
    pub fn new(rect: Rect, config: &Tag) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic: config.src.as_ref().unwrap_or(&String::from("")).clone(),
            value: 0.0,
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min: config.min.unwrap_or(0.0),
            max: config.max.unwrap_or(100.0),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }

    fn major_ticks(&self) -> Vec<f64> {
        let mut ticks = Vec::new();
        let range = self.max - self.min;
        let num_major_ticks = 5;
        let num_minor_ticks_per_major = 4;
        let major_increment = range / (num_major_ticks - 1) as f64;
        let rounding_factor = 10.0_f64.powf(major_increment.log10().floor());

        let rounded_min_value = (self.min / rounding_factor).floor() * rounding_factor;
        let rounded_max_value = (self.max / rounding_factor).ceil() * rounding_factor;
        let rounded_range = rounded_max_value - rounded_min_value;
        let major_increment = rounded_range / (num_major_ticks - 1) as f64;

        for i in 0..num_major_ticks {
            let tick_value = rounded_min_value + i as f64 * major_increment as f64;
            ticks.push(tick_value);
            /*if i < num_major_ticks - 1 && num_minor_ticks_per_major > 0 {
                let minor_increment = major_increment / (num_minor_ticks_per_major + 1) as f64;
                for j in 1..=num_minor_ticks_per_major {
                    let minor_tick_value = tick_value + j as f64 * minor_increment as f64;
                    ticks.push(minor_tick_value);
                }
            }*/
        }
        ticks
        
    }
}
