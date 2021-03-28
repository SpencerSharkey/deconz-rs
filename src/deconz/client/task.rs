use std::fmt::Debug;
use std::{collections::HashMap, fmt::Display, io};

use bytes::Bytes;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_serial::{Serial, SerialPortSettings};
use tracing::info;

use crate::deconz::{
    frame::OutgoingPacket, protocol::DeconzCommandRequest, DeconzFrame, DeconzStream,
};

use super::DeconzClientConfig;

pub enum TaskMessage {
    CommandRequest {
        command_outgoing: Box<dyn DeconzCommandRequest>,
        response_parser: Box<dyn FnOnce(DeconzFrame<Bytes>) + Send>,
    },
}

impl Display for TaskMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskMessage::CommandRequest { .. } => f.write_str("CommandRequest"),
        }
    }
}

impl Debug for TaskMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskMessage::CommandRequest {
                command_outgoing,
                response_parser: _,
            } => f
                .debug_struct("TaskMessage::CommandRequest")
                .field("command", command_outgoing)
                .field("response_parser", &"...")
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
    next_sequence_number: u8,
    in_flight_commands: HashMap<u8, InFlightCommand>,
}

impl DeconzTask {
    pub fn new(config: DeconzClientConfig, task_rx: mpsc::UnboundedReceiver<TaskMessage>) -> Self {
        Self {
            config,
            task_rx,
            next_sequence_number: 0,
            in_flight_commands: Default::default(),
        }
    }

    /// Consumes the task, starting the main loop.
    pub async fn run(mut self) -> Result<(), TaskError> {
        let serial_stream = self.connect_serial()?;
        let mut deconz_stream = DeconzStream::new(serial_stream);

        loop {
            tokio::select! {
                Some(Ok(frame)) = deconz_stream.next_frame() => {
                    self.handle_deconz_frame(frame).await;

                }
                Some(task_kmessage) = self.task_rx.recv() => {
                    self.handle_task_message(task_kmessage, &mut deconz_stream).await?;
                }
            }
        }
    }

    fn connect_serial(&self) -> Result<Serial, TaskError> {
        Ok(tokio_serial::Serial::from_path(
            self.config.device_path.clone(),
            &SerialPortSettings {
                baud_rate: 38400,
                ..Default::default()
            },
        )?)
    }

    async fn handle_deconz_frame(&mut self, incoming_frame: DeconzFrame<Bytes>) {
        info!("incoming deconz frame {:?}", incoming_frame);

        if let Some(in_flight_command) = self
            .in_flight_commands
            .remove(&incoming_frame.sequence_number())
        {
            (in_flight_command.response_parser)(incoming_frame);
        } else {
            info!("frame has no in-flight command handler registered, dropping!");
        }
    }

    async fn handle_task_message(
        &mut self,
        task_message: TaskMessage,
        deconz_stream: &mut DeconzStream<Serial>,
    ) -> Result<(), TaskError> {
        info!("incoming task message {:?}", task_message);

        match task_message {
            TaskMessage::CommandRequest {
                command_outgoing,
                response_parser,
            } => {
                let sequence_number = self.next_sequence_number();
                let command_id = command_outgoing.command_id();

                // todo: handle sequence id exhaustion (and queueing logic...)
                self.in_flight_commands
                    .insert(sequence_number, InFlightCommand { response_parser });

                let frame = command_outgoing.into_frame(sequence_number);
                deconz_stream.write_frame(frame).await.unwrap(); // todo: Error handling!
            }
        }

        Ok(())
    }

    fn next_sequence_number(&mut self) -> u8 {
        let sequence_number = self.next_sequence_number;
        self.next_sequence_number = self.next_sequence_number.wrapping_add(1);
        sequence_number
    }
}

struct InFlightCommand {
    response_parser: Box<dyn FnOnce(DeconzFrame<Bytes>) + Send>,
}
