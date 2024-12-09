use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::{button, MyAppCmd, WinProgress};
use crate::{PubSubWindow, WinGauge};
use egui::*;
use egui_modal::Modal;
use log::info;
use msg::payload_display;

use crate::WinStatus;

const FOLDER_ICON: ImageSource<'_> = include_image!("../../assets/folder.png");
const SAVE_ICON: ImageSource<'_> = include_image!("../../assets/save.png");
const GAUGE_ICON: ImageSource<'_> = include_image!("../../assets/gauge.png");
const GRAPH_ICON: ImageSource<'_> = include_image!("../../assets/graph.png");
const PROGRESS_ICON: ImageSource<'_> = include_image!("../../assets/progress.png");
const TEXT_ICON: ImageSource<'_> = include_image!("../../assets/text.png");
const LABEL_ICON: ImageSource<'_> = include_image!("../../assets/label.png");

#[derive(Debug)]
enum IconEvent {
    Folder,
    Save,
    Gauge,
    Graph,
    Progress,
    Text,
    Label,
}

pub struct WinMenu {
    rect: Rect,
    pub title: String,
    pub regexp: String,
    pub status: u32,
    pub topics: Vec<String>,
    windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>,
}

impl WinMenu {
    pub fn new(windows: Arc<Mutex<Vec<Box<dyn PubSubWindow + Send>>>>) -> Self {
        Self {
            rect: Rect::from_min_size([100.0, 100.0].into(), [200.0, 200.0].into()),
            title: "Topics".to_owned(),
            regexp: ".*".to_owned(),
            status: 0,
            topics: vec!["status".to_owned(), "progress".to_owned()],
            windows,
        }
    }

    fn create_new_window(&self, name: &str) -> Box<dyn PubSubWindow + Send> {
        if name == "status" {
            return Box::new(WinStatus::new());
        }
        if name == "progress" {
            return Box::new(WinProgress::new());
        }

        Box::new(WinStatus::new())
    }

    fn button_bar( &mut self, ui:&mut Ui )-> Option<IconEvent> {
    
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Image::new(FOLDER_ICON)
                        .max_width(20.0)
                        .rounding(1.0)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                info!("Clicked on folder icon");
                return Some(IconEvent::Folder);
            }
            if ui
                .add(
                    egui::Image::new(SAVE_ICON)
                        .max_width(20.0)
                        .rounding(1.0)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                info!("Clicked on save icon");
                return Some(IconEvent::Save);
            }
            if ui
                .add(
                    egui::Image::new(GAUGE_ICON)
                        .max_width(20.0)
                        .rounding(1.0)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                info!("Clicked on gauge icon");

                return SOme(IconEvent::Gauge);
            }
            if ui
                .add(
                    egui::Image::new(GRAPH_ICON)
                        .max_width(20.0)
                        .rounding(1.0)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                info!("Clicked on graph icon");
                return Some(IconEvent::Graph);
            }
            if ui
                .add(
                    egui::Image::new(PROGRESS_ICON)
                        .max_width(20.0)
                        .rounding(1.0)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                info!("Clicked on progress icon");
                return Some(IconEvent::Progress);
            }
            if ui
                .add(
                    egui::Image::new(TEXT_ICON)
                        .max_width(20.0)
                        .rounding(1.0)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                info!("Clicked on text icon");
                return Some(IconEvent::Text);
            }
            if ui
                .add(
                    egui::Image::new(LABEL_ICON)
                        .max_width(20.0)
                        .rounding(1.0)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                info!("Clicked on label icon");
                return Some(IconEvent::Label);
            } 
        });
        None
    }
}

impl PubSubWindow for WinMenu {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut cmd = None;
        let window_id = Id::new("win_menu");
        let popup_id = Id::new("win_menu_popup");
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE)
            .stroke(Stroke {
                width: 4.0,
                color: Color32::LIGHT_BLUE,
            })
            .inner_margin(0.0);
        let mut win = egui::Window::new(self.title.clone())
            .id(window_id)
            .default_pos(self.rect.min)
            .frame(frame)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .constrain(false)
            .scroll(false)
            .max_height(20.0)
            .anchor(Align2::LEFT_TOP, Vec2::new(0.0, 0.0));
        win.show(ctx, |ui| {
            self.button_bar(ui).map(|icon_event| {
                info!(" icon button {:?} clicked ",icon_event);
                match icon_event {
                    IconEvent::Folder => {
                        let win_gauge = WinGauge::new();
                        let r = self.windows.try_lock();
                        let _ = r.map(|mut r| r.push(Box::new(win_gauge)));
                    }
                    _ => {}
                }
                cmd = Some(MyAppCmd::AddWindow(self.create_new_window("status")))});
            //    ui.allocate_space(ui.available_size());
        });
        self.rect = ctx.memory(|mem| mem.area_rect(window_id).map(|rect| rect).unwrap());
        cmd
    }

    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) {
        info!("on_message {} {}", topic, minicbor::display(payload));
        self.topics.push(topic.to_owned());
    }
}
