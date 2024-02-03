
use egui::containers::Frame;
use egui::*;
use tokio::sync::mpsc::Sender;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use crate::widget::tag::Tag;
use crate::widget::rect_border;
use crate::PubSubCmd;
use std::time::Duration;
use std::time::Instant;

pub struct Button {
    rect : Rect,
    label: String,
    cmd_sender:Sender<PubSubCmd>,
    dst_topic: String,
    pressed:String,
    released:String,
    expire_time: Instant   ,
    expire_duration: Duration,
}

impl Widget for Button {
    fn on_message(&mut self, topic: &str, payload: &str) -> WidgetResult {
            return WidgetResult::NoEffect;
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        let mut style = egui::Style::default();
        style.visuals.override_text_color = Some(Color32::WHITE);

        ui.set_style(style);
        if ui.put(rect_border(self.rect), egui::Button::new(&self.label).fill(Color32::from_rgb(0, 0, 255))).clicked() {
            self.cmd_sender.try_send(PubSubCmd::Publish { topic: self.dst_topic.clone(), message: self.pressed.clone() });
        }

        ui.reset_style();
        Ok(())
    }
}

impl Button {
    pub fn new(rect:Rect,config: &Tag,cmd_sender:Sender<PubSubCmd>) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        Self {
            rect,
            label:config.label.as_ref().unwrap_or(&config.name).clone(),
            cmd_sender,
            dst_topic:config.dst.as_ref().unwrap_or(&String::from("")).clone(),
            pressed:config.pressed.as_ref().unwrap_or(&String::from("true")).clone(),
            released:config.released.as_ref().unwrap_or(&String::from("false")).clone(),
            expire_time : Instant::now()+expire_duration,
            expire_duration,
        }
    }    
}
