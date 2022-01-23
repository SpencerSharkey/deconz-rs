use std::time::Duration;

use deconz::{
    protocol::{device::ReadFirmwareVersion, network_parameters},
    DeconzClientHandle,
};
use structopt::StructOpt;

use crate::util::hex_string::HexString;

#[derive(Debug, StructOpt)]
pub(crate) enum WritableParameter {
    NetworkPanId {
        value: HexString<u16>,
    },
    ApsDesignatedCoordinator {
        value: network_parameters::parameters::APSDesignatedCoordinator,
    },
    ChannelMask {
        value: HexString<u32>,
    },
    ApsExtendedPanId {
        value: HexString<u64>,
    },
    TrustCenterAddress {
        value: HexString<u64>,
    },
    SecurityMode {
        value: network_parameters::parameters::SecurityMode,
    },
    PredefinedNetworkPanId {
        value: network_parameters::parameters::PredefinedNetworkPanId,
    },
    NetworkKey {
        value: HexString<[u8; 16]>,
    },
    NetworkUpdateId {
        value: u8,
    },
    WatchdogTtl {
        value: u32,
    },
    NetworkFrameCounter {
        value: u32,
    },
}

impl WritableParameter {
    pub async fn write(self, deconz: &mut DeconzClientHandle) -> Result<(), anyhow::Error> {
        match self {
            WritableParameter::NetworkPanId { value } => {
                deconz
                    .send_command(network_parameters::WriteNetworkPanId::new(*value))
                    .await?;
            }
            WritableParameter::ApsDesignatedCoordinator { value } => {
                deconz
                    .send_command(network_parameters::WriteAPSDesignatedCoordinator::new(
                        value,
                    ))
                    .await?;
            }
            WritableParameter::ChannelMask { value } => {
                deconz
                    .send_command(network_parameters::WriteChannelMask::new(*value))
                    .await?;
            }
            WritableParameter::ApsExtendedPanId { value } => {
                deconz
                    .send_command(network_parameters::WriteAPSExtendedPanId::new(*value))
                    .await?;
            }
            WritableParameter::TrustCenterAddress { value } => {
                deconz
                    .send_command(network_parameters::WriteTrustCenterAddress::new(*value))
                    .await?;
            }
            WritableParameter::SecurityMode { value } => {
                deconz
                    .send_command(network_parameters::WriteSecurityMode::new(value))
                    .await?;
            }
            WritableParameter::PredefinedNetworkPanId { value } => {
                deconz
                    .send_command(network_parameters::WritePredefinedNetworkPanId::new(value))
                    .await?;
            }
            WritableParameter::NetworkKey { value } => {
                deconz
                    .send_command(network_parameters::WriteNetworkKey::new(
                        network_parameters::parameters::NetworkKey::Set(*value),
                    ))
                    .await?;
            }
            WritableParameter::NetworkUpdateId { value } => {
                deconz
                    .send_command(network_parameters::WriteNetworkUpdateId::new(value))
                    .await?;
            }
            WritableParameter::WatchdogTtl { value } => {
                deconz
                    .send_command(network_parameters::WriteWatchdogTtl::new(
                        Duration::from_secs(value as u64),
                    ))
                    .await?;
            }
            WritableParameter::NetworkFrameCounter { value } => {
                deconz
                    .send_command(network_parameters::WriteNetworkFrameCounter::new(value))
                    .await?;
            }
        };

        Ok(())
    }
}

pub async fn read_all_parameters(deconz: &mut DeconzClientHandle) -> Result<(), anyhow::Error> {
    let firmware_version_res = deconz.send_command(ReadFirmwareVersion::new()).await?;
    println!(
        "Firmware Version: major={}, minor={}, platform={:?}",
        firmware_version_res.major_version,
        firmware_version_res.minor_version,
        firmware_version_res.platform
    );

    println!(
        "Watchdog TTL: {:?}",
        deconz
            .send_command(network_parameters::ReadWatchdogTtl::new())
            .await?
            .value
    );

    println!(
        "MAC Address: {}",
        deconz
            .send_command(network_parameters::ReadMacAddress::new())
            .await?
            .value
    );

    println!(
        "NWK Address: {}",
        deconz
            .send_command(network_parameters::ReadNetworkAddress::new())
            .await?
            .value
    );

    println!(
        "NWK PANID: {}",
        deconz
            .send_command(network_parameters::ReadNetworkPanId::new())
            .await?
            .value
    );

    println!(
        "NWK Ext PANID: {}",
        deconz
            .send_command(network_parameters::ReadAPSExtendedPanId::new())
            .await?
            .value
    );

    println!(
        "APS Mode: {}",
        deconz
            .send_command(network_parameters::ReadAPSDesignatedCoordinator::new())
            .await?
            .value
    );

    println!(
        "Trust Center Address: {}",
        deconz
            .send_command(network_parameters::ReadTrustCenterAddress::new())
            .await?
            .value
    );

    println!(
        "Security Mode: {:?}",
        deconz
            .send_command(network_parameters::ReadSecurityMode::new())
            .await?
            .value
    );

    println!(
        "Predefined NWK PANID?: {:?}",
        deconz
            .send_command(network_parameters::ReadPredefinedNetworkPanId::new())
            .await?
            .value
    );

    println!(
        "Network Key: {:?}",
        deconz
            .send_command(network_parameters::ReadNetworkKey::new())
            .await?
            .value
    );

    println!(
        "Current Channel: {:?}",
        deconz
            .send_command(network_parameters::ReadCurrentChannel::new())
            .await?
            .value
    );

    Ok(())
}
