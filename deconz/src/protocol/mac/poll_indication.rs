use bytes::{Buf, Bytes};

use crate::{
    protocol::{aps::SourceAddress, device::DeviceState},
    DeconzFrame,
};

use super::super::DeconzCommandResponse;

#[derive(Debug)]
pub struct MACPollIndication {
    pub source_address: SourceAddress,
    /// The received LQI value 0–255
    pub link_quality_indicator: u8,
    /// The received RSSI value in dBm
    pub received_signal_strength_indication: i8,
    pub neighbor_table_state: Option<NeighborTableState>,
}

#[derive(Debug)]
pub struct NeighborTableState {
    pub life_time: u32,
    pub device_timeout: u32,
}

impl DeconzCommandResponse for MACPollIndication {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let _payload_length = frame.get_u16_le();
        let source_address = SourceAddress::from_frame(&mut frame);
        let link_quality_indicator = frame.get_u8();
        let received_signal_strength_indication = frame.get_i8();

        let neighbor_table_state = match frame.has_remaining() {
            true => Some(NeighborTableState {
                life_time: frame.get_u32_le(),
                device_timeout: frame.get_u32_le(),
            }),
            false => None,
        };

        (
            Self {
                source_address,
                received_signal_strength_indication,
                neighbor_table_state,
                link_quality_indicator,
            },
            None,
        )
    }
}
