use crate::managers::channel_manager::CreateJobError;
use crate::background::processor::{init_processor, IPCData};
use lazy_static::lazy_static;
use rdkafka::producer::FutureProducer;
use std::collections::HashMap;
use std::num::NonZero;
use std::sync::Arc;

use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex, RwLock};


pub mod managers;
pub mod background;

mod helpers;
pub mod serenity;

use crate::background::connector::{initialize_client, initialize_producer};
use rdkafka::consumer::StreamConsumer;

lazy_static! {
    pub(crate) static ref PRODUCER: Mutex<Option<FutureProducer>> = Mutex::new(None);
    pub(crate) static ref CONSUMER: Mutex<Option<StreamConsumer>> = Mutex::new(None);
    pub(crate) static ref TX: Mutex<Option<Sender<String>>> = Mutex::new(None);
    pub(crate) static ref RX: Mutex<Option<Receiver<String>>> = Mutex::new(None);
}

pub struct PlayerObject {
    job_id: Arc<RwLock<Option<String>>>,
    guild_id: NonZero<u64>,
    tx: Arc<Sender<IPCData>>,
    bg_com_tx: Sender<IPCData>,
}

impl PlayerObject {
    pub async fn new(guild_id: NonZero<u64>, com_tx: Sender<IPCData>) -> Result<Self, CreateJobError> {
        let (tx, _rx) = broadcast::channel(16);

        let handler = PlayerObject {
            job_id: Arc::new(RwLock::new(None)),
            guild_id,
            tx: Arc::new(tx),
            bg_com_tx: com_tx,
        };

        Ok(handler)
    }
}

pub struct Ravalink {
    pub players: Arc<RwLock<HashMap<String, PlayerObject>>>,
    pub tx: Sender<IPCData>,
    pub rx: Receiver<IPCData>,
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

    let (tx, rx) = broadcast::channel(16);

    let global_rx = tx.subscribe();
    let sub_tx = tx.clone();

    tokio::task::spawn(async move {
        init_processor(rx, sub_tx, consumer, producer, config).await;
    });

    let c_instance = Ravalink {
        players: Arc::new(RwLock::new(HashMap::new())),
        tx,
        rx: global_rx,
    };

    Arc::new(Mutex::new(c_instance))
}
