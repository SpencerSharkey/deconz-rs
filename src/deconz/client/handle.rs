use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use super::task::TaskMessage;
use crate::deconz::protocol;

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("error communicating with the task")]
    RecvError,
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

    async fn command<T: protocol::DeconzCommandOutgoingRequest>(
        outgoing_command: T,
    ) -> Result<T::Response, HandleError> {
        let (tx, rx) = oneshot::channel();

        rx.await
    }
}
