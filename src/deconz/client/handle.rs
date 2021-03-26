use tokio::sync::mpsc;

use super::task::TaskMessage;

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
}
