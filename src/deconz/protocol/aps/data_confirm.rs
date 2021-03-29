use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::deconz::DeconzFrame;

// todo: remove super imports, use crate level relatives.
use super::super::{
    device::DeviceState, CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse,
};

use super::{DestinationAddress, SourceAddress};

#[derive(Debug)]
pub struct ReadConfirmData;

#[derive(Debug)]
pub struct ReadConfirmDataRequest;

#[derive(Debug)]
pub struct ReadConfirmDataResponse {
    pub device_state: DeviceState,
    pub request_id: u8,
    pub destination_address: DestinationAddress,
    pub destination_endpoint: Option<u8>,
    pub source_endpoint: u8,
    pub confirm_status: u8,
}

impl ReadConfirmData {
    pub fn new() -> Self {
        Self
    }
}

impl DeconzCommand for ReadConfirmData {
    type Request = ReadConfirmDataRequest;
    type Response = ReadConfirmDataResponse;

    fn into_request(self) -> Self::Request {
        Self::Request {}
    }
}

impl DeconzCommandRequest for ReadConfirmDataRequest {
    fn command_id(&self) -> CommandId {
        CommandId::ApsDataConfirm
    }

    fn payload_data(&self) -> Option<BytesMut> {
        // The APS_DATA_CONFIRM request must contain an empty payload, with a
        // length of 0, so we use Some(empty payload) instead of None.
        Some(BytesMut::new())
    }
}

impl DeconzCommandResponse for ReadConfirmDataResponse {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let _payload_length = frame.get_u16_le();
        let device_state = frame.get_u8().into();
        let request_id = frame.get_u8();
        let destination_address_mode = frame.get_u8();
        let destination_address = match destination_address_mode {
            0x01 => DestinationAddress::GroupAddress(frame.get_u16_le()),
            0x02 => DestinationAddress::NetworkAddress(frame.get_u16_le()),
            0x03 => DestinationAddress::IEEEAddress(frame.get_u64_le()),
            other => panic!("Unexpected destination address mode: {:?}", other),
        };
        let destination_endpoint = match destination_address_mode {
            0x02 | 0x03 => Some(frame.get_u8()),
            _ => None,
        };
        let source_endpoint = frame.get_u8();
        let confirm_status = frame.get_u8();

        (
            Self {
                device_state,
                destination_address,
                destination_endpoint,
                source_endpoint,
                confirm_status,
                request_id,
            },
            Some(device_state),
        )
    }
}
