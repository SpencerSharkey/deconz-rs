use bytes::{Buf, Bytes};

use crate::{
    protocol::{aps::SourceAddress, device::DeviceState},
    DeconzFrame,
};

use super::super::DeconzCommandResponse;

#[derive(Debug)]
pub struct MACBeaconIndication {
    /// "16-bit short address of a router or coordinator (0x0000)"
    /// I have no idea what that means, but thats from the deconz serial
    /// protocol doc.
    pub source_address: SourceAddress,
    /// The Zigbee network identifier.
    pub network_pan_id: u16,
    /// The Zigbee channel
    pub channel: u8,
    /// Beacon flags
    pub flags: u8,
    /// The Zigbee netwrok Update ID
    pub update_id: u8,
    /// Optional additional beacon data.
    pub data: Option<Vec<u8>>,
}

impl DeconzCommandResponse for MACBeaconIndication {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let _payload_length = frame.get_u16_le();
        let source_address = SourceAddress::NetworkAddress(frame.get_u16_le());
        let network_pan_id = frame.get_u16_le();
        let channel = frame.get_u8();
        let flags = frame.get_u8();
        let update_id = frame.get_u8();

        let data = match frame.has_remaining() {
            true => Some(frame.to_vec()),
            false => None,
        };

        (
            Self {
                source_address,
                network_pan_id,
                channel,
                data,
                flags,
                update_id,
            },
            None,
        )
    }
}
