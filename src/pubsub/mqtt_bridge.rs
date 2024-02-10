extern crate log;
use log::{debug, error, info, trace, warn};
use mqtt_async_client::client::Unsubscribe;
use mqtt_async_client::client::UnsubscribeTopic;
use serde_yaml::Value;
use tokio::join;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use std::collections::BTreeMap;
use std::fmt::Error;
use std::thread::{self, sleep, Thread};

use crate::pubsub::{PubSubCmd, PubSubEvent};
use mqtt_async_client::client::{Client, ReadResult, SubscribeTopic};
use mqtt_async_client::client::{Publish, QoS, Subscribe};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task};
use tokio_stream::StreamExt;

pub async fn mqtt(
    url: &str,
    publish_sender: Sender<PubSubEvent>,
    cmd_receiver: &mut Receiver<PubSubCmd>,
) -> Result<(), Error> {
    loop {
        let mut client = Client::builder()
            .set_url_string(&url)
            .unwrap()
            .build()
            .unwrap();
        info!("Mqtt connecting {} ...  ", url);
        let sub_args = vec!["#"];
        let subopts = Subscribe::new(
            sub_args
                .iter()
                .map(|t| SubscribeTopic {
                    qos: QoS::AtLeastOnce,
                    topic_path: t.to_string(),
                })
                .collect(),
        );
        client.connect().await.unwrap();
        match client.subscribe(subopts).await {
            Ok(_) => {
                info!(" MQTT connected.");
            }
            Err(e) => {
                error!("Error subscribing: {}", e);
            }
        };

        select!(
            cmd = cmd_receiver.recv() =>  {
                handle_cmd(&mut client, cmd);
            }
            msg = client.read_subscriptions() =>{
                handle_publish(publish_sender, msg);
            }
        );

        let _r = client.disconnect().await;
        info!("MQTT disconnect {:?}", _r);
    }
}

async fn handle_cmd(client: &mut Client, cmd: Option<PubSubCmd>) {
    match cmd {
        Some(cmd) => {
            info!("PubSubCmd {:?}", cmd);
            match cmd {
                PubSubCmd::Unsubscribe { pattern } => {
                    let topics = vec![UnsubscribeTopic::new(pattern)];
                    let _r = client.unsubscribe(Unsubscribe::new(topics)).await;
                }
                PubSubCmd::Publish { topic, message } => {
                    let payload = message.as_bytes().to_vec();
                    let r = client.publish(&Publish::new(topic, payload)).await;
                }
                PubSubCmd::Subscribe { pattern } => {
                    let sub_args = vec![pattern.as_str()];
                    let subopts = Subscribe::new(
                        sub_args
                            .iter()
                            .map(|t| SubscribeTopic {
                                qos: QoS::AtLeastOnce,
                                topic_path: t.to_string(),
                            })
                            .collect(),
                    );
                    let _r = client.subscribe(subopts).await;
                }
            }
        }
        None => {
            info!("rx_cmd closed");
        }
    }
}

async fn handle_publish(publish_sender: Sender<PubSubEvent>, read_result: Result<ReadResult>) {
    match read_result {
        Ok(msg) => {
            info!("Mqtt topic: {}", msg.topic().to_string(),);
            let _r = publish_sender
                .send(PubSubEvent::Publish {
                    topic: msg.topic().to_string(),
                    message: String::from_utf8_lossy(msg.payload()).to_string(),
                })
                .await;
            match _r {
                Ok(_) => {}
                Err(e) => {
                    error!("Error sending PubSubEvent::Publish: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Error MQTT recv {}", e);
        }
    }
}
