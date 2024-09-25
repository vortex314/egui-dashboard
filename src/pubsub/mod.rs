pub mod mqtt_pubsub;
pub mod zenoh_pubsub;

use std::any::Any;
use std::convert::Infallible;

use data::Int;
use decode::Error;
use log::info;
//pub mod mqtt_bridge;
//pub mod redis_bridge;
use minicbor::*;
use minicbor::data::*;
use zenoh::buffers::ZSliceBuffer;

use anyhow::Result;
