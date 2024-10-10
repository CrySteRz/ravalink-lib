use log::error;
use crate::background::processor::RavalinkIPC;
use crate::PlayerObject;
use ravalink_interconnect::protocol::{Event, EventType, Message};

pub trait RavalinkEventHandler {
    fn handle_event(&self, event: Event);
    fn handle_error(&self, error: Event);
}

impl PlayerObject {
    pub async fn register_event_handler(
        &mut self,
        event_handler: impl RavalinkEventHandler + Send + 'static,
    ) {
        let mut t_rx = self.tx.subscribe();
        let guild_id = self.guild_id.clone();

        tokio::spawn(async move {
            while let Ok(RavalinkIPC::Message(ravalink_message)) = t_rx.recv().await {
                let message = ravalink_message.message;

                match message {
                    Message::Event(event) => {
                        if let Some(g_id) = ravalink_message.guild_id {
                            if g_id == guild_id {
                                match event.event_type {
                                    EventType::ErrorOccurred => {
                                        event_handler.handle_error(event);
                                    }
                                    _ => {
                                        event_handler.handle_event(event);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            error!("Receiver closed, no more messages will be processed.");
        });
    }
}
