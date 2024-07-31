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
use std::collections::HashMap;
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
    pub props: HashMap<String, String>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub margin: Option<i32>,
}

impl WidgetParams {
    pub fn new(name: String, rect: WidgetRect) -> Self {
        Self {
            name,
            rect,
            props: HashMap::new(),
            height: None,
            width: None,
            margin: None,
        }
    }
    pub fn get(&self,key: &str) -> Option<&String> {
        self.props.get(key)
    }
    pub fn get_or(&self,key: &str, default: &str) -> String {
        self.props.get(key).unwrap_or(&default.to_string()).to_string()
    }

    pub fn get_or_default<T>(&self,key: &str, default: T) -> T where T: FromStr + Copy {
        self.props.get(key).map(|v| v.parse().unwrap_or(default)).unwrap_or(default)
    }


}

pub fn get_widget_params(rect: WidgetRect, element: &Element) -> Result<WidgetParams, String> {
    let mut widget_params = WidgetParams::new(String::from(element.name()), rect);
    for (attr_name, attr_value) in element.attrs() {
        let attr_value = attr_value.to_string();
        match attr_name {
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

            key  => {
                widget_params.props.insert(key.to_string(), attr_value);
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
        "{} : {:?}",
        cfg.name,
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
