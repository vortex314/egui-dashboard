use crate::MyAppCmd;
use crate::PubSubWindow;

use egui::epaint::ColorMode;
use log::info;
use minicbor::data::Int;
use msg::payload_as_f64;
use rand::Rng;
use egui::Id;
use egui::Rect;
use egui::Rounding;
use egui::Ui;
use egui::Sense;
use egui::{Align2, FontFamily, FontId,  Response, Shape};
use crate::epaint::{Color32, PathShape, Pos2, Stroke};
use crate::epaint::PathStroke;
use std::f32::consts::PI;
use std::ops::RangeInclusive;

pub struct WinGauge {
    rect: Rect,
    pub title: String,
    pub src_topic: String,
    pub suffix: String,
    pub current_value: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    window_id: Id,
    context_menu_id: Id,
}

impl WinGauge {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            rect: Rect::from_min_size([200.0, 200.0].into(), [300.0, 300.0].into()),
            title: "Latency".to_owned(),
            src_topic: "src/esp32/sys/latency".to_owned(),
            suffix: "msec".to_owned(),
            current_value: None,
            min_value: None,
            max_value: None,
            window_id: Id::new(format!("status_{}", rng.gen::<u32>())),
            context_menu_id: Id::new(format!("context_menu_{}", rng.gen::<u32>())),
        }
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

impl PubSubWindow for WinGauge {
    fn show(&mut self, ctx: &egui::Context) -> Option<MyAppCmd> {
        let mut frame = egui::Frame::default()
            .rounding(Rounding::ZERO)
            .fill(egui::Color32::WHITE);
        let mut win = egui::Window::new(self.title.clone())
            .id(self.window_id)
            .default_pos(self.rect.min)
            .current_pos(self.rect.min)
            .frame(frame)
            .title_bar(false)
            .resizable(true)
            .collapsible(false)
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
            let new_value = payload_as_f64(payload).unwrap();
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


///! This crate contains a gauge UI element for use with `egui`
///! This gauge displays a numeric value in a manner that resembles a speedometer on a car
///


pub struct EguiGauge {
    value: f64,
    min_value: f64,
    max_value: f64,
    size: f32,
    color: Color32,
    text: String,
}

impl EguiGauge {
    /// Create a gauge which displays the given value as part of the given range. The given size is
    /// with width and height of the gauge. The given color is the color used for the value
    /// indicator arc.
    pub fn new<Num: emath::Numeric>(
        value: Num,
        range: RangeInclusive<Num>,
        size: f32,
        color: Color32,
    ) -> Self {
        Self {
            value: value.to_f64(),
            min_value: range.start().to_f64(),
            max_value: range.end().to_f64(),
            size,
            color,
            text: Default::default(),
        }
    }

    /// Text to be displayed under the value in the center of the gauge
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    fn inner_width(&self) -> f32 {
        self.size - self.text_clearance() * 2.0
    }

    fn text_clearance(&self) -> f32 {
        self.size / 10.0
    }

    fn x_f(&self, rect: Rect, angle: i32, radius: f32) -> f32 {
        self.center(rect).x + (angle as f32 * PI / 180.0).cos() * radius
    }

    fn y_f(&self, rect: Rect, angle: i32, radius: f32) -> f32 {
        self.center(rect).y - (angle as f32 * PI / 180.0).sin() * radius
    }

    fn radius(&self) -> f32 {
        self.inner_width() / 2.0
    }

    fn thickness(&self) -> f32 {
        self.inner_width() / 15.0
    }

    fn center(&self, rect: Rect) -> Pos2 {
        Pos2 {
            x: rect.left() + rect.width() / 2.0,
            y: rect.bottom() - rect.height() / 2.0,
        }
    }

    fn value_to_angle(&self, v: f64) -> i32 {
        ((270.0 - ((v - self.min_value) / (self.max_value - self.min_value)) * 270.0) - 45.0) as i32
    }

    fn angle(&self) -> i32 {
        self.value_to_angle(self.value)
    }

    fn paint(&mut self, ui: &mut Ui, outer_rect: Rect) {
        let rect = Rect {
            min: Pos2 {
                x: outer_rect.min.x + self.text_clearance(),
                y: outer_rect.min.y + self.text_clearance(),
            },
            max: Pos2 {
                x: outer_rect.max.x - self.text_clearance(),
                y: outer_rect.max.y - self.text_clearance(),
            },
        };

        let visuals = ui.style().noninteractive();

        // uncomment to show bounding rect for debugging
        // ui.painter()
        //  .rect(outer_rect, 0.0, visuals.bg_fill, visuals.bg_stroke);

        let text_color = visuals.text_color();
        let arc_bg_color = if ui.visuals().dark_mode {
            Color32::WHITE
        } else {
            Color32::GRAY
        };
        let bg_color = visuals.bg_fill;

        self.paint_background_circle(ui, rect, arc_bg_color, bg_color);
        self.paint_colored_circle(ui, rect, bg_color);
        self.paint_center_mask(ui, rect, bg_color);
        self.paint_skirt_mask(ui, rect, bg_color);
        self.paint_end_caps(ui, rect, bg_color, arc_bg_color);
        self.paint_value_circle(ui, rect);
        self.write_center_value(ui, rect, text_color);
        self.write_values_around_circle(ui, rect, text_color);

        if !self.text.is_empty() {
            self.write_text(ui, rect, text_color);
        }
    }

    fn write_text(&mut self, ui: &mut Ui, rect: Rect, text_color: Color32) {
        let center = self.center(rect);
        let wrap_width = self.inner_width() * 2.0 / 3.0;
        let text = ui.painter().layout(
            self.text.clone(),
            FontId {
                size: self.inner_width() / 10.0,
                family: FontFamily::Monospace,
            },
            text_color,
            wrap_width,
        );
        let visuals = ui.style().noninteractive();
        ui.painter().galley(
            Pos2 {
                x: center.x - text.rect.width() / 2.0,
                y: center.y + self.inner_width() / 5.0 - text.rect.height() / 2.0,
            },
            text,
            visuals.bg_fill
        );
    }

    fn write_values_around_circle(&mut self, ui: &mut Ui, rect: Rect, text_color: Color32) {
        let mut value = self.min_value;
        loop {
            let angle = self.value_to_angle(value);
            ui.painter().text(
                Pos2 {
                    x: self.x_f(rect, angle, self.radius() + self.thickness()),
                    y: self.y_f(rect, angle, self.radius() + self.thickness()),
                },
                Align2::CENTER_CENTER,
                (value as i32).to_string(),
                FontId {
                    size: self.inner_width() / 15.0,
                    family: FontFamily::Monospace,
                },
                text_color,
            );
            if value == self.max_value {
                break;
            }
            value += (self.max_value - self.min_value) / 6.0;
            if (self.max_value - value) < 1.0 {
                value = self.max_value;
            }
        }
    }

    fn write_center_value(&mut self, ui: &mut Ui, rect: Rect, text_color: Color32) {
        ui.painter().text(
            self.center(rect),
            Align2::CENTER_CENTER,
            self.value.to_string(),
            FontId {
                size: self.inner_width() / 5.0,
                family: FontFamily::Monospace,
            },
            text_color,
        );
    }

    fn paint_value_circle(&mut self, ui: &mut Ui, rect: Rect) {
        ui.painter().circle(
            Pos2 {
                x: self.x_f(rect, self.angle(), self.radius() - self.thickness() / 2.0),
                y: self.y_f(rect, self.angle(), self.radius() - self.thickness() / 2.0),
            },
            self.thickness() / 2.0,
            Color32::WHITE,
            Stroke {
                width: 1.0,
                color: self.color,
            },
        );
    }

    fn paint_end_caps(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        bg_color: Color32,
        arc_bg_color: Color32,
    ) {
        ui.painter().circle(
            Pos2 {
                x: self.x_f(rect, 225, self.radius() - self.thickness() / 2.0),
                y: self.y_f(rect, 225, self.radius() - self.thickness() / 2.0),
            },
            self.thickness() / 2.0,
            self.color,
            Stroke {
                width: 0.0,
                color: bg_color,
            },
        );
        ui.painter().circle(
            Pos2 {
                x: self.x_f(rect, -45, self.radius() - self.thickness() / 2.0),
                y: self.y_f(rect, -45, self.radius() - self.thickness() / 2.0),
            },
            self.thickness() / 2.0,
            arc_bg_color,
            Stroke {
                width: 0.0,
                color: bg_color,
            },
        );
    }

    fn paint_center_mask(&mut self, ui: &mut Ui, rect: Rect, bg_color: Color32) {
        ui.painter().add(Shape::Path(PathShape {
            points: (-45..=225)
                .map(|angle: i32| Pos2 {
                    x: self.x_f(rect, angle, self.radius() - self.thickness()),
                    y: self.y_f(rect, angle, self.radius() - self.thickness()),
                })
                .collect(),
            closed: true,
            fill: bg_color,
            stroke: PathStroke {
                width: 0.0,
                color: ColorMode::TRANSPARENT,
            },
        }));
    }

    fn paint_colored_circle(&mut self, ui: &mut Ui, rect: Rect, bg_color: Color32) {
        ui.painter().add(Shape::Path(PathShape {
            points: (self.angle()..=225)
                .map(|angle: i32| Pos2 {
                    x: self.x_f(rect, angle, self.radius()),
                    y: self.y_f(rect, angle, self.radius()),
                })
                .chain(std::iter::once(self.center(rect)))
                .collect(),
            closed: true,
            fill: self.color,
            stroke: PathStroke {
                width: 0.0,
                color: ColorMode::Solid(bg_color),
            },
        }));
    }

    fn paint_background_circle(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        arc_bg_color: Color32,
        bg_color: Color32,
    ) {
        ui.painter().add(Shape::Path(PathShape {
            points: (-45..=225)
                .map(|angle: i32| Pos2 {
                    x: self.x_f(rect, angle, self.radius()),
                    y: self.y_f(rect, angle, self.radius()),
                })
                .chain(std::iter::once(self.center(rect)))
                .collect(),
            closed: true,
            fill: arc_bg_color,
            stroke: PathStroke {
                width: 0.0,
                color: ColorMode::Solid(bg_color),
            },
        }));
    }

    fn paint_skirt_mask(&mut self, ui: &mut Ui, rect: Rect, bg_color: Color32) {
        ui.painter().add(Shape::Path(PathShape {
            points: vec![
                Pos2 {
                    x: self.x_f(rect, -45, self.radius()),
                    y: self.y_f(rect, -45, self.radius()),
                },
                Pos2 {
                    x: self.x_f(rect, 225, self.radius()),
                    y: self.y_f(rect, 225, self.radius()),
                },
                Pos2 {
                    x: self.x_f(rect, 225, self.radius() - self.thickness()),
                    y: self.y_f(rect, 225, self.radius() - self.thickness()),
                },
                Pos2 {
                    x: self.x_f(rect, -45, self.radius() - self.thickness()),
                    y: self.y_f(rect, -45, self.radius() - self.thickness()),
                },
            ],
            closed: true,
            fill: bg_color,
            stroke: PathStroke {
                width: 2.0,
                color: ColorMode::Solid(bg_color),
            },
        }));
    }

    fn add_contents(&mut self, ui: &mut Ui) -> Response {
        let desired_size = egui::vec2(self.size, self.size);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        response.widget_info(|| egui::WidgetInfo::slider(true,self.value, &self.text));

        if ui.is_rect_visible(rect) {
            self.paint(ui, rect);
        }

        response
    }
}

impl egui::Widget for EguiGauge {
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.add_contents(ui)
    }
}


