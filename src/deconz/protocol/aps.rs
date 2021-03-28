use bytes::{Buf, BufMut, Bytes, BytesMut};
use pretty_hex::PrettyHex;
use tracing::info;

use crate::deconz::DeconzFrame;

use super::{
    device::DeviceState, CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse,
};

#[derive(Debug)]
pub struct ReadReceivedData;

#[derive(Debug)]
pub struct ReadReceivedDataRequest;

#[derive(Debug)]
pub struct ReadReceivedDataResponse {
    pub device_state: DeviceState,
}

impl ReadReceivedData {
    pub fn new() -> Self {
        Self
    }
}

impl DeconzCommand for ReadReceivedData {
    type Request = ReadReceivedDataRequest;
    type Response = ReadReceivedDataResponse;

    fn into_request(self) -> Self::Request {
        Self::Request {}
    }
}

impl DeconzCommandRequest for ReadReceivedDataRequest {
    fn command_id(&self) -> CommandId {
        CommandId::ApsDataIndication
    }

    fn payload_data(&self) -> BytesMut {
        let mut payload = BytesMut::new();
        payload.put_u8(0x01);
        payload
    }
}

impl DeconzCommandResponse for ReadReceivedDataResponse {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let _payload_length = frame.get_u16_le();
        let flags = frame.get_u8();
        let device_state = flags.into();
        info!("frame hex dump: {}", frame.hex_dump());
        (Self { device_state }, Some(device_state))
    }
}
