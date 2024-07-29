use crate::egui::Rect;
use crate::payload_as_f64;
use crate::payload_display;
use evalexpr::build_operator_tree;
use evalexpr::ContextWithMutableFunctions;
use evalexpr::ContextWithMutableVariables;
use evalexpr::HashMapContext;
use evalexpr::Node;
use evalexpr::Value;
use log::{debug, error, info, trace, warn};
use minicbor::decode;
use minidom::Element;
use std::fs::File;
use std::hash::Hash;
use std::io::Error;
use std::io::Read;
use std::time::Duration;
use std::{collections::BTreeMap, str::FromStr};

use serde_xml_rs::from_str;

#[derive(Debug, Clone, Copy)]
pub struct WidgetRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl WidgetRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}
#[derive(Clone)]
pub struct Eval {
    pub node: Node,
    pub context: HashMapContext,
}

#[derive(Clone,Debug)]
pub struct WidgetParams {
    pub name: String,
    pub rect: WidgetRect,
    pub label: Option<String>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub margin: Option<i32>,
    pub text_size: Option<i32>,
    pub msec: Option<i32>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub timeout: Option<i32>,
    pub src_topic: Option<String>,
    pub dst_topic: Option<String>,
    pub dst_val: Option<String>,
    pub src_val: Option<String>,
    pub pressed: Option<String>,
    pub released: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub unit: Option<String>,
    pub ok: Option<String>,
    pub ko: Option<String>,
    pub url: Option<String>,
    pub image: Option<String>,
    pub on: Option<String>,
    pub off: Option<String>,
    pub children: Vec<WidgetParams>,
    pub max_samples: Option<usize>,
    pub max_timespan: Option<i32>,
    pub eval: Option<String>,
}

impl WidgetParams {
    pub fn new(name: String, rect: WidgetRect) -> Self {
        Self {
            name,
            rect,
            label: None,
            height: None,
            width: None,
            margin: None,
            text_size: None,
            msec: None,
            min: None,
            max: None,
            timeout: None,
            src_topic: None,
            dst_topic: None,
            src_val: None,
            dst_val: None,
            pressed: None,
            released: None,
            prefix: None,
            suffix: None,
            unit: None,
            ok: None,
            ko: None,
            url: None,
            image: None,
            on: None,
            off: None,
            children: Vec::new(),
            max_samples: None,
            max_timespan: None,
            eval: None,
        }
    }
}

pub fn get_widget_params(rect: WidgetRect, element: &Element) -> Result<WidgetParams, String> {
    let mut widget_params = WidgetParams::new(String::from(element.name()), rect);
    for (attr_name, attr_value) in element.attrs() {
        let attr_value = attr_value.to_string();
        match attr_name {
            "label" => {
                widget_params.label = Some(attr_value);
            }
            "src" => {
                widget_params.src_topic = Some(attr_value);
            }
            "dst" => {
                widget_params.dst_topic = Some(attr_value);
            }
            "src_val" => {
                widget_params.src_val = Some(attr_value);
            }
            "dst_val" => {
                widget_params.dst_val = Some(attr_value);
            }
            "pressed" => {
                widget_params.pressed = Some(attr_value);
            }
            "released" => {
                widget_params.released = Some(attr_value);
            }
            "prefix" => {
                widget_params.prefix = Some(attr_value);
            }
            "suffix" => {
                widget_params.suffix = Some(attr_value);
            }
            "unit" => {
                widget_params.unit = Some(attr_value);
            }
            "image" => {
                widget_params.image = Some(attr_value);
            }
            "url" => {
                widget_params.url = Some(attr_value);
            }
            "ok" => {
                widget_params.ok = Some(attr_value);
            }
            "nok" => {
                widget_params.ko = Some(attr_value);
            }
            "h" => {
                widget_params.rect.h = attr_value.parse().expect("Invalid height");
                widget_params.height = Some(widget_params.rect.h);
            }
            "w" => {
                widget_params.rect.w = attr_value.parse().expect("Invalid width");
                widget_params.width = Some(widget_params.rect.w);
            }
            "margin" => {
                widget_params.margin = attr_value.parse().ok();
            }
            "min" => {
                widget_params.min = attr_value.parse().ok();
            }
            "max" => {
                widget_params.max = attr_value.parse().ok();
            }
            "timeout" => {
                widget_params.timeout = attr_value.parse().ok();
            }
            "msec" => {
                widget_params.msec = attr_value.parse().ok();
            }
            "on" => {
                widget_params.on = Some(attr_value);
            }
            "off" => {
                widget_params.off = Some(attr_value);
            }
            "text_size" => {
                widget_params.text_size = attr_value.parse().ok();
            }
            "samples" => {
                widget_params.max_samples = attr_value.parse().ok();
            }
            "timespan" => {
                widget_params.max_timespan = attr_value.parse().ok();
            }
            "eval" => {
                widget_params.eval = Some(attr_value.to_string());
            }
            _ => {
                error!("Unknown attribute: {}", attr_value);
            }
        };
    }
    Ok(widget_params)
}

pub fn load_xml_file(path: &str) -> Result<Element, minidom::Error> {
    let mut file = File::open(path).expect(std::format!("Unable to open file {} ", path).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file ");
    contents.parse::<Element>()
}

pub fn load_dashboard(root: &Element) -> Result<Vec<WidgetParams>, String> {
    let mut widgets: Vec<WidgetParams> = Vec::new();
    let mut cfg = get_widget_params(WidgetRect::new(0, 0, 0, 0), &root)?;
    if cfg.name != "Dashboard" {
        return Err("Invalid config file. Missing Dashboard tag.".to_string());
    }
    let mut rect = WidgetRect::new(0, 0, cfg.width.unwrap_or(1025), cfg.height.unwrap_or(769));
    cfg.rect = rect;
    for child_element in root.children() {
        let child = get_widget_params(rect, child_element)?;
        info!("Loading widget {}", child.name);
        let mut sub_widgets = load_widgets(rect, child_element)?;
        widgets.append(&mut sub_widgets);
        if child.width.is_some() {
            rect.x += child.width.unwrap();
        }
        if child.height.is_some() {
            rect.y += child.height.unwrap();
        }
    }
    Ok(widgets)
}

fn load_widgets(rect: WidgetRect, element: &Element) -> Result<Vec<WidgetParams>, String> {
    let cfg = get_widget_params(rect, element)?;
    let mut widgets: Vec<WidgetParams> = Vec::new();
    let mut rect = cfg.rect;

    info!(
        "{} : {} {:?}",
        cfg.name,
        cfg.label.as_ref().get_or_insert(&String::from("NO_LABEL")),
        cfg.rect
    );

    match cfg.name.as_str() {
        "Row" => {
            rect.h = cfg.height.unwrap_or(rect.h);
            for child_element in element.children() {
                let child = get_widget_params(rect, child_element)?;
                let mut sub_widgets = load_widgets(rect, child_element)?;
                widgets.append(&mut sub_widgets);
                rect.x += child.width.unwrap_or(0);
            }
        }
        "Col" => {
            rect.w = cfg.width.unwrap_or(rect.w);

            for child_element in element.children() {
                let child = get_widget_params(rect, child_element)?;
                let mut sub_widgets = load_widgets(rect, child_element)?;
                widgets.append(&mut sub_widgets);
                rect.y += child.height.unwrap_or(0);
            }
        }
        _ => {
            widgets.push(cfg.clone());
            rect.y = rect.y + cfg.height.unwrap_or(0);
            rect.x = rect.x + cfg.width.unwrap_or(0);
        }
    }
    Ok(widgets)
}

pub fn split_underscore(str: &String) -> (Option<&str>, Option<&str>) {
    let mut it = str.split("_");
    (it.next(), it.next())
}
