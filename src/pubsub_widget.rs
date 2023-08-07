use serde_yaml::Value;

use tokio::task::block_in_place;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::sync::mpsc::{Sender,Receiver};
use tokio::task;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct PublishMessage {
    topic: String,
    message: String,
}


#[derive(Debug, Clone)]
pub enum PubSubEvent {
    Publish{ topic: String, message: String},
}


pub enum PubSubCmd {
    Subscribe{ pattern: String },
    Unsubscribe{ pattern: String },
    Publish{ topic: String, message: String },
}


pub trait PubSubWidget {
    fn new(grid: &mut Grid, config: &Value, tx_redis_cmd: Sender<PubSubCmd>) -> Self
    where
        Self: Sized;
    fn on_publish(&mut self, topic: &String, message: &String);
}

pub fn split_underscore(str: &String) -> (Option<&str>, Option<&str>) {
    let mut it = str.split("_");
    (it.next(), it.next())
}

fn get_array_of_2(object: &Value, key: &str) -> Option<(i64, i64)> {
    info!(
        " get array of 2 for '{}' in {:?}",
        key,
        object[key].as_sequence()
    );
    let field1 = object[key]
        .as_sequence()
        .and_then(|seq| Some(seq[0].clone()))
        .and_then(|v| v.as_i64());

    let field2 = object[key]
        .as_sequence()
        .and_then(|seq| Some(seq[1].clone()))
        .and_then(|v| v.as_i64());

    field1
        .and(field2)
        .and(Some((field1.unwrap(), field2.unwrap())))
}

pub fn get_size(object: &Value) -> Option<(i32, i32)> {
    let pos = get_array_of_2(object, "size");
    pos.and_then(|(v1, v2)| {
        let f1 = i32::try_from(v1);
        let f2 = i32::try_from(v2);
        if f1.is_ok() && f2.is_ok() {
            Some((f1.unwrap(), f2.unwrap()))
        } else {
            None
        }
    })
}

pub fn get_pos(object: &Value) -> Option<(usize, usize)> {
    let pos = get_array_of_2(object, "pos");
    pos.and_then(|(v1, v2)| {
        let f1 = usize::try_from(v1);
        let f2 = usize::try_from(v2);
        if f1.is_ok() && f2.is_ok() {
            Some((f1.unwrap(), f2.unwrap()))
        } else {
            None
        }
    })
}

pub fn value_string_default(object: &Value, key: &str, default: &str) -> String {
    object[key]
        .as_str()
        .map(String::from)
        .unwrap_or(String::from(default))
}