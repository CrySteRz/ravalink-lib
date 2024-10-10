
use crate::background::processor::{init_processor, RavalinkIPC};
use lazy_static::lazy_static;
use log::error;
use ravalink_interconnect::protocol::{Command, Message, Request};
use rdkafka::producer::FutureProducer;
use snafu::Snafu;
use std::collections::HashMap;
use std::num::NonZero;
use std::sync::Arc;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex, RwLock};
use nanoid::nanoid;
use crate::helpers::get_timestamp;
use snafu::ResultExt;
pub mod managers;
pub mod background;
pub mod handlers;

mod helpers;
pub mod serenity;

use crate::background::connector::{initialize_client, initialize_producer};
use rdkafka::consumer::StreamConsumer;

lazy_static! {
    pub(crate) static ref PRODUCER: Mutex<Option<FutureProducer>> = Mutex::new(None);
    pub(crate) static ref CONSUMER: Mutex<Option<StreamConsumer>> = Mutex::new(None);
    pub(crate) static ref TX: Mutex<Option<Sender<RavalinkIPC>>> = Mutex::new(None);
    pub(crate) static ref RX: Mutex<Option<Receiver<RavalinkIPC>>> = Mutex::new(None);
}

#[derive(Debug, Snafu)]
pub enum PlayerError {
    InitializationError,
    FailedToReceiveIPCResponse,
    FailedToSendIPCRequest { source: SendError<RavalinkIPC> },
}

pub struct PlayerObject {
    guild_id: NonZero<u64>,
    tx: Arc<Sender<RavalinkIPC>>,
    bg_com_tx: Sender<RavalinkIPC>,
}

impl PlayerObject {
    pub async fn new(guild_id: NonZero<u64>, com_tx: Sender<RavalinkIPC>) -> Result<Self, PlayerError> {
        let (tx, _rx) = broadcast::channel(16);

        let handler = PlayerObject {
            guild_id,
            tx: Arc::new(tx),
            bg_com_tx: com_tx,
        };

        Ok(handler)
    }

    async fn send_request_with_response(
        &self,
        command: Command,
        voice_channel_id: Option<NonZero<u64>>,
    ) -> Result<Message, PlayerError> {
        let job_id = nanoid!();
        let guild_id = self.guild_id.clone();

        self.bg_com_tx
            .send(RavalinkIPC::create_bot_request(
                Message::Request(Request {
                    job_id: job_id.clone(),
                    guild_id: guild_id.clone(),
                    voice_channel_id,
                    command,
                    timestamp: get_timestamp(),
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;
        
        self.wait_for_response(job_id, guild_id).await
    }

    //Probably need to implement a timeout here
    pub async fn wait_for_response(
        &self,
        job_id: String,
        guild_id: NonZero<u64>,
    ) -> Result<Message, PlayerError > {
        let mut rx = self.tx.subscribe();
        loop {
            match rx.recv().await {
                Ok(RavalinkIPC::Message(response)) => {
                    if let Message::Response(res) = &response.message {
                        if res.job_id == job_id && res.guild_id == guild_id {
                            return Ok(response.message.clone());
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive response: {:?}", e);
                    return Err(PlayerError::FailedToReceiveIPCResponse);
                }
            }
        }
    }
}

pub struct Ravalink {
    pub players: Arc<RwLock<HashMap<String, PlayerObject>>>,
    pub tx: Sender<RavalinkIPC>,
    pub rx: Receiver<RavalinkIPC>,
}

#[derive(Clone)]
pub struct SSLConfig {
    pub ssl_key: String,
    pub ssl_ca: String,
    pub ssl_cert: String,
}

#[derive(Clone)]
pub struct SASLConfig {
    pub kafka_username: String,
    pub kafka_password: String,
}

#[derive(Clone)]

pub struct RavalinkConfig {
    pub ssl: Option<SSLConfig>,
    pub sasl: Option<SASLConfig>,
    pub kafka_topic: String,
}


pub async fn init_ravalink(broker: String, config: RavalinkConfig) -> Arc<Mutex<Ravalink>> {
    let consumer = initialize_client(&broker, &config).await;
    let producer = initialize_producer(&broker, &config);


    let (tx, _rx) = broadcast::channel(16);

    {
        let mut tx_lock = TX.lock().await;
        *tx_lock = Some(tx.clone());
    }

    {
        let mut rx_lock = RX.lock().await;
        *rx_lock = Some(tx.subscribe());
    }

    let task_tx = tx.clone();

    tokio::task::spawn(async move {
        init_processor(task_tx.subscribe(), task_tx, consumer.into(), producer, config).await;
    });

    Arc::new(Mutex::new(Ravalink {
        players: Arc::new(RwLock::new(HashMap::new())),
        tx: tx.clone(),
        rx: tx.subscribe(),
    }))
}