use std::path::PathBuf;

use deconz::{
    protocol::{device::*, network_parameters::*},
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
    ReadInfo,
    PermitJoin { seconds: u8 },
    Daemon,
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
            watchdog.await??;
        }
        OptCommand::ReadInfo => {
            let firmware_version_res = deconz.send_command(ReadFirmwareVersion::new()).await?;
            println!(
                "Firmware Version: major={}, minor={}, platform={:?}",
                firmware_version_res.major_version,
                firmware_version_res.minor_version,
                firmware_version_res.platform
            );

            println!(
                "Watchdog TTL: {:?}",
                deconz.send_command(ReadWatchdogTtl::new()).await?.value
            );

            println!(
                "MAC Address: {}",
                deconz.send_command(ReadMacAddress::new()).await?.value
            );

            println!(
                "NWK Address: {}",
                deconz.send_command(ReadNetworkAddress::new()).await?.value
            );

            println!(
                "NWK PANID: {}",
                deconz.send_command(ReadNetworkPanId::new()).await?.value
            );

            println!(
                "NWK Ext PANID: {}",
                deconz
                    .send_command(ReadAPSExtendedPanId::new())
                    .await?
                    .value
            );

            println!(
                "APS Mode: {}",
                deconz
                    .send_command(ReadAPSDesignatedCoordinator::new())
                    .await?
                    .value
            );

            println!(
                "Trust Center Address: {}",
                deconz
                    .send_command(ReadTrustCenterAddress::new())
                    .await?
                    .value
            );

            println!(
                "Security Mode: {:?}",
                deconz.send_command(ReadSecurityMode::new()).await?.value
            );

            println!(
                "Predefined NWK PANID?: {:?}",
                deconz
                    .send_command(ReadPredefinedNetworkPanId::new())
                    .await?
                    .value
            );

            println!(
                "Network Key: {:?}",
                deconz.send_command(ReadNetworkKey::new()).await?.value
            );

            println!(
                "Current Channel: {:?}",
                deconz.send_command(ReadCurrentChannel::new()).await?.value
            );

            println!(
                "Permit Join: {:?}",
                deconz.send_command(ReadPermitJoin::new()).await?.value
            );
        }
        OptCommand::PermitJoin { seconds } => {
            dbg!(deconz.send_command(WritePermitJoin::new(seconds)).await?);
        }
    };

    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::fmt().init();
}
