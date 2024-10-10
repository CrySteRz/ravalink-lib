use crate::background::connector::send_message;
use crate::RavalinkConfig;
use ravalink_interconnect::protocol::Message;
use log::{debug, error};
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::FutureProducer;
use rdkafka::Message as KafkaMessage;
use std::{collections::HashMap, num::NonZero};
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use futures::stream::StreamExt;


#[derive(Clone, Debug)]
pub struct RavalinkMessage {
    pub message: Message,
    pub guild_id: Option<NonZero<u64>>,
    pub response_tx: Option<Arc<Sender<RavalinkIPC>>>
}

#[derive(Clone, Debug)]
pub enum RavalinkIPC {
    Message(RavalinkMessage),
}

impl RavalinkIPC {
    pub fn create_bot_ping_request(
        message: Message,
        sender: Arc<Sender<RavalinkIPC>>,
    ) -> RavalinkIPC {
        RavalinkIPC::Message(RavalinkMessage {
            message,
            guild_id: None,
            response_tx: Some(sender),
        })
    }
    
    pub fn create_bot_request(
        message: Message,
        sender: Arc<Sender<RavalinkIPC>>,
        guild_id: NonZero<u64>,
    ) -> RavalinkIPC {
        RavalinkIPC::Message(RavalinkMessage {
            message,
            guild_id: Some(guild_id),
            response_tx: Some(sender),
        })
    }
    
    pub fn create_server_response(message: Message) -> RavalinkIPC {
        RavalinkIPC::Message(RavalinkMessage {
            message: message.clone(),
            guild_id: message.get_guild_id(),
            response_tx: None,
        })
    }
}

pub async fn parse_message(
    message: Message,
    guild_id_to_tx: &mut HashMap<NonZero<u64>, Arc<Sender<RavalinkIPC>>>,
    global_tx: &mut Sender<RavalinkIPC>,
) {
    match message {
        Message::Ping { .. } | Message::Request { .. } => {
            return;
        }
        _ => {}
    }

    let guild_id = message.get_guild_id();
    let message_clone = message.clone();

    if let Some(guild_id) = guild_id {
        if let Some(tx) = guild_id_to_tx.get(&guild_id) {
            if tx.send(RavalinkIPC::create_server_response(message_clone)).is_err() {
                error!("Failed to send message to specific guild {}", guild_id);
            }
        }
    } else {
        if global_tx.send(RavalinkIPC::create_server_response(message_clone)).is_err() {
            error!("Failed to send global message");
        }
    }
}
trait GuildIdProvider {
    fn get_guild_id(&self) -> Option<NonZero<u64>>;
}

impl GuildIdProvider for Message {
    fn get_guild_id(&self) -> Option<NonZero<u64>> {
        match self {
            Message::Response(r) => Some(r.guild_id.clone()),
            Message::Request(r) => Some(r.guild_id.clone()),
            Message::Event(e) => Some(e.guild_id.clone()),
            _ => None,
        }
    }
}

pub async fn init_processor(
    mut rx: Receiver<RavalinkIPC>,
    mut global_tx: Sender<RavalinkIPC>,
    consumer: Arc<StreamConsumer>,
    mut producer: FutureProducer,
    config: RavalinkConfig,
) {
    let mut guild_id_to_tx: HashMap<NonZero<u64>, Arc<Sender<RavalinkIPC>>> = HashMap::new();
    let mut kafka_stream = consumer.stream();

    loop {
        tokio::select! {
            ipc_message = rx.recv() => {
                match ipc_message {
                    Ok(RavalinkIPC::Message(m)) => {
                        if let Message::Pong { id: _ } = &m.message {
                            debug!("Skipping Pong message, not sending to Kafka.");
                        } else {
                            if let Some(guild_id) = m.guild_id {
                                guild_id_to_tx.insert(guild_id, m.response_tx.clone().unwrap());
                            }
                            send_message(&m.message, &config.kafka_topic, &mut producer).await;
                        }
                    },
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        error!("IPC channel closed, stopping processor.");
                        break;
                    },
                    Err(e) => {
                        error!("Error receiving IPC message: {}", e);
                    }
                }
            },

            kafka_message = kafka_stream.next() => {
                match kafka_message {
                    Some(Ok(m)) => {
                        let payload = m.payload();
                        if payload.is_none() {
                            error!("Received no payload!");
                            continue;
                        }

                        match serde_json::from_slice::<Message>(payload.unwrap()) {
                            Ok(parsed_message) => {
                                parse_message(parsed_message, &mut guild_id_to_tx, &mut global_tx).await;
                            },
                            Err(e) => error!("Failed to parse message: {}", e),
                        }
                    },
                    Some(Err(e)) => error!("Failed to consume message: {}", e),
                    None => debug!("No Kafka message received."),
                }
            }
        }
    }
}