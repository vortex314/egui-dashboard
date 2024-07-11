use std::convert::Infallible;

use decode::Error;
use log::info;
//pub mod mqtt_bridge;
//pub mod redis_bridge;
use minicbor::*;
use zenoh::buffers::ZSliceBuffer;

#[derive(Clone)]
pub enum PubSubCmd {
    Publish { topic: String, message: Vec<u8> },
    Disconnect,
    Connect,
    Subscribe { topic: String },
    Unsubscribe { topic: String },
}

#[derive(Clone, Debug)]
pub enum PubSubEvent {
    Connected,
    Disconnected,
    Publish { topic: String, message: Vec<u8> },
}

pub fn payload_encode<X>( v: X) -> Vec<u8>
where
    X: Encode<()>,
{
    let mut buffer = Vec::<u8>::new();
    let mut encoder = Encoder::new(&mut buffer);
    let _x = encoder.encode(v);
    _x.unwrap().writer().to_vec()
}

pub fn payload_decode<'a,T>(v: &'a Vec<u8>) -> Result<T, decode::Error>
where T : Decode<'a,()>
{
    let mut decoder = Decoder::new(v);
    decoder.decode::<T>()
}


pub fn payload_display(v: &Vec<u8>) -> String {
    let line:String  = v.iter().map(|b| format!("{:02X} ", b)).collect();
    let s = format!("{}", minicbor::display(v.as_slice()));
    if s.len() == 0 {
        line
    } else {
        s
    }
}