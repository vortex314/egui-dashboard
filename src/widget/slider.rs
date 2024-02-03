use crate::widget::tag::Tag;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use crate::PubSubCmd;
use egui::containers::Frame;
use egui::*;
use log::error;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::Sender;

pub struct Slider {
    rect: Rect,
    label: String,
    src_topic: String,
    dst_topic: String,
    cmd_sender: Sender<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
    min: f64,
    max: f64,
    unit: String,
    value: f32,
}

impl Widget for Slider {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
        if self.src_topic != topic {
            return WidgetResult::NoEffect;
        }
        match payload.parse() {
            Ok(v) => {
                self.value = v;
                WidgetResult::Update
            }
            Err(e) => {
                error!("{}: {}", topic, e);
                WidgetResult::NoEffect
            }
        }
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        let mut style = egui::Style::default();
        style.spacing.slider_width = self.rect.width();
        ui.set_style(style);
        if ui
            .add(
                egui::Slider::new(&mut self.value, self.min as f32..=self.max as f32)
                    .text(self.label.clone()),
            )
            .changed()
        {
            let _r = self.cmd_sender.try_send(PubSubCmd::Publish {
                topic: self.dst_topic.clone(),
                message: self.value.to_string(),
            });
        };

        Ok(())
    }
}

impl Slider {
    pub fn new(rect: Rect, config: &Tag, cmd_sender: Sender<PubSubCmd>) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic: config.src.as_ref().unwrap_or(&String::from("")).clone(),
            dst_topic: config.dst.as_ref().unwrap_or(&String::from("")).clone(),
            cmd_sender,
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min: config.min.unwrap_or(0.0),
            max: config.max.unwrap_or(1.0),
            unit: config.unit.as_ref().unwrap_or(&String::from("")).clone(),
            value: 0.0,
        }
    }
}
