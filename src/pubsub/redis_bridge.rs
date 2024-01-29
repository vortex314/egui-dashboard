extern crate log;
use log::{debug, error, info, trace, warn};
use redis::RedisResult;
use serde_yaml::Value;

use std::fmt::Error;
use std::thread::{self, Thread};
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::sleep;
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task};
use tokio_stream::StreamExt;

use crate::pubsub::PubSubEvent;
use crate::PubSubCmd;
/* 
async fn redis_publish_received(url: &str, mut tx_publish_received: Sender<PubSubEvent>) {
    info!("Redis config {:?} ", url);
    loop {
        let url = String::from(url);
        let client = redis::Client::open(url.clone()).unwrap();
        info!("Redis connecting {} ...  ", url);
        let connection = client.get_async_connection().await;
        match connection {
            Ok(_) => {}
            Err(e) => {
                error!("Error connecting: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
        let mut pubsub = connection.unwrap().into_pubsub();
        //    let redis_cmd_channel = connection.into_monitor();
        let r = pubsub.psubscribe("*").await;
        match r {
            Ok(()) => {
                info!("Redis psubscribe *");
            }
            Err(e) => {
                error!("Error psubscribe: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }

        let mut pubsub_stream = pubsub.into_on_message();

        while let Some(msg) = pubsub_stream.next().await {
            let s: String = msg.get_payload().unwrap();
            info!(
                "Redis topic: {} => {} ",
                msg.get_channel_name().to_string(),
                s
            );
            match tx_publish_received
                .send(PubSubEvent::Publish {
                    topic: msg.get_channel_name().to_string(),
                    message: msg.get_payload().unwrap(),
                })
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("Error sending: {}", e);
                }
            }
        }
    }
}

async fn redis_cmd_received(url: &str, mut rx_redis_cmd: Receiver<PubSubCmd>) {
    info!("Redis config {:?} ", url);
    loop {
        let url = String::from(url);
        let client = redis::Client::open(url.clone()).unwrap();
        info!("Redis connecting {} ...  ", url);
        let mut publish_conn = client.get_multiplexed_async_connection().await;

        match publish_conn {
            Ok(_) => {}
            Err(e) => {
                error!("Error connecting: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
        //    let redis_cmd_channel = connection.into_monitor();

        while let Some(cmd) = rx_redis_cmd.recv().await {
            info!("PubSubCmd {:?}", cmd);
            match cmd {
                PubSubCmd::Unsubscribe { pattern } => {
                    let _: () = redis::cmd("PUNSUBSCRIBE")
                        .arg(pattern)
                        .query_async(&mut publish_conn)
                        .await
                        .unwrap();
                }
                PubSubCmd::Publish { topic, message } => {
                    let _: () = redis::cmd("PUBLISH")
                        .arg(topic)
                        .arg(message)
                        .query_async(&mut publish_conn)
                        .await
                        .unwrap();
                }
                PubSubCmd::Subscribe { pattern } => {
                    let _: () = redis::cmd("PSUBSCRIBE")
                        .arg(pattern)
                        .query_async(&mut publish_conn)
                        .await
                        .unwrap();
                }
            }
        }
    }
}*/

async fn rxd_publish(url: &str, events: Sender<PubSubEvent>) -> RedisResult<()>{
    let client = redis::Client::open(url)?;
    let mut con = client.get_connection()?;
    let mut pubsub = con.as_pubsub();
    let _ = pubsub.psubscribe("*");
    loop {
        let msg = pubsub.get_message()?;
        let payload : String = msg.get_payload()?;
        let _r = events.send(PubSubEvent::Publish { topic: msg.get_channel_name().to_string(), message: payload } ).await;
    }
}

pub async fn redis(
    url: &str,
    tx_publish_received: Sender<PubSubEvent>,
    rx_cmd: Receiver<PubSubCmd>,
) -> Result<(), Error> {
    let _r = rxd_publish(url, tx_publish_received).await;

    Ok(())
}
