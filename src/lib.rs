//! Ravalink is a client-library for Hearth that makes it easy to use Hearth with Rust.
//! See Examples in the Github repo [here](https://github.com/Hearth-Industries/Ravalink/tree/main/examples)

use crate::actions::channel_manager::CreateJobError;
use crate::background::processor::{init_processor, IPCData};
use crate::constants::{EXPIRATION_LAGGED_BY_1, EXPIRATION_LAGGED_BY_2, EXPIRATION_LAGGED_BY_4};
use ravalink_interconnect::protocol::Message;
use lazy_static::lazy_static;
use log::{error, info};
use rdkafka::producer::FutureProducer;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time;

pub mod actions;
pub mod background;
pub(crate) mod constants;
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

/// Represents an instance in a voice channel
pub struct PlayerObject {
    worker_id: Arc<RwLock<Option<String>>>,
    job_id: Arc<RwLock<Option<String>>>,
    guild_id: String,
    tx: Arc<Sender<IPCData>>,
    bg_com_tx: Sender<IPCData>,
}

impl PlayerObject {
    /// Creates a new Player Object that can then be joined to channel and used to playback audio
    pub async fn new(guild_id: String, com_tx: Sender<IPCData>) -> Result<Self, CreateJobError> {
        let (tx, _rx) = broadcast::channel(16);

        let handler = PlayerObject {
            worker_id: Arc::new(RwLock::new(None)),
            job_id: Arc::new(RwLock::new(None)),
            guild_id,
            tx: Arc::new(tx),
            bg_com_tx: com_tx,
        };

        Ok(handler)
    }
}

/// Stores Ravalink instance
pub struct Ravalink {
    pub players: Arc<RwLock<HashMap<String, PlayerObject>>>, // Guild ID to PlayerObject
    pub tx: Sender<IPCData>,
    pub rx: Receiver<IPCData>,
}

impl Ravalink {
    // fn start_global_checker(&mut self) {
    //     info!("Started global data checker!");
    //     let mut rxx = self.tx.subscribe();
    //     let t_players = self.players.clone();
    //     let mut tick_adjustments = 0;
    //     tokio::task::spawn(async move {
    //         let mut interval = time::interval(Duration::from_secs(1));

    //         loop {
    //             interval.tick().await;
    //             let catch = rxx.try_recv();
    //             match catch {
    //                 Ok(c) => {
    //                     if let IPCData::FromBackground(bg) = c {
    //                         match bg.message {
    //                             Message::ExternalJobExpired(je) => {
    //                                 info!("Job Expired: {}", je.job_id);
    //                                 let mut t_p_write = t_players.write().await;
    //                                 t_p_write.remove(&je.guild_id);
    //                             }
    //                             Message::WorkerShutdownAlert(shutdown_alert) => {
    //                                 info!("Worker shutdown! Cancelling Players!");
    //                                 let mut t_p_write = t_players.write().await;
    //                                 for job_id in shutdown_alert.affected_guild_ids {
    //                                     t_p_write.remove(&job_id);
    //                                 }
    //                             }
    //                             _ => {}
    //                         }
    //                     }
    //                 }
    //                 Err(e) => match e {
    //                     TryRecvError::Empty => {}
    //                     TryRecvError::Lagged(count) => {
    //                         error!("Expiration Checker - Lagged by: {}", count);
    //                         let mut tick_adjustment_ratio: f32 = tick_adjustments as f32;
    //                         if count >= 4 {
    //                             tick_adjustment_ratio =
    //                                 (tick_adjustment_ratio * 1.2).clamp(1.0, 3.0);
    //                             interval = time::interval(Duration::from_millis(
    //                                 (EXPIRATION_LAGGED_BY_4 / tick_adjustment_ratio) as u64,
    //                             ));
    //                             tick_adjustments += 1;
    //                         } else if count >= 2 {
    //                             tick_adjustment_ratio =
    //                                 (tick_adjustment_ratio * 1.2).clamp(1.0, 3.0);
    //                             interval = time::interval(Duration::from_millis(
    //                                 (EXPIRATION_LAGGED_BY_2 / tick_adjustment_ratio) as u64,
    //                             ));
    //                             tick_adjustments += 1;
    //                         } else if count >= 1 {
    //                             tick_adjustment_ratio =
    //                                 (tick_adjustment_ratio * 1.2).clamp(1.0, 3.0);
    //                             interval = time::interval(Duration::from_millis(
    //                                 (EXPIRATION_LAGGED_BY_1 / tick_adjustment_ratio) as u64,
    //                             ));
    //                             tick_adjustments += 1;
    //                         }
    //                         info!("Performed auto adjustment to prevent lag new interval: {} milliseconds",interval.period().as_millis());
    //                     }
    //                     _ => {
    //                         error!("Failed to receive expiration check with error: {}", e);
    //                     }
    //                 },
    //             }
    //         }
    //     });
    // }
}

#[derive(Clone)]
/// Stores SSL Config for Kafka
pub struct SSLConfig {
    /// Path to the SSL key file
    pub ssl_key: String,
    /// Path to the SSL CA file
    pub ssl_ca: String,
    /// Path to the SSL cert file
    pub ssl_cert: String,
}

#[derive(Clone)]
pub struct SASLConfig {
    /// Kafka Username
    pub kafka_username: String,
    /// Kafka Password
    pub kafka_password: String,
}

#[derive(Clone)]

pub struct RavalinkConfig {
    pub ssl: Option<SSLConfig>,
    pub sasl: Option<SASLConfig>,
    pub kafka_topic: String,
}

/// Initializes Ravalink Instance
pub async fn init_ravalink(broker: String, config: RavalinkConfig) -> Arc<Mutex<Ravalink>> {
    // This isn't great we should really switch to rdkafka instead of kafka

    let consumer = initialize_client(&broker, &config).await;

    let producer = initialize_producer(&broker, &config);

    let (tx, rx) = broadcast::channel(16);

    let global_rx = tx.subscribe();
    let sub_tx = tx.clone();

    tokio::task::spawn(async move {
        init_processor(rx, sub_tx, consumer, producer, config).await;
    });

    let mut c_instance = Ravalink {
        players: Arc::new(RwLock::new(HashMap::new())),
        tx,
        rx: global_rx,
    };

    // c_instance.start_global_checker(); // Start checking for expired jobs

    Arc::new(Mutex::new(c_instance))
}
