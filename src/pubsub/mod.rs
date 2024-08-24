pub mod mqtt_pubsub;

pub mod zenoh_pubsub;

use std::any::Any;
use std::convert::Infallible;

mod payload_cbor;
mod payload_json;
use payload_cbor::*;
use payload_json::*;

use data::Int;
use decode::Error;
use log::info;
//pub mod mqtt_bridge;
//pub mod redis_bridge;
use minicbor::*;
use minicbor::data::*;
use zenoh::buffers::ZSliceBuffer;

use anyhow::Result;

#[derive(Clone,Debug)]
pub enum PubSubCmd {
    Publish { topic: String, payload: Vec<u8> },
    Disconnect,
    Connect,
    Subscribe { topic: String },
    Unsubscribe { topic: String },
}

#[derive(Clone, Debug)]
pub enum PubSubEvent {
    Connected,
    Disconnected,
    Publish { topic: String, payload: Vec<u8> },
}

pub trait PayloadCodec {
    fn as_f64(v:&Vec<u8>) -> Result<f64> where Self:Sized;
    fn as_bool(&self) -> Result<bool> where Self:Sized;
    fn as_int(&self) -> Result<i64> where Self:Sized;
    fn to_string(v:&Vec<u8>) -> Result<String> where Self:Sized;// => display as string
    fn decode<T>(v: &Vec<u8>) -> Result<T> where Self:Sized;
    fn encode<T>(v: &T) -> Vec<u8> where Self:Sized;
}

pub enum SerializationType {
    JSON,
    CBOR,
}

impl dyn PayloadCodec {
    pub fn from(codec: &str) -> Box<Self> {
        match codec {
            "json" => Box::new(Json {}),
            "cbor" => Box::new(Cbor {}),
            _ => Box::new(Json {}),
        }
    }
}

struct Cbor {}

impl PayloadCodec for Cbor {
     fn as_f64(v: &Vec<u8>) -> Result<f64> {
        payload_as_f64_cbor(v)
    }
     fn as_bool(v: &Vec<u8>) -> Result<bool> {
        let f = payload_as_f64_cbor(v)?;
        Ok(f != 0.0)
    }
     fn as_int(v: &Vec<u8>) -> Result<i64> {
        let f = payload_as_f64_cbor(v)?;
        Ok(f as i64)
    }
     fn to_string(v: &Vec<u8>) -> Result<String> {
        Ok(payload_display_cbor(v))
    }
     fn decode<T>(v: &Vec<u8>) -> Result<T> {
        payload_decode_cbor(v)
    }
     fn encode<T>(v: &T) -> Vec<u8> {
        payload_encode_cbor(v)
    }
}

struct Json {}

impl PayloadCodec for Json {
     fn as_f64(v: &Vec<u8>) -> Result<f64> {
        payload_as_f64_json(v)
    }
     fn as_bool(v: &Vec<u8>) -> Result<bool> {
        let f = payload_as_f64_json(v)?;
        Ok(f != 0.0)
    }
     fn as_int(v: &Vec<u8>) -> Result<Int> {
        let f = payload_as_f64_json(v)?;
        Ok(f as Int)
    }
     fn to_string(v: &Vec<u8>) -> Result<String> {
        Ok(payload_display_json(v))
    }
     fn decode<T>(v: &Vec<u8>) -> Result<T> {
        payload_decode_json(v)
    }
     fn encode<T>(v: &T) -> Vec<u8> {
        payload_encode_json(v)
    }
}


