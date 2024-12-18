use log::{debug, error, info, trace, warn};
use minicbor::decode::info;
use serde_yaml::Value;

use std::collections::BTreeMap;
use std::fmt::Error;
use std::thread::{self, sleep, Thread};

//get_pos, get_size, value_string_default
use mqtt_async_client::client::{Client, ReadResult, SubscribeTopic};
use mqtt_async_client::client::{Publish, QoS, Subscribe};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task};
use tokio_stream::StreamExt;

use log::*;
use std::io;
use std::io::Write;
use std::result::Result;
use zenoh::buffers::ZSliceBuffer;

use minicbor::encode;
use tokio::io::split;
use tokio::io::AsyncReadExt;
use tokio::select;

use limero::*;
use limero::Endpoint;

use minicbor::display;
use msg::{PubSubCmd, PubSubEvent};
use msg::payload_display;

pub struct MqttPubSubActor {
    cmds: CmdQueue<PubSubCmd>,
    events: EventHandlers<PubSubEvent>,
    url: String,
    pattern: String,
    connected : bool,
}

impl MqttPubSubActor {
    pub fn new(url:&str,pattern:&str) -> Self {
        //    let url = format!("mqtt://{}:{}/", "broker.emqx.io", "1883");
        //       let url = format!("mqtt://{}:{}/", "test.mosquitto.org", "1883");
        MqttPubSubActor {
            cmds: CmdQueue::new(100),
            events: EventHandlers::new(),
            url: url.to_string(),
            pattern: pattern.to_string(),
            connected : false,
        }
    }
}

impl Actor<PubSubCmd, PubSubEvent> for MqttPubSubActor {
    async fn run(&mut self) {
        let mut client = Client::builder()
            .set_url_string(&self.url)
            .unwrap()
            .build()
            .unwrap();
        info!("Mqtt connecting {} ...  ", self.url);
        let subopts = Subscribe::new(vec![SubscribeTopic {
            qos: QoS::AtLeastOnce,
            topic_path: self.pattern.clone(),
        }]);

        if client.connect().await.is_err() {
            error!("Error connecting to MQTT");
            return;
        }
        info!("Mqtt connected {}", self.url);
        self.connected = true;
        match client.subscribe(subopts).await {
            Ok(_) => {
                info!("Subscribed to MQTT");
            }
            Err(e) => {
                error!("Error subscribing: {}", e);
            }
        };
        loop {
            select! {
                cmd = self.cmds.next() => {
                    match cmd {
                        Some(PubSubCmd::Connect { client_id:_ }) => {
                            info!("Connecting to MQTT");
                            self.events.handle(&PubSubEvent::Connected);
                        }
                        Some(PubSubCmd::Disconnect) => {
                            info!("Disconnecting from MQTT");
                            self.events.handle(&PubSubEvent::Disconnected);
                            break;
                        }
                        Some(PubSubCmd::Publish { topic, payload}) => {
                            let s = format!("{}", minicbor::display(payload.as_slice()));

                            // info!("Pub to MQTT : {}:{}", topic, s);
                            let _res = client.publish(&Publish::new(topic, payload)).await;
                        }
                        Some(PubSubCmd::Subscribe { topic }) => {
                            info!("Subscribing to MQTT");
                            let sub_args = vec![&topic];
                            let subopts = Subscribe::new(
                                sub_args
                                    .iter()
                                    .map(|t| SubscribeTopic {
                                        qos: QoS::AtLeastOnce,
                                        topic_path: t.to_string(),
                                    })
                                    .collect(),
                            );
                            match client.subscribe(subopts).await {
                                Ok(_) => {info!("MQTT subscribe {} success.",topic); },
                                Err(e) => {
                                    error!("Error subscribing: {}", e);
                                }
                            };
                        }
                        Some(PubSubCmd::Unsubscribe { topic }) => {
                            info!("Unsubscribing from zenoh");

                           // let _res = static_session.remove_subscriber(&topic).res().await;
                        }
                        None => {
                            info!("PubSubActor::run() None");
                        }
                    }
                },
                    read_result =  client.read_subscriptions() => {
                    match read_result {
                        Ok(msg) => {
                            let topic = msg.topic().to_string();
                            let payload = Vec::from(msg.payload());
                            debug!(
                                "Publish from Mqtt : {} => {} ",
                                topic,
                                payload_display(&payload)
                            );
                            self.events.handle(&PubSubEvent::Publish {topic,payload,}) ;
                        }
                        Err(e) => {
                            error!("PubSubActor::run() error {:?} ",e);
                            self.connected = false;
                            self.events.handle(&PubSubEvent::Disconnected);
                            continue;
                        }
                    }
                }
            }
        }
        error!("Exiting mqtt loop.")
    }

    fn handler(&self) -> Box<dyn Handler<PubSubCmd>> {
        self.cmds.handler()
    }

    fn add_listener(&mut self, listener : Box<dyn Handler<PubSubEvent>>) { 
        self.events.add_listener(listener)
    }
}
