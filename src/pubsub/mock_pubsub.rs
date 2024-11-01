use limero::CmdQueue;
use limero::EventHandlers;
use log::*;
use msg::payload_display;
use msg::PubSubCmd;
use msg::PubSubEvent;
use rand::Rng;
use tokio::sync::mpsc::Sender;
use std::collections::BTreeMap;
use std::io;
use std::io::Write;
use std::result::Result;
use std::time::Duration;

use minicbor::encode;
use tokio::io::split;
use tokio::io::AsyncReadExt;
use tokio::select;

use limero::*;

use minicbor::display;


struct MockValue {
    topic : &'static str,
    bytes : Vec<u8>
}



pub struct MockPubSubActor {
    cmds: CmdQueue<PubSubCmd>,
    event_handlers: EventHandlers<PubSubEvent>,
    timers : Timers,
    mock_values : Vec<MockValue>,
}

impl MockPubSubActor {
    pub fn new() -> Self {
        let mut mock_values = Vec::new();
        mock_values.push(MockValue { topic : "test/a",bytes : minicbor::to_vec(1u64).unwrap()});
        mock_values.push(MockValue { topic : "test/b",bytes : minicbor::to_vec(0.01_f32).unwrap()});

        MockPubSubActor {
            cmds: CmdQueue::new(100),
            event_handlers: EventHandlers::new(),
            timers : Timers::new(),
            mock_values,
        }
    }

    pub fn sender(&self) -> Sender<PubSubCmd> {
        self.cmds.sender()
    }
}

impl Actor<PubSubCmd, PubSubEvent> for MockPubSubActor {
    async fn run(&mut self) {
        self.timers.add_timer(Timer::new_repeater(0, Duration::from_millis(1000)));

        loop {
            select! {
                cmd = self.cmds.next() => {
                    match cmd {
                        _ => {
                            info!("PubSubActor::run() None");
                        }
                    }
                },
                idx = self.timers.alarm() => {
                    match idx {
                        0 => {
                            info!("PubSubActor::run() Timer 0");
                            for mock_value in self.mock_values.iter() {
                                self.event_handlers.handle(&PubSubEvent::Publish {
                                    topic: mock_value.topic.to_string(),
                                    payload: mock_value.bytes.clone(),
                                });
                            }
                            self.event_handlers.handle(&PubSubEvent::Publish {
                                topic: "test/c".to_string(),
                                payload: minicbor::to_vec(rand::random::<f32>()).unwrap(),
                            });
                        }
                        _ => {
                            info!("PubSubActor::run() Timer {}", idx);
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


