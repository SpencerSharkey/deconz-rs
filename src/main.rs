pub mod deconz;

use std::{path::PathBuf, time::Duration};

use deconz::{
    protocol::{aps::*, device::*, network_parameters::*},
    DeconzClient, DeconzClientConfig,
};
use structopt::StructOpt;
use tokio::{task, time::sleep};
// use tokio_serial::SerialPortSettings;
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

    let mut hdl = deconz.clone();
    task::spawn(async move {
        loop {
            let x = hdl.send_command(ReadDeviceState::new()).await;
            info!("device state is: {:?}", x);
            sleep(Duration::from_secs(1)).await;
        }
    });

    // let mut hdl = deconz.clone();
    // task::spawn(async move {
    //     loop {
    //         info!("beep boop...");
    //         let mut hdl = hdl.clone();
    //         task::spawn(async move {
    //             let fut = hdl.send_command(ReadWatchdogTtl::new());
    //             fut.await;
    //         });
    //         // info!("device state is: {:?}", x);
    //         sleep(Duration::from_secs(1)).await;
    //     }
    // });

    dbg!(deconz.send_command(ReadWatchdogTtl::new()).await);
    // dbg!(deconz.send_command(ReadCommandVersion::new()).await);
    // dbg!(deconz.send_command(ReadNetworkKey::new()).await);
    // // dbg!(deconz.send_command(ReadNetworkAddress::new()).await);
    // dbg!(
    //     deconz
    //         .send_command(ReadAPSDesignatedCoordinator::new())
    //         .await
    // );
    // dbg!(deconz.send_command(ReadMacAddress::new()).await);
    // dbg!(deconz.send_command(ReadNetworkPanId::new()).await);
    // dbg!(deconz.send_command(ReadCurrentChannel::new()).await);
    // dbg!(deconz.send_command(ReadNetworkFrameCounter::new()).await);
    // dbg!(deconz.send_command(ReadReceivedData::new()).await);

    // loop {
    //     let data = deconz.send_command(ReadReceivedData::new()).await;
    //     dbg!(data);
    //     sleep(Duration::from_secs(10)).await;
    // }

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
