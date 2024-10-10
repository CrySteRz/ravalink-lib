use std::sync::Arc;
use std::time::Duration;
use nanoid::nanoid;
use ravalink_interconnect::protocol::Message;
use async_trait::async_trait;
use tokio::sync::broadcast::error::SendError;
use tokio::time::timeout;
use crate::RavalinkIPC;
use crate::TX;

#[derive(Debug)]
pub enum DefaultError {
    InitializationError,
    FailedToReceiveIPCResponse,
    FailedToSendIPCRequest { source: SendError<RavalinkIPC> },
    NoGlobalTX,
}

//CHECK IF ITS WORKING, implement it in the bot and implement the wait for Pong
pub struct DefaultObject;

#[async_trait]
pub trait DefaultManager {
    async fn ping(&self) -> Result<Message, DefaultError>;
}

#[async_trait]
impl DefaultManager for DefaultObject {
    async fn ping(&self) -> Result<Message, DefaultError> {

        let ping_id = nanoid!();

        let tx_lock = TX.lock().await;
        let sender = if let Some(ref tx) = *tx_lock {
            Arc::new(tx.clone())
        } else {
            return Err(DefaultError::NoGlobalTX);
        };

        let ping = RavalinkIPC::create_bot_ping_request(Message::Ping { id: ping_id.clone() }, sender.clone());
        sender.send(ping).map_err(|source| DefaultError::FailedToSendIPCRequest { source })?;

        let mut rx = sender.subscribe();
        let timeout_duration = Duration::from_secs(5);
        match timeout(timeout_duration, async {
            loop {
                match rx.recv().await {
  
                    Ok(RavalinkIPC::Message(response)) => {
                        if let Message::Pong { id } = &response.message {
                            if *id == ping_id {
                                return Ok(response.message);
                            }
                        }
                    }

                    Err(_) => {
                        return Err(DefaultError::FailedToReceiveIPCResponse);
                    }
                }
            }
        }).await {
            Ok(result) => result,
            Err(_) => Err(DefaultError::FailedToReceiveIPCResponse),
        }
    }
}