use std::fmt::Debug;
use std::{fmt::Display, io, time::Duration};

use bytes::Bytes;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_serial::{Serial, SerialPortSettings};
use tracing::info;

use crate::deconz::{
    protocol::{aps::ReadReceivedDataResponse, device::DeviceState, DeconzCommandRequest},
    DeconzFrame, DeconzStream,
};

use super::{
    queue::{DeconzQueue, InFlightCommand},
    DeconzClientConfig,
};

pub enum TaskMessage {
    CommandRequest {
        command_request: Box<dyn DeconzCommandRequest>,
        response_parser: Box<dyn FnOnce(DeconzFrame<Bytes>) -> Option<DeviceState> + Send>,
    },
    SubscribeRequest(SubscribeRequest),
}

#[derive(Debug)]
pub enum SubscribeRequest {
    ApsDataIndication(oneshot::Sender<broadcast::Receiver<ReadReceivedDataResponse>>),
}

impl Display for TaskMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskMessage::CommandRequest { .. } => f.write_str("CommandRequest"),
            TaskMessage::SubscribeRequest(_) => f.write_str("SubscribeRequest"),
        }
    }
}

impl Debug for TaskMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskMessage::CommandRequest {
                command_request: command_outgoing,
                response_parser: _,
            } => f
                .debug_struct("TaskMessage::CommandRequest")
                .field("command", command_outgoing)
                .field("response_parser", &"...")
                .finish(),

            TaskMessage::SubscribeRequest(request) => f
                .debug_struct("TaskMessage::SubscribeRequest")
                .field("request", request)
                .finish(),
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
    queue: DeconzQueue,
}

impl DeconzTask {
    pub fn new(config: DeconzClientConfig, task_rx: mpsc::UnboundedReceiver<TaskMessage>) -> Self {
        Self {
            config,
            task_rx,
            queue: DeconzQueue::new(),
        }
    }

    /// Consumes the task, starting the main loop.
    pub async fn run(mut self) -> Result<(), TaskError> {
        let serial_stream = self.connect_serial()?;
        let mut deconz_stream = DeconzStream::new(serial_stream);

        loop {
            self.queue.try_io(&mut deconz_stream).await;

            tokio::select! {
                Some(Ok(frame)) = deconz_stream.next_frame() => {
                    self.handle_deconz_frame(frame).await;
                }
                Some(task_message) = self.task_rx.recv() => {
                    self.handle_task_message(task_message).await?;
                }
            }
        }
    }

    fn connect_serial(&self) -> Result<Serial, TaskError> {
        Ok(tokio_serial::Serial::from_path(
            self.config.device_path.clone(),
            &SerialPortSettings {
                baud_rate: 38400,
                flow_control: tokio_serial::FlowControl::None,
                timeout: Duration::from_secs(100),
                ..Default::default()
            },
        )?)
    }

    async fn handle_deconz_frame(&mut self, incoming_frame: DeconzFrame<Bytes>) {
        info!("incoming deconz frame {:?}", incoming_frame);
        self.queue.handle_deconz_frame(incoming_frame);
    }

    async fn handle_task_message(&mut self, task_message: TaskMessage) -> Result<(), TaskError> {
        info!("incoming task message {:?}", task_message);

        match task_message {
            TaskMessage::CommandRequest {
                command_request,
                response_parser,
            } => self.queue.enqueue_command(
                command_request,
                InFlightCommand::External { response_parser },
            ),

            TaskMessage::SubscribeRequest(SubscribeRequest::ApsDataIndication(sender)) => {
                sender
                    .send(
                        self.queue
                            .broadcast_channels
                            .subscribe_aps_data_indication(),
                    )
                    .ok();
            }
        }

        Ok(())
    }
}
