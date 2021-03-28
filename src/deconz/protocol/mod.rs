pub mod device;
pub mod network_parameters;

use std::convert::TryFrom;
use std::fmt::Debug;

use bytes::{Bytes, BytesMut};

use super::{
    frame::{OutgoingPacket, ProtocolError},
    DeconzFrame,
};

/// To be implemented on all outgoing command structs
pub trait DeconzCommandRequest: Debug + Send + 'static {
    fn command_id(&self) -> CommandId;

    /// Returns the payload to proceed the header
    fn payload_data(&self) -> BytesMut;

    /// Concatenates the packet payload onto a common header
    fn into_frame(&self, sequence_number: u8) -> DeconzFrame<OutgoingPacket> {
        DeconzFrame::new(self.command_id(), sequence_number, self.payload_data())
    }
}

pub trait DeconzCommand {
    type Request: DeconzCommandRequest;
    type Response: DeconzCommandResponse;

    fn into_request(self) -> Self::Request;
}

pub trait DeconzCommandResponse: Send + 'static {
    /// Parses an incoming payload frame into the right type
    fn from_frame(frame: DeconzFrame<Bytes>) -> Self;
}

#[derive(Debug)]
struct UnsupportedValueError<T>(T);

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum StatusCode {
    Success = 0x00,
    Failure = 0x01,
    Busy = 0x02,
    Timeout = 0x03,
    Unsupported = 0x04,
    Error = 0x05,
    NoNetwork = 0x06,
    InvalidValue = 0x07,
}

impl TryFrom<u8> for StatusCode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            x if x == Self::Success as u8 => Ok(Self::Success),
            x if x == Self::Failure as u8 => Ok(Self::Failure),
            x if x == Self::Busy as u8 => Ok(Self::Busy),
            x if x == Self::Timeout as u8 => Ok(Self::Timeout),
            x if x == Self::Unsupported as u8 => Ok(Self::Unsupported),
            x if x == Self::Error as u8 => Ok(Self::Error),
            x if x == Self::NoNetwork as u8 => Ok(Self::NoNetwork),
            x if x == Self::InvalidValue as u8 => Ok(Self::InvalidValue),
            x => Err(ProtocolError::UnknownCommandId(x)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum NetworkState {
    NetOffline = 0x00,
    NetJoining = 0x01,
    NetConnected = 0x02,
    NetLeaving = 0x03,
}

impl TryFrom<u8> for NetworkState {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::NetOffline as u8 => Ok(Self::NetOffline),
            x if x == Self::NetJoining as u8 => Ok(Self::NetJoining),
            x if x == Self::NetConnected as u8 => Ok(Self::NetConnected),
            x if x == Self::NetLeaving as u8 => Ok(Self::NetLeaving),
            x => Err(ProtocolError::UnknownNetworkState(x)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum CommandId {
    DeviceState = 0x07,
    ChangeNetworkState = 0x08,
    ReadParameter = 0x0A,
    WriteParameter = 0x0B,
    DeviceStateChanged = 0x0E,
    Version = 0x0D,
    ApsDataRequest = 0x12,
    ApsDataConfirm = 0x04,
    ApsDataIndication = 0x17,
    MacPollIndication = 0x1C,
    MacBeaconIndication = 0x1F,
    UpdateBootloader = 0x21,
}

impl TryFrom<u8> for CommandId {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::DeviceState as u8 => Ok(Self::DeviceState),
            x if x == Self::ChangeNetworkState as u8 => Ok(Self::ChangeNetworkState),
            x if x == Self::ReadParameter as u8 => Ok(Self::ReadParameter),
            x if x == Self::WriteParameter as u8 => Ok(Self::WriteParameter),
            x if x == Self::DeviceStateChanged as u8 => Ok(Self::DeviceStateChanged),
            x if x == Self::Version as u8 => Ok(Self::Version),
            x if x == Self::ApsDataRequest as u8 => Ok(Self::ApsDataRequest),
            x if x == Self::ApsDataConfirm as u8 => Ok(Self::ApsDataConfirm),
            x if x == Self::ApsDataIndication as u8 => Ok(Self::ApsDataIndication),
            x if x == Self::MacPollIndication as u8 => Ok(Self::MacPollIndication),
            x if x == Self::MacBeaconIndication as u8 => Ok(Self::MacBeaconIndication),
            x if x == Self::UpdateBootloader as u8 => Ok(Self::UpdateBootloader),
            x => Err(ProtocolError::UnknownCommandId(x)),
        }
    }
}
