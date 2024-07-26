
use egui::containers::Frame;
use egui::*;
use tokio::sync::mpsc::Sender;
use crate::limero::SinkRef;
use crate::widgets::Widget;
use crate::widgets::WidgetResult;
use crate::widgets::tag::Tag;
use crate::widgets::rect_border;
use crate::PubSubCmd;
use crate::payload_encode;
use crate::limero::SinkTrait;


use std::time::Duration;
use std::time::Instant;

pub struct Button {
    rect : Rect,
    label: String,
    cmd_sender:SinkRef<PubSubCmd>,
    dst_topic: String,
    pressed:String,
    released:String,
    expire_time: Instant   ,
    expire_duration: Duration,
}

impl Widget for Button {
    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) -> WidgetResult {
            return WidgetResult::NoEffect;
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        let mut style = egui::Style::default();
        style.visuals.override_text_color = Some(Color32::WHITE);

        ui.set_style(style);
        if ui.put(rect_border(self.rect), egui::Button::new(&self.label).fill(Color32::from_rgb(0, 0, 255))).clicked() {
            let _ = self.cmd_sender.push(PubSubCmd::Publish { topic: self.dst_topic.clone(), message: payload_encode::<String>(self.pressed.clone()) });
        }

        ui.reset_style();
        Ok(())
    }
}

impl Button {
    pub fn new(rect:Rect,config: &Tag,cmd_sender:SinkRef<PubSubCmd>) -> Self {
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
