pub mod deconz;

use std::path::PathBuf;

use deconz::{
    protocol::{
        device::{ReadCommandVersion, ReadFirmwareVersionRequest},
        network_parameters::{
            ReadAPSDesignatedCoordinator, ReadMacAddress, ReadNetworkAddress, ReadNetworkKey,
        },
        DeconzCommandRequest,
    },
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

    // dbg!(deconz.send_command(ReadCommandVersion::new()).await);
    dbg!(deconz.send_command(ReadNetworkKey::new()).await);
    // dbg!(deconz.send_command(ReadNetworkAddress::new()).await);
    dbg!(
        deconz
            .send_command(ReadAPSDesignatedCoordinator::new())
            .await
    );
    // dbg!(deconz.send_command(ReadMacAddress::new()).await);

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
