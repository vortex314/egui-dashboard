use egui::Color32;
struct Theme {
    pub active_foreground_color : Color32,
    pub static_color : Color32,
    pub background_color : Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            active_foreground_color: Color32::BLUE,
            static_color: Color32::BLACK,
            background_color: Color32::WHITE,
        }
    }
}

pub fn theme() -> Theme {
    Theme::default()
}