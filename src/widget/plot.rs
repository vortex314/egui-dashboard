use crate::payload_decode;
use crate::widget::rect_border;
use crate::widget::tag::Tag;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use crate::store::timeseries;
use egui::containers::Frame;
use egui::*;
use egui_plot::PlotPoints;
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Plot {
    rect: Rect,
    label: String,
    src_topic: String,
    expire_time: Instant,
    timespan: Duration,
    min: f64,
    max: f64,
    value: f64,
    unit: String,
    timeseries: timeseries::TimeSeries,
}

impl Widget for Plot {
    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        self.value = payload_decode(payload).unwrap_or(self.min);
        self.timeseries.add(Instant::now(), self.value);
        WidgetResult::Update
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        let s = format!("{} {}", self.value, self.unit);
        let rect = rect_border(self.rect);
        self.timeseries.clean();
        let line_data = self.timeseries.get_series();
        let earlier = Instant::now()-self.timespan;
        let line_points: PlotPoints = line_data
            .iter()
            .map(|entry| {
                let x = entry.time.duration_since(earlier).as_secs_f64();
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
        let mut child_ui = ui.child_ui(self.rect, layout);
        let _r  = pl.show(&mut child_ui, |plot_ui| {
            plot_ui.line(line);
        });
        Ok(())
    }
}

impl Plot {
    pub fn new(rect: Rect, config: &Tag) -> Self {
        let timespan = Duration::from_millis(config.timespan.unwrap_or(10000) as u64);
        let min = config.min.unwrap_or(0.0);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic: config.src.as_ref().unwrap_or(&String::from("")).clone(),
            expire_time: Instant::now() + timespan,
            timespan,
            min,
            max: config.max.unwrap_or(1.0),
            value: min,
            unit: config.unit.as_ref().unwrap_or(&String::from("")).clone(),
            timeseries: timeseries::TimeSeries::new(
                config.name.clone(),
                Duration::from_millis(config.timespan.unwrap_or(3000) as u64),
                config.samples.unwrap_or(100) as usize,
            ),
        }
    }
    pub fn fraction(&self, value: f64) -> f32 {
        let mut value = if value < self.min { self.min } else { value };
        value = if value > self.max { self.max } else { value };
        ((value - self.min) / (self.max - self.min)) as f32
    }
}
