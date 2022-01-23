use std::path::PathBuf;

use tokio::{sync::mpsc, task::JoinHandle};

use self::{
    handle::DeconzClientHandle,
    task::{DeconzTask, TaskError},
};

mod handle;
mod queue;
mod task;

/// Common configuration passed to the deCONZ client and used by the underlying task.
#[derive(Clone)]
pub struct DeconzClientConfig {
    /// The path to a deCONZ-compatible device, like /dev/ttyUSB0.
    pub device_path: PathBuf,
}

/// The deCONZ-protocol client, capable of connecting to a device and providing a means to communicate with it.
/// The actual process lives in DeconzTask, this just configures and starts it as a background task.
pub struct DeconzClient {
    config: DeconzClientConfig,
}

impl DeconzClient {
    /// Starts a new deCONZ client task. To start it, use start().
    pub fn new(config: DeconzClientConfig) -> Self {
        Self { config }
    }

    /// Starts a deCONZ task and returns a handle to it.
    pub fn start(self) -> (JoinHandle<Result<(), TaskError>>, DeconzClientHandle) {
        let (task_tx, task_rx) = mpsc::unbounded_channel();
        let task = DeconzTask::new(self.config, task_rx);

        // deconz task runner
        let task_joinhandle = tokio::spawn(task.run());

        (task_joinhandle, DeconzClientHandle::new(task_tx))
    }
}
