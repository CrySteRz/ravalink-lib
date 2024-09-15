use ravalink_interconnect::protocol::{Command, Message, Request};
use std::time::Duration;
use crate::PlayerObject;
use async_trait::async_trait;
use nanoid::nanoid;
use crate::background::connector::{boilerplate_parse_ipc, BoilerplateParseIPCError};
use crate::background::processor::IPCData;
use snafu::prelude::*;
use tokio::sync::broadcast::error::SendError;
use crate::helpers::get_timestamp;

#[derive(Debug, Snafu)]
pub enum CreateJobError {
    #[snafu(display("Did not receive job creation confirmation within time-frame"))]
    TimedOutWaitingForJobCreationConfirmation { source: BoilerplateParseIPCError },
    #[snafu(display("Failed to send internal IPC job creation request"))]
    FailedToSendIPC { source: SendError<IPCData> },
}

#[derive(Debug, Snafu)]
pub enum ChannelManagerError {
    #[snafu(display("Failed to send IPC request to Background thread"))]
    FailedToSendIPCRequest { source: SendError<IPCData> },
}

#[async_trait]
pub trait ChannelManager {
    async fn connect(
        &mut self,
        voice_channel_id: String,
    ) -> Result<(), CreateJobError>;
    async fn exit(&self) -> Result<(), ChannelManagerError>;
}

#[async_trait]
impl ChannelManager for PlayerObject {
    async fn connect(
        &mut self,
        voice_channel_id: String,
    ) -> Result<(), CreateJobError> {
        let guild_id = self.guild_id.clone();

        let tx = self.tx.clone();
        let bg_com = self.bg_com_tx.clone();

        let job_id = self.job_id.clone();

        tokio::spawn(async move {
            bg_com
                .send(IPCData::new_from_main(
                    Message::Request(Request {
                        job_id: nanoid!(),
                        guild_id: guild_id.clone(),
                        voice_channel_id: Some(voice_channel_id.clone()),
                        command: Command::Connect,
                        timestamp: get_timestamp(),
                    }),
                    tx.clone(),
                    guild_id.clone(),
                ))
                .unwrap();
            let mut job_id_a = job_id.write().await;

            boilerplate_parse_ipc(
                |msg| {
                    if let IPCData::FromBackground(bg) = msg {
                        if let Message::Request(q) = bg.message {
                            *job_id_a = Some(q.job_id);
                            return false;
                        }
                    }
                    true
                },
                tx.subscribe(),
                Duration::from_secs(3),
            )
            .await
            .unwrap();

            bg_com
                .send(IPCData::new_from_main(
                    Message::Request(Request {
                        job_id: job_id_a.clone().unwrap(),
                        guild_id: guild_id.clone(),
                        voice_channel_id: Some(voice_channel_id),
                        command: Command::Connect,
                        timestamp: get_timestamp(),

                    }),
                    tx.clone(),
                    guild_id,
                ))
                .context(FailedToSendIPCRequestSnafu)
                .unwrap();
        });

        Ok(())
    }
    async fn exit(&self) -> Result<(), ChannelManagerError> {
        self.bg_com_tx
            .send(IPCData::new_from_main(
                Message::Request(Request {
                    job_id: self.job_id.read().await.clone().unwrap(),
                    command: Command::Stop,
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
}
