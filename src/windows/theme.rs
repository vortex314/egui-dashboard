use egui::Color32;
pub struct Theme {
    pub active_foreground_color : Vec<Color32>,
    pub static_color : Color32,
    pub background_color : Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            active_foreground_color: vec![Color32::BLUE,Color32::RED,Color32::GREEN,Color32::YELLOW,],
            static_color: Color32::BLACK,
            background_color: Color32::WHITE,
        }
    }
}

pub fn theme() -> Theme {
    Theme::default()
}