use crate::payload_display;
use crate::store::sub_table::OrderSort;
use crate::store::*;
use crate::widget::tag::Tag;
use crate::widget::Widget;
use crate::widget::WidgetResult;
use egui::containers::Frame;
use egui::*;
use egui_extras::{Column, TableBuilder};
use regex::Regex;
use egui::TextStyle::Body;
use egui::FontFamily::Proportional;
use egui::TextStyle::Heading;

use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Table {
    rect: Rect,
    label: String,
    text_size: i32,
    table: sub_table::SubTable,
    src_topic: String,
    expire_time: Instant,
    expire_duration: Duration,
    regex: Regex,
    reverse:bool,
}

impl Widget for Table {
    fn on_message(&mut self, topic: &str, payload: &Vec<u8>) -> WidgetResult {
        if self.regex.is_match(topic) {
            self.table.add(topic.to_string(), payload_display(&payload));
            WidgetResult::Update
        } else {
            WidgetResult::NoEffect
        }
    }
    fn draw(&mut self, ui: &mut Ui) -> Result<(), String> {
        let layout = Layout::top_down(Align::LEFT);
  //      info!("Plot {} : {:?}", self.label, self.rect);
        let mut child_ui = ui.child_ui(self.rect, layout);
        let mut style = egui::Style::default();
        // small font
        style.text_styles.insert(Body, FontId::new(12.0, Proportional));
        style.text_styles.insert(Heading, FontId::new(12.0, Proportional));
        style.visuals.override_text_color = Some(Color32::BLACK);
        child_ui.set_style(style);
        let mut builder = TableBuilder::new(&mut child_ui)
            .column(Column::initial(60.0))
            .column(Column::initial(80.0))
            .column(Column::initial(160.0).resizable(true))
            .column(Column::remainder().resizable(true))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    if ui.heading("Count").clicked() {
                        self.table.order(OrderSort::Count,self.reverse);
                        self.reverse = !self.reverse;
                    };
                });
                header.col(|ui| {
                    if ui.heading("Time").clicked() {
                        self.table.order(OrderSort::Time,self.reverse);
                        self.reverse = !self.reverse;
                    };
                });
                header.col(|ui| {
                    if ui.heading("Topic").clicked() {
                        self.table.order(OrderSort::Topic,self.reverse);
                        self.reverse = !self.reverse;
                    };
                });
                header.col(|ui| {
                    if ui.heading("Value").clicked() {
                        self.table.order(OrderSort::Value,self.reverse);
                        self.reverse = !self.reverse;
                    };
                });
            });

        builder.body(|mut body| {
            self.table.entries.iter().for_each(|x| {
                body.row(15.0, |mut row| {
                    row.col(|ui| {
                        ui.label(x.count.to_string());
                    });
                    row.col(|ui| {
                        ui.label(x.date_time.clone().format("%H:%M:%S").to_string().as_str());
                    });
                    row.col(|ui| {
                        //ui.label(x.topic.as_str());
                        ui.add(egui::Label::new(x.topic.as_str()).truncate(true));
                    });
                    row.col(|ui| {
                        //  ui.label(x.value.as_str());
                        ui.add(egui::Label::new(x.value.as_str()).truncate(true));
                    });
                });
            });
        });

        Ok(())
    }
}

impl Table {
    pub fn new(rect: Rect, config: &Tag) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);
        let src_pattern = config
            .src
            .as_ref()
            .unwrap_or(&String::from("IMPOSSIBLE"))
            .clone();
        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            text_size: config.text_size.unwrap_or(20),
            table: sub_table::SubTable::new(),
            src_topic: src_pattern.clone(),
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            regex: Regex::new(src_pattern.as_str()).unwrap(),
            reverse:false,
        }
    }
}
