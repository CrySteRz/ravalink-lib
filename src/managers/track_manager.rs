use async_trait::async_trait;
use ravalink_interconnect::protocol::{Command, Message};
use std::time::Duration;
use crate::PlayerObject;
use crate::PlayerError;

#[async_trait]
pub trait TrackManager {
    async fn set_volume(&self, playback_volume: f32) -> Result<Message, PlayerError>;
    async fn loop_song(&self) -> Result<Message, PlayerError>;
    async fn seek(&self, position: Duration) -> Result<Message, PlayerError>;
    async fn resume(&self) -> Result<Message, PlayerError>;
    async fn pause(&self) -> Result<Message, PlayerError>;
}

#[async_trait]
impl TrackManager for PlayerObject {
    async fn set_volume(&self, playback_volume: f32) -> Result<Message, PlayerError> {
        self.send_request_with_response(
            Command::SetVolume { volume: playback_volume },
            None,
        ).await
    }

    async fn loop_song(&self) -> Result<Message, PlayerError> {
        self.send_request_with_response(
            Command::Loop,
            None,
        ).await
    }

    async fn seek(&self, position: Duration) -> Result<Message, PlayerError> {
        self.send_request_with_response(
            Command::SeekToPosition { position: position.as_millis() as u64 },
            None,
        ).await
    }

    async fn resume(&self) -> Result<Message, PlayerError> {
        self.send_request_with_response(
            Command::Resume,
            None,
        ).await
    }

    async fn pause(&self) -> Result<Message, PlayerError> {
        self.send_request_with_response(
            Command::Pause,
            None,
        ).await
    }
}