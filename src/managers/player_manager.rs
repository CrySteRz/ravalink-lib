use async_trait::async_trait;
use ravalink_interconnect::protocol::{Message, Request, Command};

use crate::background::processor::IPCData;
use crate::PlayerObject;
use snafu::prelude::*;
use tokio::sync::broadcast::error::SendError;
use crate::helpers::get_timestamp;

#[derive(Debug, Snafu)]
pub enum PlayerActionError {
    #[snafu(display("Failed to send IPC request to Background thread"))]
    FailedToSendIPCRequest { source: SendError<IPCData> },
}

#[async_trait]
pub trait Player {
    async fn play(&mut self, url: String, voice_channel_id: String) -> Result<(), PlayerActionError>;
}

#[async_trait]
impl Player for PlayerObject {
    async fn play(&mut self, url: String, voice_channel_id: String) -> Result<(), PlayerActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.clone().read().await.clone().unwrap(),
                    command: Command::Play { url: url.clone() },
                    guild_id: self.guild_id.clone(),
                    voice_channel_id: voice_channel_id.clone(),
                    timestamp: get_timestamp(),
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }
}
