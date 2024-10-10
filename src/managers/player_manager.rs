use async_trait::async_trait;
use ravalink_interconnect::protocol::{Message, Command};
use crate::{PlayerError, PlayerObject};

#[async_trait]
pub trait Player {
    async fn play(&mut self, url: String) -> Result<Message, PlayerError>;
}

#[async_trait]
impl Player for PlayerObject {
    async fn play(&mut self, url: String) -> Result<Message, PlayerError> {
        self.send_request_with_response(
            Command::Play { url },
            None,
        ).await
    }
}
