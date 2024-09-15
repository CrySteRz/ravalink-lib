use crate::background::connector::send_message;
use crate::RavalinkConfig;
use ravalink_interconnect::protocol::Message;
use log::{debug, error};
use rdkafka::consumer::BaseConsumer;
use rdkafka::producer::FutureProducer;
use rdkafka::Message as KafkaMessage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Clone, Debug)]
pub struct FromBackgroundData {
    pub message: Message,
}

#[derive(Clone, Debug)]
pub struct FromMainData {
    pub message: Message,
    pub response_tx: Arc<Sender<IPCData>>,
    pub guild_id: String,
}

#[derive(Clone, Debug)]
pub enum IPCData {
    FromBackground(FromBackgroundData),
    FromMain(FromMainData),
}

impl IPCData {
    pub fn new_from_main(
        message: Message,
        sender: Arc<Sender<IPCData>>,
        guild_id: String,
    ) -> IPCData {
        IPCData::FromMain(FromMainData {
            message,
            response_tx: sender,
            guild_id,
        })
    }
    pub fn new_from_background(message: Message) -> IPCData {
        IPCData::FromBackground(FromBackgroundData { message })
    }
}

pub async fn parse_message(
    message: Message,
    guild_id_to_tx: &mut HashMap<String, Arc<Sender<IPCData>>>,
    global_tx: &mut Sender<IPCData>,
) {
    if let Some(guild_id) = message.get_guild_id() {
        let message_clone = message.clone();
        if let Some(tx) = guild_id_to_tx.get(&guild_id) {
            if tx.send(IPCData::new_from_background(message_clone)).is_err() {
                error!("Failed to send message to specific guild {}", guild_id);
            }
        } else {
            if global_tx.send(IPCData::new_from_background(message_clone)).is_err() {
                error!("Failed to send message globally");
            }
        }
    }
}

trait GuildIdProvider {
    fn get_guild_id(&self) -> Option<String>;
}

impl GuildIdProvider for Message {
    fn get_guild_id(&self) -> Option<String> {
        match self {
            Message::Response(r) => Some(r.guild_id.clone()),
            Message::Request(r) => Some(r.guild_id.clone()),
            Message::Event(e) => Some(e.guild_id.clone()),
            _ => None,
        }
    }
}

pub async fn init_processor(
    mut rx: Receiver<IPCData>,
    mut global_tx: Sender<IPCData>,
    consumer: BaseConsumer,
    mut producer: FutureProducer,
    config: RavalinkConfig,
) {
    let mut guild_id_to_tx: HashMap<String, Arc<Sender<IPCData>>> = HashMap::new();
    loop {
        let mss = consumer.poll(Duration::from_millis(25));
        if let Some(p) = mss {
            match p {
                Ok(m) => {
                    let payload = m.payload();

                    match payload {
                        Some(payload) => {
                            let parsed_message: Result<Message, serde_json::Error> =
                                serde_json::from_slice(payload);

                            match parsed_message {
                                Ok(m) => {
                                    parse_message(m, &mut guild_id_to_tx, &mut global_tx).await;
                                }
                                Err(e) => error!("{}", e),
                            }
                        }
                        None => {
                            error!("Received No Payload!");
                        }
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
        let rx_data = rx.try_recv();
        match rx_data {
            Ok(d) => {
                if let IPCData::FromMain(m) = d {
                    guild_id_to_tx.insert(m.guild_id, m.response_tx);
                    send_message(&m.message, &config.kafka_topic, &mut producer).await;
                }
            }
            Err(e) => {
                if e.to_string() == "channel empty" {
                    debug!("Channel empty!");
                } else {
                    error!("Receive failed with: {}", e);
                }
            }
        }
    }
}
