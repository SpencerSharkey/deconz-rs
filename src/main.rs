pub mod deconz;

use std::path::PathBuf;

use deconz::{
    protocol::{device::ReadFirmwareVersionRequest, DeconzCommandOutgoing, IncomingCommand},
    DeconzClient, DeconzClientConfig,
};
use structopt::StructOpt;
use tokio_serial::SerialPortSettings;
use tracing::info;

#[derive(Debug, StructOpt)]
#[structopt(name = "deconz-tap", about = "Taps a deCONZ device over serial/usb")]
struct Opt {
    /// Device path where the the deCONZ compatible device is available at.
    #[structopt(short, long, default_value = "/dev/ttyUSB0")]
    device: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    setup_tracing();

    let opt = Opt::from_args();

    info!("connecting to device {:?}", opt.device);

    let deconz_config = DeconzClientConfig {
        device_path: opt.device,
    };

    let (watchdog, mut deconz) = DeconzClient::new(deconz_config).start();

    watchdog.await??;

    Ok(())
    // deconz
    //     .write_command(0, ReadFirmwareVersionRequest)
    //     .await
    //     .unwrap();

    // loop {
    //     info!(
    //         "got frame: {:?}",
    //         deconz
    //             .next_frame()
    //             .await
    //             .unwrap()
    //             .map(|e| { IncomingCommand::decode_frame(e) })
    //     );
    // }
}

fn setup_tracing() {
    tracing_subscriber::fmt().init();
}
