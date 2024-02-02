extern crate log;
use fred::clients::RedisClient;
use fred::interfaces::{ClientLike, EventInterface, PubsubInterface};
use fred::types::{Blocking, MultipleStrings, ReconnectPolicy, RedisConfig, RespVersion, ServerConfig, TracingConfig};
use log::{debug, error, info, trace, warn};
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
use fred::*;

pub async fn redis(
    url: &str,
    tx_publish_received: Sender<PubSubEvent>,
    rx_cmd: &mut Receiver<PubSubCmd>,
) -> Result<(), Error> {
    info!("Redis config {:?} ", url);
    let mut config = RedisConfig::default();
    let reconnect_policy = ReconnectPolicy::Exponential {
        attempts: 10,
        max_attempts: 10,
        min_delay: 1,
        max_delay: 1000,
        mult: 2,
        jitter: 3,
    };
    config.server = ServerConfig::new_centralized("limero.ddns.net", 6379);
    config.tracing = TracingConfig::default();
    config.tracing.enabled=true;
    config.version = RespVersion::RESP3;
    config.blocking = Blocking::default();


    let client = RedisClient::new(config, None, None, Some(reconnect_policy));
    let task  = client.init().await.unwrap();
    info!("redis connecting ... ");
    let _r = client.connect().await.unwrap();
    info!("redis connected ");
    let patterns = MultipleStrings::from(vec!["*"]);
    client.psubscribe(patterns).await.unwrap();
    info!("redis subscribed ");
    client.on_message(move |msg| {
        info!(
            "Redis topic: {} => {:?} ",
            msg.channel,
            msg.value.as_string()
        );
        let _r = tx_publish_received.send(PubSubEvent::Publish {
            topic: msg.channel.to_string(),
            message: msg.value.as_string().unwrap(),
        });
        Ok(())
    });
    loop {
        select! {
            cmd = rx_cmd.recv() => {
                match cmd {
                    Some(cmd) => {
                        info!("PubSubCmd {:?}", cmd);
                        match cmd {
                            PubSubCmd::Unsubscribe { pattern } => {
                                let _r = client.punsubscribe(pattern).await;
                            }
                            PubSubCmd::Publish { topic, message } => {
                               // let _r = client.publish(topic, message).await;
                            }
                            PubSubCmd::Subscribe { pattern } => {
                                let _r = client.psubscribe(pattern).await;
                            }
                        }
                    }
                    None => {
                        info!("rx_cmd closed");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
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
}

async fn handle_cmds(
    pub_connection: & redis::Connection,
    sub_connection: &mut redis::PubSub<'_>,
    cmd_channel: Receiver<PubSubCmd>,
) -> Result<(), RedisError> {
    while let Some(cmd) = cmd_channel.recv().await {
        info!("PubSubCmd {:?}", cmd);
        match cmd {
            PubSubCmd::Unsubscribe { pattern } => {
                let _r = sub_connection.punsubscribe(pattern).await;
            }
            PubSubCmd::Publish { topic, message } => {
                let _r = pub_connection.publish(topic, message).await;
            }
            PubSubCmd::Subscribe { pattern } => {
                let _r = sub_connection.psubscribe(pattern).await;
            }
        }
    }
    Ok(())
}
async fn rxd_publish(
    url: &str,
    event_sender: Sender<PubSubEvent>,
    cmd_receiver: Receiver<PubSubCmd>,
) -> RedisResult<()> {
    let client = redis::Client::open(url)?;
    let mut pub_connection = client.get_connection()?;
    let mut sub_connection = pub_connection.as_pubsub();
    let mut pub_conn2 = pub_connection.clone();
    let _ = sub_connection.psubscribe("*");
    loop {
        let msg = sub_connection.get_message()?;
        let payload: String = msg.get_payload()?;
        let _r = event_sender
            .send(PubSubEvent::Publish {
                topic: msg.get_channel_name().to_string(),
                message: payload,
            })
            .await;
    }
}
*/
