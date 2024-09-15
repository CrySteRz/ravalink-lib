use async_trait::async_trait;
use chrono::Utc;
use ravalink_interconnect::protocol::{Message, Request, Command};

use crate::background::processor::IPCData;
use crate::PlayerObject;
use nanoid::nanoid;
use snafu::prelude::*;
use tokio::sync::broadcast::error::SendError;

#[derive(Debug, Snafu)]
pub enum PlayerActionError {
    #[snafu(display("Failed to send IPC request to Background thread"))]
    FailedToSendIPCRequest { source: SendError<IPCData> },
}

#[async_trait]
pub trait Player {
    async fn play(&mut self, url: String) -> Result<(), PlayerActionError>;
}

#[async_trait]
impl Player for PlayerObject {
    async fn play(&mut self, url: String) -> Result<(), PlayerActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.clone().read().await.clone().unwrap(),
                    command: Command::Play { url: ("Data".to_string()) },
                    guild_id: self.guild_id.clone(),
                    worker_id: self.worker_id.clone().read().await.clone().unwrap(),
                    voice_channel_id: None,
                    timestamp: Utc::now().timestamp_millis().abs() as u64,
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }
}
