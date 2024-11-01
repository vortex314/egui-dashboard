use msg::payload_as_f64;
use crate::MyAppCmd;
use crate::PubSubWindow;
use egui::*;
use log::info;
use minicbor::data::Int;
use rand::Rng;
use msg::payload_display;

pub struct WinText {
    rect: Rect,
    pub title: String,
    pub src_topic: String,
    pub suffix: String,
    pub current_value: Option<String>,
    pub prefix: String,
    window_id: Id,
    context_menu_id: Id,
}

impl WinText {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            rect: Rect::from_min_size([200.0, 200.0].into(), [300.0, 300.0].into()),
            title: "Default_title".to_owned(),
            src_topic: "default_topic".to_owned(),
            suffix: "".to_owned(),
            current_value: None,
            prefix: "".to_owned(),
            window_id: Id::new(format!("status_{}", rng.gen::<u32>())),
            context_menu_id: Id::new(format!("context_menu_{}", rng.gen::<u32>())),
        }
    }
    pub fn topic(&mut self, topic: &str) -> &mut Self {
        self.src_topic = topic.to_owned();
        self
    }
    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_owned();
        self
    }
    pub fn prefix(&mut self, prefix: &str) -> &mut Self {
        self.prefix = prefix.to_owned();
        self
    }
 
}

fn get_opt(v: &Option<f64>) -> String {
    match v {
        Some(value) => format!("{}", value),
        None => "--".to_owned(),
    }
}

fn round_rect_to_multiple(rect: Rect, multiple: f32) -> Rect {
    let min = rect.min;
    let max = rect.max;
    let min_x = (min.x / multiple).round() * multiple;
    let min_y = (min.y / multiple).round() * multiple;
    let max_x = (max.x / multiple).round() * multiple;
    let max_y = (max.y / multiple).round() * multiple;
    Rect::from_min_max([min_x, min_y].into(), [max_x, max_y].into())
}

impl PubSubWindow for WinText {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE);
        let mut win = egui::Window::new(self.title.clone())
            .id(self.window_id)
            .default_pos(self.rect.min)
//            .current_pos(self.rect.min)
            .frame(frame)
            .title_bar(true)
            .resizable(true)
            .collapsible(true)
            .constrain(false);
        win.show(ctx, |ui| {
            ui.label(self.src_topic.clone());
            ui.label(self.prefix.clone());
            ui.label(self.current_value.clone().unwrap_or("".to_owned()));
            ui.label(self.suffix.clone());
        });
 //       self.rect = ctx.memory(|mem| mem.area_rect(self.window_id).map(|rect| rect).unwrap());
 //       self.rect = round_rect_to_multiple(self.rect, 30.0);
        None
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        if topic == self.src_topic {
            self.current_value = Some(format!("{}",minicbor::display(payload)));
        }
    }
}
