use ravalink_interconnect::protocol::{Command, Message};
use std::num::NonZero;
use crate::{PlayerError, PlayerObject};
use async_trait::async_trait;

#[async_trait]
pub trait ChannelManager {
    async fn connect(
        &mut self,
        voice_channel_id: NonZero<u64>,
    ) -> Result<Message, PlayerError>;
    async fn stop(&self) -> Result<Message, PlayerError>;
}

#[async_trait]
impl ChannelManager for PlayerObject {
    async fn connect(
        &mut self,
        voice_channel_id: NonZero<u64>,
    ) -> Result<Message, PlayerError> {  
        self.send_request_with_response(
            Command::Connect,
            Some(voice_channel_id),
        ).await
    }
    
    async fn stop(&self) -> Result<Message, PlayerError> {
        self.send_request_with_response(
            Command::Stop,
            None,
        ).await
    }
}
