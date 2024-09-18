use async_trait::async_trait;
use ravalink_interconnect::protocol::{Command, Message, Request};
use std::time::Duration;
use crate::background::connector::BoilerplateParseIPCError;
use crate::background::processor::IPCData;
use crate::PlayerObject;
use snafu::prelude::*;
use tokio::sync::broadcast::error::SendError;
use crate::helpers::get_timestamp;

#[derive(Debug, Snafu)]
pub enum TrackActionError {
    #[snafu(display("Failed to send IPC request to Background thread"))]
    FailedToSendIPCRequest { source: SendError<IPCData> },
    #[snafu(display("Did not receive metadata result within timeout time-frame"))]
    TimedOutWaitingForMetadataResult { source: BoilerplateParseIPCError },
}

#[async_trait]
pub trait TrackManager {
    async fn set_volume(&self, playback_volume: f32) -> Result<(), TrackActionError>;
    async fn loop_song(&self) -> Result<(), TrackActionError>;
    async fn seek(&self, position: Duration) -> Result<(), TrackActionError>;
    async fn resume(&self) -> Result<(), TrackActionError>;
    async fn pause(&self) -> Result<(), TrackActionError>;

}

#[async_trait]
impl TrackManager for PlayerObject {
    async fn set_volume(&self, playback_volume: f32) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::SetVolume { volume: playback_volume },
                    guild_id: self.guild_id.clone(),
                    voice_channel_id: None,
                    timestamp: get_timestamp(),
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;
        Ok(())
    }

    async fn loop_song(&self) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::Loop,
                    guild_id: self.guild_id.clone(),
                    voice_channel_id: None,
                    timestamp: get_timestamp(),
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }

    async fn seek(&self, position: Duration) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::SeekToPosition { position: position.as_millis() as u64 },
                    guild_id: self.guild_id.clone(),
                    voice_channel_id: None,
                    timestamp: get_timestamp(),
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }

    async fn resume(&self) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::Resume,
                    guild_id: self.guild_id.clone(),
                    voice_channel_id: None,
                    timestamp: get_timestamp()
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }

    async fn pause(&self) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::Pause,
                    guild_id: self.guild_id.clone(),
                    voice_channel_id: None,
                    timestamp: get_timestamp(),
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }
}
