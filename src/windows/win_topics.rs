use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate:: MyAppCmd;
use msg::payload_display;
use crate::PubSubWindow;
use egui::*;
use egui_modal::Modal;
use log::{error, info};

use crate::WinStatus;
use crate::WinText;

pub struct WinTopics {
    rect: Rect,
    pub title: String,
    pub regexp: String,
    pub status: u32,
    pub topics: HashMap<String, String>,
    windows : Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>,
}

impl WinTopics {
    pub fn new(windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>) -> Self {
        Self {
            rect: Rect::from_min_size([100.0, 100.0].into(), [200.0, 200.0].into()),
            title: "TopicsMqtt".to_owned(),
            regexp: ".*".to_owned(),
            status: 0,
            topics: HashMap::new(),
            windows,
        }
    }

}

impl PubSubWindow for WinTopics {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut cmd = None;
        let window_id = Id::new("win_topics");
        let popup_id = Id::new("win_topics_popup");
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE)
            .stroke(Stroke {
                width: 4.0,
                color: Color32::LIGHT_BLUE,
            })
            .inner_margin(5.0);
        let mut win = egui::Window::new(self.title.clone())
            .id(window_id)
            .default_pos(self.rect.min)
            .frame(frame)
            .title_bar(true)
            .resizable(true)
            .collapsible(true)
            .constrain(false)
            .scroll(true);
            //           .max_width(300.0)
            //           .max_height(600.0)
        //    .anchor(Align2::RIGHT_TOP, Vec2::new(0.0, 0.0));
        win.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter");
                let mut text_edit = egui::TextEdit::singleline(&mut self.regexp);
                let te = text_edit.desired_width(50.0);
                ui.add(te);
            });
            let re = regex::Regex::new(self.regexp.as_str()).unwrap();
            for (topic, payload) in self.topics.iter() {
                if !re.is_match(topic) {
                    continue;
                }
                ui.horizontal(|ui| {
                    let button_name = format!("{}_", topic);
                    let label1 = Button::new(button_name).sense(Sense::click());
                    let response =  ui.add(label1);
                    response.context_menu(|ui| {
                        info!("Context menu for {}", topic);
                        if ui.button("Progress window").clicked() {
                            let mut win_progress = WinStatus::new();
                            win_progress.topic(topic).title(topic);
                            cmd = Some(MyAppCmd::AddWindow(Box::new(win_progress)));
                        }
                        if ui.button("Text window").clicked() {
                            let mut win_text = WinText::new();
                            win_text.topic(topic).title(topic);
                            cmd = Some(MyAppCmd::AddWindow(Box::new(win_text)));
                        }
                        if ui.button("Gauge window").clicked() {
                            let mut win_gauge = WinStatus::new();
                            win_gauge.topic(topic).title(topic);
                            cmd = Some(MyAppCmd::AddWindow(Box::new(win_gauge)));
                        }
                        if ui.button("Close the menu").clicked() {
                            ui.close_menu();
                        }
                    });
                    if response.clicked() {
                        info!("Clicked on {}", topic);

                        let mut win_text = WinText::new();
                        win_text.topic(topic).title(topic);
                        cmd = Some(MyAppCmd::AddWindow(Box::new(win_text)));
                    };
                    /*let line = format!("{}", payload);
                    let label2 = Label::new(line.as_str());
                    ui.add(label2);*/
                });
            }
            //    ui.allocate_space(ui.available_size());
        });
        self.rect = ctx.memory(|mem| mem.area_rect(window_id).map(|rect| rect).unwrap());
        cmd
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        let value_string = payload_display(payload);

        self.topics.insert(topic.to_string(), value_string);
    }
}
