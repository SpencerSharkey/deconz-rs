use tokio::sync::mpsc;
use tokio_serial::SerialPortSettings;

use super::DeconzClientConfig;

pub enum TaskMessage {}

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
    pub async fn run(mut self) {
        loop {
            let serialfd = tokio_serial::Serial::from_path(
                self.config.device_path.clone(),
                &SerialPortSettings {
                    baud_rate: 38400,
                    ..Default::default()
                },
            )
            .unwrap();

            tokio::select! {
                msg = self.task_rx.recv() => {

                }
            }
        }
    }
}
