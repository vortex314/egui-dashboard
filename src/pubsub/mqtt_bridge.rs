extern crate log;
use log::{debug, error, info, trace, warn};
use serde_yaml::Value;

use std::collections::BTreeMap;
use std::fmt::Error;
use std::thread::{self, sleep, Thread};

use crate::config::{
    get_pos, get_size, value_string_default
};
use crate::pubsub::{PubSubCmd,PubSubEvent};
use mqtt_async_client::client::{Client, ReadResult, SubscribeTopic};
use mqtt_async_client::client::{Publish, QoS, Subscribe};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task};
use tokio_stream::StreamExt;

pub async fn mqtt(config: Value, tx_broadcast: broadcast::Sender<PubSubEvent>) {
    loop {
        let url = format!(
            "mqtt://{}:{}/",
            config["host"].as_str().or(Some("localhost")).unwrap(),
            config["port"].as_str().or(Some("1883")).unwrap()
        );
        let mut client = Client::builder()
            .set_url_string(&url)
            .unwrap()
            .build()
            .unwrap();
        info!("Mqtt connecting {} ...  ", url);
        let sub_args = vec!["#"];
        let subopts = Subscribe::new(sub_args.iter().map(|t|
            SubscribeTopic { qos: QoS::AtLeastOnce, topic_path: t.to_string() }
        ).collect());
        client.connect().await.unwrap();
        match client.subscribe(subopts).await {
            Ok(_) => {}
            Err(e) => {
                error!("Error subscribing: {}", e);
            }
        };

        loop {
            let read_result = client.read_subscriptions().await;
            match read_result {
                Ok(msg) => {
                    info!("Mqtt topic: {}", msg.topic().to_string(),);
                    match tx_broadcast.send(PubSubEvent::Publish {
                        topic: msg.topic().to_string(),
                        message: String::from_utf8_lossy(msg.payload()).to_string(),
                    }) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error sending PubSubEvent::Publish: {}", e);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
