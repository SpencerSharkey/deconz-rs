use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot};

use super::task::{SubscribeRequest, TaskMessage};
use crate::deconz::protocol::{
    aps::ReadReceivedDataResponse, DeconzCommand, DeconzCommandResponse,
};

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("error communicating with the task")]
    TaskFailure,
}

/// The DeconzClientHandle has methods for interacting with the Deconz client task.
#[derive(Clone)]
pub struct DeconzClientHandle {
    task_tx: mpsc::UnboundedSender<TaskMessage>,
}

impl DeconzClientHandle {
    /// Used by DeconzClient to construct a new Handle.
    pub(super) fn new(task_tx: mpsc::UnboundedSender<TaskMessage>) -> Self {
        Self { task_tx }
    }

    pub async fn send_command<T>(&mut self, outgoing_command: T) -> Result<T::Response, HandleError>
    where
        T: DeconzCommand,
    {
        let (tx, rx) = oneshot::channel();
        let response_parser = move |frame| {
            let (response, device_state) = T::Response::from_frame(frame);
            tx.send(response).ok();
            device_state
        };
        let task_message = TaskMessage::CommandRequest {
            command_request: Box::new(outgoing_command.into_request()),
            response_parser: Box::new(response_parser),
        };

        self.task_tx
            .send(task_message)
            .map_err(|_| HandleError::TaskFailure)?;

        rx.await.map_err(|_| HandleError::TaskFailure)
    }

    pub async fn subscribe_aps_data_indication(
        &mut self,
    ) -> Result<broadcast::Receiver<ReadReceivedDataResponse>, HandleError> {
        let (tx, rx) = oneshot::channel();

        let task_message = TaskMessage::SubscribeRequest(SubscribeRequest::ApsDataIndication(tx));
        self.task_tx
            .send(task_message)
            .map_err(|_| HandleError::TaskFailure)?;

        rx.await.map_err(|_| HandleError::TaskFailure)
    }
}
