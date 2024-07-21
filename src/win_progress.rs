use crate::payload_decode;
use crate::pubsub::decode_f64;
use crate::MyAppCmd;
use crate::PubSubWindow;
use egui::*;
use log::info;
use minicbor::data::Int;
use rand::Rng;

pub struct WinProgress {
    rect: Rect,
    pub title: String,
    pub src_topic: String,
    pub suffix: String,
    pub current_value: Option<f64>,
    pub prefix: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    window_id: Id,
    context_menu_id: Id,
}

impl WinProgress {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            rect: Rect::from_min_size([200.0, 200.0].into(), [300.0, 300.0].into()),
            title: "Progress".to_owned(),
            src_topic: "src/esp32/sys/latency".to_owned(),
            suffix: "msec".to_owned(),
            current_value: None,
            prefix: "".to_owned(),
            min_value: None,
            max_value: None,
            window_id: Id::new(format!("status_{}", rng.gen::<u32>())),
            context_menu_id: Id::new(format!("context_menu_{}", rng.gen::<u32>())),
        }
    }
    pub fn fraction(&self, value: f64) -> f32 {
        let min = self.min_value.unwrap_or(0.0);
        let max = self.max_value.unwrap_or(1.0);
        let mut value = if value < min { min } else { value };
        value = if value > max { max } else { value };
        ((value - min) / (max- min)) as f32
    }
    fn context_menu(&mut self, ui: &mut Ui) {
        let topics = vec![
            "src/esp32/sys/uptime",
            "src/esp32/sys/latency",
            "src/esp32/sys/heap_free",
            "src/esp32/sys/heap_used",
        ];
        ui.label("Context menu");
        ui.horizontal(|ui| {
            ui.label("Title: ");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.horizontal(|ui| {
            ui.label("Suffix: ");
            ui.text_edit_singleline(&mut self.suffix);
        });
        let mut topic_selected = self.src_topic.clone();
        ui.horizontal(|ui| {
            ui.label("Source topic: ");
            egui::ComboBox::from_id_source("Source topic")
                .selected_text(format!("{:?}", topic_selected))
                .show_ui(ui, |ui| {
                    for topic in topics.iter() {
                        ui.selectable_value(
                            &mut topic_selected,
                            topic.to_string(),
                            topic.to_string(),
                        );
                    }
                })
        });
        if topic_selected != self.src_topic {
            self.title = topic_selected.clone();
            self.src_topic = topic_selected;
            self.current_value = None;
            self.min_value = None;
            self.max_value = None;
        }
        ui.allocate_space(ui.available_size());
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

impl PubSubWindow for WinProgress {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE);
        let mut win = egui::Window::new(self.title.clone())
            .id(self.window_id)
            .default_pos(self.rect.min)
            .current_pos(self.rect.min)
            .frame(frame)
            .title_bar(true)
            .resizable(true)
            .collapsible(true)
            .constrain(false);
        win.show(ctx, |ui| {
            let s = format!("{} {}", self.current_value.unwrap_or(0.0), self.suffix);
            ui.put(
                self.rect,
                egui::ProgressBar::new(self.fraction(self.current_value.unwrap_or(0.0)))
                    .fill(Color32::RED)
                    .rounding(Rounding::ZERO)
                    .desired_height(self.rect.height())
                    .desired_width(self.rect.width())
                    .text(s),
            );
        });
        self.rect = ctx.memory(|mem| mem.area_rect(self.window_id).map(|rect| rect).unwrap());
        self.rect = round_rect_to_multiple(self.rect, 30.0);
        None
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        if topic == self.src_topic {
            let new_value = decode_f64(payload).unwrap();
            self.current_value = Some(new_value);
            if self.min_value.is_none() {
                self.min_value = Some(new_value);
            };
            if self.max_value.is_none() {
                self.max_value = Some(new_value);
            }

            if new_value < self.min_value.unwrap() {
                self.min_value = Some(new_value);
            }
            if new_value > self.max_value.unwrap() {
                self.max_value = Some(new_value);
            }
        }
    }
}
