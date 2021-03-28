use std::convert::TryInto;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use pretty_hex::PrettyHex;

use super::{
    CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse, DeconzFrame,
    NetworkState,
};

// Read Firmware Version

#[derive(Debug)]
#[repr(u8)]
pub enum FirmwareVersionPlatform {
    /// ConBee and RaspBee (AVR)
    Avr,
    /// ConBee II and RaspBee II (ARM/R21)
    ArmR21,
    Unknown(u8),
}

impl From<u8> for FirmwareVersionPlatform {
    fn from(i: u8) -> Self {
        match i {
            0x05 => Self::Avr,
            0x07 => Self::ArmR21,
            i => Self::Unknown(i),
        }
    }
}

#[derive(Debug)]
pub struct ReadFirmwareVersionRequest;

impl DeconzCommandRequest for ReadFirmwareVersionRequest {
    fn command_id(&self) -> CommandId {
        CommandId::Version
    }

    fn payload_data(&self) -> BytesMut {
        let mut payload = BytesMut::new();
        payload.put_u16_le(0); // Reserved
        payload
    }
}

pub struct ReadCommandVersion;

impl ReadCommandVersion {
    pub fn new() -> Self {
        Self {}
    }
}

impl DeconzCommand for ReadCommandVersion {
    type Request = ReadFirmwareVersionRequest;
    type Response = ReadFirmwareVersionResponse;

    fn into_request(self) -> Self::Request {
        Self::Request {}
    }
}

#[derive(Debug)]
pub struct ReadFirmwareVersionResponse {
    major_version: u8,
    minor_version: u8,
    platform: FirmwareVersionPlatform,
}

impl DeconzCommandResponse for ReadFirmwareVersionResponse {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let _reserved = frame.get_u8();
        (
            Self {
                platform: frame.get_u8().into(),
                minor_version: frame.get_u8(),
                major_version: frame.get_u8(),
            },
            None,
        )
    }
}

pub struct ReadDeviceState;

impl ReadDeviceState {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct ReadDeviceStateRequest;

#[derive(Debug)]
pub struct ReadDeviceStateResponse {
    pub device_state: DeviceState,
}

impl DeconzCommand for ReadDeviceState {
    type Request = ReadDeviceStateRequest;
    type Response = ReadDeviceStateResponse;

    fn into_request(self) -> Self::Request {
        Self::Request {}
    }
}

impl DeconzCommandRequest for ReadDeviceStateRequest {
    fn command_id(&self) -> CommandId {
        CommandId::DeviceState
    }

    fn payload_data(&self) -> BytesMut {
        let mut payload = BytesMut::new();
        payload.put_u8(0); // Reserved
        payload.put_u8(0); // Reserved
        payload.put_u8(0); // Reserved
        payload
    }
}

impl DeconzCommandResponse for ReadDeviceStateResponse {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let flags = frame.get_u8();
        let device_state = flags.into();
        (Self { device_state }, Some(device_state))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceState {
    network_state: NetworkState,
    apsde_data_confirm: bool,
    apsde_data_indication: bool,
    configuration_changed: bool,
    apsde_data_request_free_slots: bool,
}

impl From<u8> for DeviceState {
    fn from(flags: u8) -> Self {
        let network_state = (flags & 0x03).try_into().unwrap(); // This should *never* fail.
        let flag_set = move |flag: u8| flags & flag == flag;
        Self {
            network_state,
            apsde_data_confirm: flag_set(0x04),
            apsde_data_indication: flag_set(0x08),
            configuration_changed: flag_set(0x10),
            apsde_data_request_free_slots: flag_set(0x20),
        }
    }
}
