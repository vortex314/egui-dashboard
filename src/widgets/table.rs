use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::store::sub_table;
use crate::store::sub_table::OrderSort;
use crate::store::timeseries;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::FontFamily::Proportional;
use egui::TextStyle::Body;
use egui::TextStyle::Heading;
use egui::*;
use egui_extras::Column;
use egui_extras::TableBuilder;
use egui_plot::PlotPoints;
use epaint::RectShape;
use log::info;
use msg::payload_display;
use regex::Regex;
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
    reverse: bool,
}

impl PubSubWidget for Table {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.regex.is_match(topic) {
                    self.table.add(topic.to_string(), msg::cbor::to_string(&payload));
                    WidgetResult::Update
                } else {
                    WidgetResult::NoEffect
                }
            }
            WidgetMsg::Tick => {
                if Instant::now() > self.expire_time {
                    return WidgetResult::Update;
                }
                WidgetResult::NoEffect
            }
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let mut frame = Frame::default()
            .stroke(egui::Stroke::new(1.0, Color32::LIGHT_GRAY))
            .rounding(Rounding::ZERO)
            .shadow(egui::Shadow::NONE)
            .outer_margin(Margin::ZERO)
            .inner_margin(Margin::ZERO)
            .fill(egui::Color32::WHITE);
        let rect = inside_rect(self.rect, 5.0);
        let window = Window::new(self.label.as_str())
            .id(egui::Id::new(self.label.as_str()))
            .collapsible(false)
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .default_pos(rect.min)
            .fixed_rect(rect)
            .scroll([true, true])
            .frame(frame)
            .show(ui.ctx(), |ui| {
                self.draw_table(ui);
            })
            .unwrap()
            .response
            .hover_pos();
        if let Some(pos) = window {
            info!("Table {} pos {:?}", self.label, pos);
        }
    }
}

impl Table {
    pub fn new(rect: Rect, config: &WidgetParams) -> Self {
        Self {
            rect,
            label: config.get_or("label", &config.name).clone(),
            src_topic: config.get_or("src", "undefined").clone(),
            text_size: config.get_or_default("text_size", 20),
            expire_time: Instant::now()
                + Duration::from_millis(config.get_or_default("timeout", 5000)),
            expire_duration: Duration::from_millis(config.get_or_default("timeout", 5000)),
            table: sub_table::SubTable::new(),
            regex: Regex::new(r"").unwrap(),
            reverse: false,
        }
    }

    pub fn draw_table(&mut self, ui: &mut egui::Ui) {
        let mut style = egui::Style::default();
        // small font
        // style.text_styles.insert(Body, FontId::new(12.0, Proportional));
        style
            .text_styles
            .insert(Heading, FontId::new(12.0, Proportional));
        style.visuals.override_text_color = Some(Color32::BLACK);
        ui.set_style(style);
        let mut builder = TableBuilder::new(ui)
            .vscroll(true)
            .striped(true)
            .sense(Sense::hover())
            .column(Column::initial(60.0))
            .column(Column::initial(80.0))
            .column(Column::initial(160.0))
            .column(Column::remainder().resizable(true))
            .header(12.0, |mut header| {
                header.col(|ui| {
                    if ui.heading("Count").clicked() {
                        self.table.order(OrderSort::Count, self.reverse);
                        self.reverse = !self.reverse;
                    };
                });
                header.col(|ui| {
                    if ui.heading("Time").clicked() {
                        self.table.order(OrderSort::Time, self.reverse);
                        self.reverse = !self.reverse;
                    };
                });
                header.col(|ui| {
                    if ui.heading("Topic").clicked() {
                        self.table.order(OrderSort::Topic, self.reverse);
                        self.reverse = !self.reverse;
                    };
                });
                header.col(|ui| {
                    if ui.heading("Value").clicked() {
                        self.table.order(OrderSort::Value, self.reverse);
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
                        ui.add(egui::Label::new(x.topic.as_str()).truncate());
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new(x.value.as_str()).truncate());
                    });
                });
            });
        });
    }
}
