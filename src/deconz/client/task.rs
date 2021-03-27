use std::{convert::Infallible, fmt::Display, io};

use bytes::{Bytes, BytesMut};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_serial::{Serial, SerialPortSettings};
use tracing::info;

use crate::deconz::{
    protocol::{CommandType, DeconzCommandOutgoing, DeconzCommandOutgoingRequest},
    DeconzFrame, DeconzStream,
};

use super::DeconzClientConfig;

pub enum TaskMessage {
    CommandRequest {
        command: DeconzFrame<BytesMut>,
        response_parser: Box<dyn FnOnce(DeconzFrame<Bytes>)>,
    },
}

impl Display for TaskMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskMessage::CommandRequest { .. } => f.write_str("CommandRequest"),
        }
    }
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error(transparent)]
    IoError(#[from] io::Error),
}

/// The main loop task has a few responsibilities:
/// - Initiating a deCONZ device communications stream.
/// - Reacting to and/or responding to TaskMessages sent from client handles.
/// - Sending and receiving messages to the deCONZ device stream.
pub struct DeconzTask {
    config: DeconzClientConfig,
    task_rx: mpsc::UnboundedReceiver<TaskMessage>,
}

impl DeconzTask {
    pub fn new(config: DeconzClientConfig, task_rx: mpsc::UnboundedReceiver<TaskMessage>) -> Self {
        Self { config, task_rx }
    }

    /// Consumes the task, starting the main loop.
    pub async fn run(mut self) -> Result<(), TaskError> {
        let serial_stream = self.connect_serial()?;
        let mut deconz_stream = DeconzStream::new(serial_stream);

        loop {
            tokio::select! {
                Some(Ok(cmd)) = deconz_stream.next_command() => {
                    info!("cmd {:?}", cmd);
                }
                Some(msg) = self.task_rx.recv() => {
                    info!("msg {:?}", msg);
                }
            }
        }
    }

    /*
    let mut boxes: Vec<Box<dyn FnOnce(Vec<u8>)>> = vec![];
        boxes.push(Box::new(move |bytes| {
            let response = X::B::parse_from_bytes(bytes);
            tx.send(response).ok();
        }));
    */

    fn connect_serial(&self) -> Result<Serial, TaskError> {
        tokio_serial::Serial::from_path(
            self.config.device_path.clone(),
            &SerialPortSettings {
                baud_rate: 38400,
                ..Default::default()
            },
        )
        .map_err(|e| e.into())
    }
}
