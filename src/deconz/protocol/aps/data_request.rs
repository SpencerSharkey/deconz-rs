use std::num::NonZeroU8;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::deconz::DeconzFrame;

// todo: remove super imports, use crate level relatives.
use super::{
    super::{
        device::DeviceState, CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse,
    },
    DestinationAddress,
};

#[derive(Debug)]
pub struct APSFramePayload(Vec<u8>);

/// Returned by [`APSFramePayload::from_vec`] if the provided vec is over 127 bytes.
#[derive(Debug)]
pub struct OverflowError;

impl APSFramePayload {
    pub fn from_vec(vec: Vec<u8>) -> Result<Self, OverflowError> {
        if vec.len() > 127 {
            Err(OverflowError)
        } else {
            Ok(Self(vec))
        }
    }
}

#[derive(Default, Debug)]
pub struct SendDataOptions {
    pub use_aps_acks: bool,
}

#[derive(Debug)]
pub struct SendData {
    pub destination_address: DestinationAddress,
    /// Only included if destination address mode is [`DestinationAddress::NetworkAddress`]
    /// or [`DestinationAddress::IEEEAddress`].
    pub destination_endpoint: u8,
    pub profile_id: u16,
    pub cluster_id: u16,
    pub source_endpoint: u8,
    pub payload: APSFramePayload,
    pub options: SendDataOptions,
    /// The maximum hops the request will be forwarded,
    /// set to None for unlimited hops.
    pub radius: Option<NonZeroU8>,
}

#[derive(Debug)]
pub struct SendDataRequest {
    inner: SendData,
    request_id: u8,
}

#[derive(Debug)]
pub struct SendDataResponse {
    pub request_id: u8,
}

impl DeconzCommand for SendData {
    type Request = SendDataRequest;
    type Response = SendDataResponse;

    fn into_request(self) -> Self::Request {
        Self::Request {
            inner: self,
            // todo: plumb down how to generate the request id...
            request_id: 0,
        }
    }
}

impl DeconzCommandRequest for SendDataRequest {
    fn command_id(&self) -> CommandId {
        CommandId::ApsDataRequest
    }

    fn payload_data(&self) -> Option<BytesMut> {
        let mut payload = BytesMut::new();
        let inner = &self.inner;
        payload.put_u8(self.request_id);
        payload.put_u8(0); // flags. docs seem to say there are none yet.
        match &inner.destination_address {
            DestinationAddress::GroupAddress(group_address) => {
                payload.put_u8(0x01);
                payload.put_u16_le(*group_address);
            }
            DestinationAddress::NetworkAddress(network_address) => {
                payload.put_u8(0x02);
                payload.put_u16_le(*network_address);
                payload.put_u8(inner.destination_endpoint);
            }
            DestinationAddress::IEEEAddress(ieee_address) => {
                payload.put_u8(0x03);
                payload.put_u64_le(*ieee_address);
                payload.put_u8(inner.destination_endpoint);
            }
        }

        payload.put_u16_le(inner.profile_id);
        payload.put_u16_le(inner.cluster_id);
        payload.put_u8(inner.source_endpoint);
        payload.put_u16_le(inner.payload.0.len() as _);
        payload.put_slice(&inner.payload.0[..]);

        if inner.options.use_aps_acks {
            payload.put_u8(0x04);
        } else {
            payload.put_u8(0);
        }

        match inner.radius {
            Some(radius) => payload.put_u8(radius.get()),
            None => payload.put_u8(0),
        }

        Some(payload)
    }
}

impl DeconzCommandResponse for SendDataResponse {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let _payload_length = frame.get_u16_le();
        let device_state = frame.get_u8().into();
        let request_id = frame.get_u8();
        let response = Self { request_id };

        (response, Some(device_state))
    }
}
