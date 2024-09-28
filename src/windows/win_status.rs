use msg::payload_as_f64;
use crate::MyAppCmd;
use crate::PubSubWindow;
use egui::*;
use log::info;
use minicbor::data::Int;
use rand::Rng;

pub struct WinStatus {
    rect: Rect,
    pub title: String,
    pub src_topic: String,
    pub prefix: String,
    pub suffix: String,
    pub current_value: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    window_id: Id,
    context_menu_id: Id,
}

impl WinStatus {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            rect: Rect::from_min_size([200.0, 200.0].into(), [300.0, 300.0].into()),
            title: "Latency".to_owned(),
            src_topic: "src/esp32/sys/latency".to_owned(),
            prefix: "".to_owned(),
            suffix: "msec".to_owned(),
            current_value: None,
            min_value: None,
            max_value: None,
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

impl PubSubWindow for WinStatus {
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
            ui.vertical_centered(|ui| {
                ui.label(self.title.clone());
                let line = format!("CURRENT : {} {}", get_opt(&self.current_value), self.suffix);
                ui.label(line.as_str());
                let line = format!("MIN : {} {}", get_opt(&self.min_value), self.suffix);
                ui.label(line.as_str());
                let line = format!("MAX: {} {}", get_opt(&self.max_value), self.suffix);
                ui.label(line.as_str());
                ui.allocate_space(ui.available_size());
                ui.interact(self.rect, self.context_menu_id, Sense::click())
                    .context_menu(|ui| {
                        self.context_menu(ui);
                    });
            });
        });
        self.rect = ctx.memory(|mem| mem.area_rect(self.window_id).map(|rect| rect).unwrap());
        self.rect = round_rect_to_multiple(self.rect, 30.0);
        None
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        if topic == self.src_topic {
            if let Ok(value) = payload_as_f64(payload) {
                self.current_value = Some(value);
                if self.min_value.is_none() {
                    self.min_value = Some(value);
                };
                if self.max_value.is_none() {
                    self.max_value = Some(value);
                }

                if value < self.min_value.unwrap() {
                    self.min_value = Some(value);
                }
                if value > self.max_value.unwrap() {
                    self.max_value = Some(value);
                }
            }

        }
    }
}
