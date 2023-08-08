pub mod mqtt_bridge;
pub mod redis_bridge;


#[derive(Debug, Clone)]
pub enum PubSubEvent {
    Publish{ topic: String, message: String},
}
#[derive(Debug, Clone)]
pub enum PubSubCmd {
    Subscribe{ pattern: String },
    Unsubscribe{ pattern: String },
    Publish{ topic: String, message: String },
}
