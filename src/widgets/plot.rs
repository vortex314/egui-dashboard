use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::payload_as_f64;
use crate::payload_decode;
use crate::payload_display;
use crate::store::timeseries;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use egui_plot::PlotPoints;
use epaint::RectShape;
use log::info;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
pub struct Plot {
    rect: Rect,
    label: String,
    text: String,
    text_size: i32,
    src_topic: String,
    expire_time: Instant,
    expire_duration: Duration,
    max_timespan: Duration,
    max_samples: usize,
    min: f64,
    max: f64,
    value: f64,
    unit: String,
    timeseries: timeseries::TimeSeries,
}

impl PubSubWidget for Plot {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        let previous_text = self.text.clone();
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.src_topic != *topic {
                    WidgetResult::NoEffect
                } else {
                    self.value = payload_as_f64(payload).unwrap_or(0.0);
                    self.timeseries.add(Instant::now(), self.value);
                    WidgetResult::Update
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
        let id = Id::new(self.label.clone());
        draw_border(self.rect, ui);
        let s = format!("{} {}", self.value, self.unit);
        let rect = inside_rect(self.rect, 1.0);
        self.timeseries.clean();
        let line_data = self.timeseries.get_series();
        let earlier = Instant::now() - self.max_timespan;
        let now = Instant::now();
        let line_points: PlotPoints = line_data
            .iter()
            .map(|entry| {
                let x = -now.duration_since(entry.time).as_secs_f64();
                [x, entry.value]
            })
            .collect();
        let line = egui_plot::Line::new(line_points);
        let pl = egui_plot::Plot::new("example_plot")
            .height(rect.height())
            .width(rect.width())
            .show_axes(true)
            .show_grid(true);
        let layout = Layout::top_down(Align::LEFT);
        //    info!("Plot {} : {:?}", self.label, self.rect);
        let mut child_ui = ui.child_ui(self.rect, layout, None);
        let _r = pl.show(&mut child_ui, |plot_ui| {
            plot_ui.line(line);
        });
    }
}

impl Plot {
    pub fn new(rect: Rect, config: &WidgetParams) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            text: String::new(),
            text_size: config.text_size.unwrap_or(20),
            src_topic: config
                .src_topic
                .as_ref()
                .unwrap_or(&String::from(""))
                .clone(),
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            max_timespan: Duration::from_secs(config.max_timespan.unwrap_or(60) as u64),
            max_samples: config.max_samples.unwrap_or(100) as usize,
            min: config.min.unwrap_or(0.0),
            max: config.max.unwrap_or(1.0),
            value: 0.0,
            unit: config.unit.as_ref().unwrap_or(&String::from("")).clone(),
            timeseries: timeseries::TimeSeries::new(
                config.name.clone(),
                Duration::from_millis(config.max_timespan.unwrap_or(3000) as u64),
                config.max_samples.unwrap_or(100) as usize,
            ),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }

    pub fn fraction(&self, value: f64) -> f32 {
        let mut value = if value < self.min { self.min } else { value };
        value = if value > self.max { self.max } else { value };
        ((value - self.min) / (self.max - self.min)) as f32
    }
}
