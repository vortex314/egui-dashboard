use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use log::{debug, error, info, trace, warn};

pub mod file_change;
pub mod file_xml;


use file_change::FileChangeActor;
use file_xml::load_xml_file;
pub use file_xml::WidgetParams;



