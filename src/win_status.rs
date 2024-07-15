use crate::payload_decode;
use crate::PubSubWindow;
use egui::*;
use log::info;

pub struct WinStatus {
    rect: Rect,
    pub title: String,
    pub status: u32,
    pub message: String,
}

impl WinStatus {
    pub fn new() -> Self {
        Self {
            rect: Rect::from_min_size([200.0, 200.0].into(), [300.0, 300.0].into()),
            title: "Default Title".to_owned(),
            status: 0,
            message: String::new(),
        }
    }
}

impl Default for WinStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl PubSubWindow for WinStatus {
    fn show(&mut self, ctx: &egui::Context) {
        let window_id = Id::new("status");
        let popup_id = Id::new("popup");
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE);
        let mut win = egui::Window::new(self.title.clone())
            .id(window_id)
            .default_pos(self.rect.min)
            .frame(frame)
            .title_bar(false)
            .resizable(true)
            .collapsible(true)
            .constrain(false)
            .max_width(600.0)
            .max_height(600.0);
        win.show(ctx, |ui| {
            let line = format!(" Time {:?}", chrono::Local::now());
            ui.label(line.as_str());
            ui.allocate_space(ui.available_size());
            ui.interact(self.rect, popup_id, Sense::click())
                .context_menu(|ui| {
                    ui.label("Context menu");
                    ui.horizontal(|ui| {
                        ui.label("Title: ");
                        ui.text_edit_singleline(&mut self.title);
                    });
                    ui.separator();
                    if ui.button("Reset").clicked() {
                        info!("Reset");
                    }
                });
        });
        self.rect = ctx.memory(|mem| {
            mem.area_rect(window_id)
                .map(|rect| rect)
                .unwrap()
        });
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        if topic == "status" {
            self.status = payload_decode::<u32>(payload).unwrap_or(0);
            self.message = format!("Status: {}", self.status);
        }
    }
}
