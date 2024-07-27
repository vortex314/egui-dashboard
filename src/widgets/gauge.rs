use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::payload_as_f64;
use crate::payload_decode;
use crate::payload_display;
use crate::store::timeseries;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use egui_plot::PlotPoints;
use epaint::ColorMode;
use epaint::PathStroke;
use epaint::RectShape;
use log::info;
use std::time::Duration;
use std::time::Instant;

pub struct Gauge {
    rect: Rect,
    label: String,
    src_topic: String,
    value: f64,
    expire_time: Instant,
    expire_duration: Duration,
    min: f64,
    max: f64,
}

impl PubSubWidget for Gauge {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if self.src_topic != *topic {
                    WidgetResult::NoEffect
                } else {
                    self.value = payload_as_f64(payload).unwrap_or(payload_decode::<u64>(payload).unwrap_or(self.min as u64 ) as f64);
                    self.expire_time = Instant::now() + self.expire_duration;
                        WidgetResult::Update
                    
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
        let mut range = self.min..=self.max;
        let square = self.rect.width().min(self.rect.height());
        let g = EguiGauge::new(self.value, range, square,Color32::RED)
            .text(self.label.clone());
        let rect = Rect::from_min_size(self.rect.min, egui::vec2(square, square));
        ui.put(rect, g);
    }
}

impl Gauge {
    pub fn new(rect: Rect, config: &WidgetParams) -> Self {
        let expire_duration = Duration::from_millis(config.timeout.unwrap_or(3000) as u64);

        Self {
            rect,
            label: config.label.as_ref().unwrap_or(&config.name).clone(),
            src_topic: config.src_topic.as_ref().unwrap_or(&String::from("")).clone(),
            expire_time: Instant::now() + expire_duration,
            expire_duration,
            min:config.min.unwrap_or(0.0),
            max: config.max.unwrap_or(1.0),
            value: 0.0,
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }

    pub fn fraction(&self, value: f64) -> f32 {
        let mut value = if value < self.min { self.min } else { value };
        value = if value > self.max { self.max } else { value };
        ((value - self.min) / (self.max - self.min)) as f32
    }
}



///! This crate contains a gauge UI element for use with `egui`
///! This gauge displays a numeric value in a manner that resembles a speedometer on a car
///
use egui::{Align2, FontFamily, FontId, Rect, Response, Sense, Shape, Ui};
use epaint::{Color32, PathShape, Pos2, Stroke};
use std::f32::consts::PI;
use std::ops::RangeInclusive;

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
                color: ColorMode::Solid(bg_color),
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
                color: ColorMode::Solid(bg_color),            },
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
                color: ColorMode::Solid(bg_color),            },
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
                color: ColorMode::Solid(bg_color),            },
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

