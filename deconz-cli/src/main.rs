mod net_params;
pub mod util;

use std::path::PathBuf;

use deconz::{DeconzClient, DeconzClientConfig};
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
        OptCommand::WriteParameter { param } => {
            param.write(&mut deconz).await?;
        }
        OptCommand::ReadParameters => {
            net_params::read_all_parameters(&mut deconz).await?;
        }
    };

    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::fmt().init();
}
