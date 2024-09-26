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

pub struct WinTopics {
    rect: Rect,
    pub title: String,
    pub regexp: String,
    pub status: u32,
    pub topics: HashMap<String, String>
}

impl WinTopics {
    pub fn new(windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>) -> Self {
        Self {
            rect: Rect::from_min_size([100.0, 100.0].into(), [200.0, 200.0].into()),
            title: "Topics".to_owned(),
            regexp: ".*".to_owned(),
            status: 0,
            topics: HashMap::new(),
        }
    }

}

impl PubSubWindow for WinTopics {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let window_id = Id::new("win_menu");
        let popup_id = Id::new("win_menu_popup");
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
            .collapsible(false)
            .constrain(false)
            //           .max_width(300.0)
            //           .max_height(600.0)
            .anchor(Align2::RIGHT_TOP, Vec2::new(0.0, 0.0));
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
                    let label1 = Button::new(topic).sense(Sense::click());
                    if ui.add(label1).clicked() {
                        info!("Clicked on {}", topic);
                    };
                    /*let line = format!("{}", payload);
                    let label2 = Label::new(line.as_str());
                    ui.add(label2);*/
                });
            }
            //    ui.allocate_space(ui.available_size());
        });
        self.rect = ctx.memory(|mem| mem.area_rect(window_id).map(|rect| rect).unwrap());
        None
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        let value_string = payload_display(payload);

        self.topics.insert(topic.to_string(), value_string);
    }
}
