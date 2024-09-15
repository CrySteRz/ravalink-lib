use async_trait::async_trait;
use chrono::Utc;
use ravalink_interconnect::protocol::{Command, Message, Request};
use std::time::Duration;
use nanoid::nanoid;
use crate::background::connector::BoilerplateParseIPCError;
use crate::background::processor::IPCData;
use crate::PlayerObject;
use snafu::prelude::*;
use tokio::sync::broadcast::error::SendError;

#[derive(Debug, Snafu)]
pub enum TrackActionError {
    #[snafu(display("Failed to send IPC request to Background thread"))]
    FailedToSendIPCRequest { source: SendError<IPCData> },
    #[snafu(display("Did not receive metadata result within timeout time-frame"))]
    TimedOutWaitingForMetadataResult { source: BoilerplateParseIPCError },
}

#[async_trait]
/// Provides functionality that can be used once you start playing a track such as: looping, pausing, and resuming.
pub trait TrackManager {
    /// Set playback volume
    async fn set_playback_volume(&self, playback_volume: f32) -> Result<(), TrackActionError>;
    /// Loop forever
    async fn loop_indefinitely(&self) -> Result<(), TrackActionError>;
    /// Seek to a specific position in the track
    async fn seek_to_position(&self, position: Duration) -> Result<(), TrackActionError>;
    /// Resume playback
    async fn resume_playback(&self) -> Result<(), TrackActionError>;
    /// Pause playback
    async fn pause_playback(&self) -> Result<(), TrackActionError>;

}

#[async_trait]
impl TrackManager for PlayerObject {
    async fn set_playback_volume(&self, playback_volume: f32) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::SetVolume { volume: playback_volume },
                    guild_id: self.guild_id.clone(),
                    worker_id: self.worker_id.read().await.clone().unwrap(),
                    voice_channel_id: None,
                    timestamp: Utc::now().timestamp_millis().abs() as u64,
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;
        Ok(())
    }

    async fn loop_indefinitely(&self) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::Loop { enabled: true },
                    guild_id: self.guild_id.clone(),
                    worker_id: self.worker_id.read().await.clone().unwrap(),
                    voice_channel_id: None,
                    timestamp: Utc::now().timestamp_millis().abs() as u64,
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }

    async fn seek_to_position(&self, position: Duration) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::SeekToPosition { position: position.as_millis() as u64 },
                    guild_id: self.guild_id.clone(),
                    worker_id: self.worker_id.read().await.clone().unwrap(),
                    voice_channel_id: None,
                    timestamp: Utc::now().timestamp_millis().abs() as u64,
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }

    async fn resume_playback(&self) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::Resume,
                    guild_id: self.guild_id.clone(),
                    worker_id: self.worker_id.read().await.clone().unwrap(),
                    voice_channel_id: None,
                    timestamp: Utc::now().timestamp_millis().abs() as u64,
                }),
                self.tx.clone(),
                self.guild_id.clone(),
            ))
            .context(FailedToSendIPCRequestSnafu)?;

        Ok(())
    }

    async fn pause_playback(&self) -> Result<(), TrackActionError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::Pause,
                    guild_id: self.guild_id.clone(),
                    worker_id: self.worker_id.read().await.clone().unwrap(),
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
