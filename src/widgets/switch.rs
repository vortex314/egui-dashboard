use crate::draw_border;
use crate::file_xml::WidgetParams;
use crate::inside_rect;
use crate::limero::SinkRef;
use crate::limero::SinkTrait;
use crate::payload_decode;
use crate::payload_display;
use crate::payload_encode;
use crate::widgets::PubSubWidget;
use crate::widgets::WidgetResult;
use crate::PubSubCmd;
use crate::WidgetMsg;
use egui::containers::Frame;
use egui::*;
use epaint::RectShape;
use log::info;
use std::time::Duration;
use std::time::Instant;

use super::get_eval_or;
use super::Eval;

pub struct Switch {
    rect: Rect,
    margin: f32,
    label: String,
    text: String,
    text_size: i32,
    src_topic: String,
    dst_topic: String,
    on_state: bool,
    sinkref_cmd: SinkRef<PubSubCmd>,
    expire_time: Instant,
    expire_duration: Duration,
    eval: Eval,
}

impl PubSubWidget for Switch {
    fn update(&mut self, event: &WidgetMsg) -> WidgetResult {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if *topic == self.src_topic {
                    let value = payload_decode::<bool>(&payload);
                    let _ = self.eval.eval_bool(payload).map(|value| {
                            self.on_state = value;
                        });
                    self.expire_time = Instant::now() + self.expire_duration;
                    WidgetResult::Update
                } else {
                    WidgetResult::NoEffect
                }
            }
            WidgetMsg::Tick => {
                if self.expired() {
                    self.on_state = false;
                    WidgetResult::Update
                } else {
                    WidgetResult::NoEffect
                }
            }
        }
    }
    

    fn draw(&mut self, ui: &mut egui::Ui) {
        toggle_ui_compact(ui, &mut false);
    }
}

impl Switch {
    pub fn new(rect: Rect, cfg: &WidgetParams, sinkref_cmd: SinkRef<PubSubCmd>) -> Self {

        Self {
            rect,
            margin: cfg.margin.unwrap_or(5) as f32,
            label: cfg.get_or("label", &cfg.name).clone(),
            text: String::new(),
            text_size: cfg.get_or_default("text_size", 16),
            src_topic: cfg.get_or("src", "undefined").clone(),
            dst_topic: cfg.get_or("dst", "undefined").clone(),
            on_state: false,
            sinkref_cmd,
            expire_time: Instant::now()
                + Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            expire_duration: Duration::from_millis(cfg.get_or_default("timeout", 3000)),
            eval: get_eval_or(cfg, "eval", "msg_str"),
        }
    }

    fn expired(&self) -> bool {
        Instant::now() > self.expire_time
    }
}

fn toggle_ui_compact(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}
