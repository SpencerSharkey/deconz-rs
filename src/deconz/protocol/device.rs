use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{
    CommandType, DeconzCommandIncoming, DeconzCommandOutgoing, DeconzCommandOutgoingRequest,
    DeconzFrame,
};

// Read Firmware Version

#[derive(Debug)]
#[repr(u8)]
pub enum FirmwareVersionPlatform {
    V1,
    V2,
    Unknown(u8),
}

impl From<u8> for FirmwareVersionPlatform {
    fn from(i: u8) -> Self {
        match i {
            0x05 => Self::V1,
            0x07 => Self::V2,
            i => Self::Unknown(i),
        }
    }
}

#[derive(Debug)]
pub struct ReadFirmwareVersionRequest;

impl DeconzCommandOutgoing for ReadFirmwareVersionRequest {
    fn get_command_id(&self) -> CommandType {
        CommandType::Version
    }

    fn payload_data(&self) -> BytesMut {
        let mut payload = BytesMut::new();
        payload.put_u32_le(0); // Reserved
        payload
    }
}

pub struct ReadCommandVersion;

impl ReadCommandVersion {
    pub fn new() -> Self {
        Self {}
    }
}

impl DeconzCommandOutgoingRequest for ReadCommandVersion {
    type Request = ReadFirmwareVersionRequest;
    type Response = ReadFirmwareVersionResponse;

    fn into_outgoing(self) -> Self::Request {
        Self::Request {}
    }
}

#[derive(Debug)]
pub struct ReadFirmwareVersionResponse {
    major_version: u8,
    minor_version: u8,
    platform: FirmwareVersionPlatform,
}

impl DeconzCommandIncoming for ReadFirmwareVersionResponse {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
        let _reserved = frame.get_u8();
        Self {
            platform: frame.get_u8().into(),
            minor_version: frame.get_u8(),
            major_version: frame.get_u8(),
        }
    }
}
