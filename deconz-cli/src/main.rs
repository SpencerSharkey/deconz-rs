pub mod daemon;
mod net_params;
pub mod util;

use std::{collections::HashMap, path::PathBuf};

use bytes::Buf;
use deconz::{
    protocol::{
        aps::{IEEEAddress, NetworkAddress},
        device::{ChangeNetworkState, ReadDeviceState},
        NetworkState,
    },
    DeconzClient, DeconzClientConfig,
};
use structopt::StructOpt;
use tracing::info;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "deconz-cli",
    about = "Commands to interface with a deCONZ (RaspBee/ConBee) Serial Device"
)]
struct Opt {
    /// Device path where the the deCONZ compatible device is available at.
    #[structopt(short, long, default_value = "/dev/ttyUSB0")]
    device: PathBuf,
    #[structopt(subcommand)]
    command: OptCommand,
}

#[derive(Debug, StructOpt)]
enum OptCommand {
    ReadParameters,
    WriteParameter {
        #[structopt(subcommand)]
        param: net_params::WritableParameter,
    },
    SetOffline,
    SetOnline,
    DeviceState,
    Daemon,
}

#[derive(Debug, Clone)]
struct ZdoDevice {
    ieee: IEEEAddress,
    address: NetworkAddress,
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

    match opt.command {
        OptCommand::Daemon => {
            let mut sub = deconz.subscribe_aps_data_indication().await?;

            // todo: do this better
            tokio::spawn(async { watchdog.await.unwrap().unwrap() });

            let mut devices = HashMap::<IEEEAddress, ZdoDevice>::new();

            loop {
                let data = sub.recv().await?;
                dbg!(&data);

                if data.destination_endpoint == 0 && data.cluster_id == 0x0013 {
                    info!("received device state");

                    let ieee = data.source_address.unwrap_ieee_address();

                    let mut payload = data.data();
                    let _seq = payload.get_u8();
                    let nwk_addr = payload.get_u16_le();
                    let ieee_addr = payload.get_u64_le();

                    if let std::collections::hash_map::Entry::Vacant(e) = devices.entry(ieee) {
                        e.insert(ZdoDevice {
                            ieee: ieee_addr,
                            address: nwk_addr,
                        });
                    } else {
                        info!("Received data from device we already know about");
                    }
                }

                dbg!(&devices);
            }
        }
        OptCommand::WriteParameter { param } => {
            param.write(&mut deconz).await?;
        }
        OptCommand::ReadParameters => {
            net_params::read_all_parameters(&mut deconz).await?;
        }
        OptCommand::DeviceState => {
            let state = deconz.send_command(ReadDeviceState::new()).await?;
            println!("{:?}", state);
        }
        OptCommand::SetOffline => {
            deconz
                .send_command(ChangeNetworkState::new(NetworkState::NetOffline))
                .await?;
        }
        OptCommand::SetOnline => {
            deconz
                .send_command(ChangeNetworkState::new(NetworkState::NetConnected))
                .await?;
        }
    };

    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::fmt().init();
}
