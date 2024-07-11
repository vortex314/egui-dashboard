use log::*;
use std::collections::BTreeMap;
use std::io;
use std::io::Write;
use std::result::Result;
use zenoh::buffers::ZSliceBuffer;

use minicbor::encode;
use tokio::io::split;
use tokio::io::AsyncReadExt;
use tokio::select;

use crate::limero::Sink;
use crate::limero::SinkRef;
use crate::limero::SinkTrait;
use crate::limero::Source;
use crate::payload_display;
use crate::ActorTrait;
use crate::SourceTrait;



use minicbor::display;
use zenoh::open;
use zenoh::prelude::r#async::*;
use zenoh::subscriber::Subscriber;
use crate::pubsub::{PubSubCmd, PubSubEvent};


pub struct PubSubActor {
    cmds: Sink<PubSubCmd>,
    events: Source<PubSubEvent>,
    config: zenoh::config::Config,
}

impl PubSubActor {
    pub fn new() -> Self {
        let mut config = Config::from_file("./zenohd.json5");
        if config.is_err() {
            error!(
                "Error reading zenohd.json5 file, using default config {}",
                config.err().unwrap()
            );
            config = Ok(config::default());
        } else {
            info!("Using zenohd.json5 file");
        }
        PubSubActor {
            cmds: Sink::new(100),
            events: Source::new(),
            config: config.unwrap(),
        }
    }
}

impl ActorTrait<PubSubCmd, PubSubEvent> for PubSubActor {
    async fn run(&mut self) {
        let static_session: &'static mut Session =
            Session::leak(zenoh::open(config::default()).res().await.unwrap());
        loop {
            select! {
                cmd = self.cmds.next() => {
                    match cmd {
                        Some(PubSubCmd::Connect) => {
                            info!("Connecting to zenoh");
                            self.events.emit(PubSubEvent::Connected);
                        }
                        Some(PubSubCmd::Disconnect) => {
                            info!("Disconnecting from zenoh");
                            self.events.emit(PubSubEvent::Disconnected);
                        }
                        Some(PubSubCmd::Publish { topic, message}) => {
                            let s = format!("{}", minicbor::display(message.as_slice()));
                            let s :&str = s.as_str();
                            let v:Value = s.into();
                            info!("Publishing to zenoh: {}", v);
                            let _res = static_session
                                .put(&topic,v)
                                .encoding(KnownEncoding::TextPlain)
                                .res().await;
                        }
                        Some(PubSubCmd::Subscribe { topic }) => {
                            info!("Subscribing to zenoh");
                            let subscriber = static_session.declare_subscriber(&topic).res().await;
                            match subscriber {
                                Ok(sub) => {
                                    let emitter =  self.events.clone();
                                    tokio::spawn(async move {
                                        while let Ok(sample) = sub.recv_async().await {
                                            let data = sample.payload.contiguous().to_vec();
                                            let s = format!("{}", minicbor::display(&data));
                                            info!("Received: {}:[{}]:{}",sample.key_expr.to_string(), data.len(),payload_display(&data));
                                            emitter.emit(PubSubEvent::Publish {
                                                topic: sample.key_expr.to_string(),
                                                message:sample.payload.contiguous().to_vec(),
                                            });
                                        };
                                    });
                                }
                                Err(e) => {
                                    error!("Error subscribing to zenoh: {}", e);
                                }
                            }
                        }
                        Some(PubSubCmd::Unsubscribe { topic }) => {
                            info!("Unsubscribing from zenoh");
                           // let _res = static_session.remove_subscriber(&topic).res().await;
                        }
                        None => {
                            info!("PubSubActor::run() None");
                        }
                    }
                }
            }
        }
    }

    fn sink_ref(&self) -> SinkRef<PubSubCmd> {
        self.cmds.sink_ref()
    }
}

impl SourceTrait<PubSubEvent> for PubSubActor {
    fn add_listener(&mut self, sink: SinkRef<PubSubEvent>) {
        self.events.add_listener(sink);
    }
}
