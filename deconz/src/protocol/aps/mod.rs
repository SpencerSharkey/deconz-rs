mod data_confirm;
mod data_indication;
mod data_request;

use bytes::{Buf, Bytes};
pub use data_confirm::{ReadConfirmData, ReadConfirmDataRequest, ReadConfirmDataResponse};
pub use data_indication::{ReadReceivedData, ReadReceivedDataRequest, ReadReceivedDataResponse};
pub use data_request::{
    APSFramePayload, OverflowError, SendData, SendDataOptions, SendDataRequest, SendDataResponse,
};

use crate::DeconzFrame;

#[derive(Debug, Clone, Copy)]
pub enum DestinationAddress {
    GroupAddress(u16),
    NetworkAddress(u16),
    IEEEAddress(u64),
}

#[derive(Debug, Clone, Copy)]
pub enum SourceAddress {
    NetworkAddress(u16),
    IEEEAddress(u64),
    Both {
        network_address: u16,
        ieee_address: u64,
    },
}

impl SourceAddress {
    pub(crate) fn from_frame(frame: &mut DeconzFrame<Bytes>) -> Self {
        let source_address_mode = frame.get_u8();
        match source_address_mode {
            0x02 => SourceAddress::NetworkAddress(frame.get_u16_le()),
            0x03 => SourceAddress::IEEEAddress(frame.get_u64_le()),
            0x04 => SourceAddress::Both {
                network_address: frame.get_u16_le(),
                ieee_address: frame.get_u64_le(),
            },
            other => panic!("Unexpected source address mode: {:?}", other),
        }
    }
}
