use limero::CmdQueue;
use limero::EventHandlers;
use log::*;
use msg::payload_display;
use msg::PubSubCmd;
use msg::PubSubEvent;
use tokio::sync::mpsc::Sender;
use std::collections::BTreeMap;
use std::io;
use std::io::Write;
use std::result::Result;
use zenoh::buffers::ZSliceBuffer;

use minicbor::encode;
use tokio::io::split;
use tokio::io::AsyncReadExt;
use tokio::select;

use limero::*;

use minicbor::display;
use zenoh::open;
use zenoh::prelude::r#async::*;
use zenoh::subscriber::Subscriber;

pub struct ZenohPubSubActor {
    cmds: CmdQueue<PubSubCmd>,
    event_handlers: EventHandlers<PubSubEvent>,
    config: zenoh::config::Config,
}

impl ZenohPubSubActor {
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
        ZenohPubSubActor {
            cmds: CmdQueue::new(100),
            event_handlers: EventHandlers::new(),
            config: config.unwrap(),
        }
    }

    pub fn sender(&self) -> Sender<PubSubCmd> {
        self.cmds.sender()
    }
}

impl Actor<PubSubCmd, PubSubEvent> for ZenohPubSubActor {
    async fn run(&mut self) {
        let static_session: &'static mut Session =
            Session::leak(zenoh::open(config::default()).res().await.unwrap());
        let subscriber = static_session.declare_subscriber("**").res().await.unwrap();
        loop {
            select! {
                cmd = self.cmds.next() => {
                    match cmd {
                        Some(PubSubCmd::Connect { client_id:_}) => {
                            info!("Connecting to zenoh");
                            self.event_handlers.handle(&PubSubEvent::Connected);
                        }
                        Some(PubSubCmd::Disconnect) => {
                            info!("Disconnecting from zenoh");
                            self.event_handlers.handle(&PubSubEvent::Disconnected);
                        }
                        Some(PubSubCmd::Publish { topic, payload}) => {
                            debug!("To zenoh: {}:{}", topic,payload_display(&payload));
                            let _res = static_session
                                .put(&topic,payload.as_slice())
                                .encoding(KnownEncoding::AppOctetStream)
                                .res().await;
                        }
                        Some(PubSubCmd::Subscribe { topic }) => {
                            info!("Subscribing to zenoh");
                            let subscriber = static_session.declare_subscriber(&topic).res().await;
                            match subscriber {
                                Ok(sub) => {

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
                },
                msg = subscriber.recv_async() => {
                    match msg {
                        Ok(msg) => {
                            let topic = msg.key_expr.to_string();
                            let payload = msg.payload.contiguous().to_vec();
                            debug!("From zenoh: {}:{}", topic,msg::cbor::to_string(&payload));
                            let event = PubSubEvent::Publish { topic, payload };
                            self.event_handlers.handle(&event);
                        }
                        Err(e) => {
                            info!("PubSubActor::run() error {} ",e);
                        }
                    }
                }
            }
        }
    }

  fn handler(&self) -> Box<dyn limero::Handler<PubSubCmd>> {
        self.cmds.handler()
    }

    fn add_listener(&mut self, listener : Box<dyn limero::Handler<PubSubEvent> + 'static>) { 
        self.event_handlers.add_listener(listener)
    }
}


