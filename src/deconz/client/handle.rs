use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use super::task::TaskMessage;
use crate::deconz::protocol;
use crate::deconz::protocol::DeconzCommandIncoming;

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

    pub async fn send_command<T: protocol::DeconzCommandOutgoingRequest>(
        &mut self,
        outgoing_command: T,
    ) -> Result<T::Response, HandleError> {
        let (tx, rx) = oneshot::channel();

        let response_parser = move |frame| {
            let response = T::Response::from_frame(frame);
            tx.send(response).ok();
        };

        self.task_tx
            .send(TaskMessage::CommandRequest {
                command_outgoing: Box::new(outgoing_command.into_outgoing()),
                response_parser: Box::new(response_parser),
            })
            .map_err(|e| HandleError::TaskFailure)?;

        rx.await.map_err(|e| HandleError::TaskFailure)
    }
}
