use std::{
    convert::TryInto,
    ops::{Deref, DerefMut},
    u16,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use pretty_hex::PrettyHex;
use thiserror::Error;

use super::protocol::{CommandId, StatusCode};

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("unknown command (id={0})")]
    UnknownCommandId(u8),
    #[error("unknown network state (id={0})")]
    UnknownNetworkState(u8),
    #[error("unknown status code (id={0})")]
    UnknownStatusCode(u8),
    #[error("frame too small (len={0})")]
    SmallFrame(usize),
    #[error("frame too large (len={0})")]
    LargeFrame(usize),
    #[error(transparent)]
    CrcError(#[from] CrcError),
}

/// A raw DeconzFrame, just a Bytes container with some header information.
/// The state of the frame is encoded in the types, Bytes is immutable where BytesMut is a mutable payload.
/// Likewise, a DeconzFrame<OutgoingPacket> is a container ready to send to the hardware device.
#[derive(Debug, Clone)]
pub struct DeconzFrame<T> {
    command_id: CommandId,
    sequence_number: u8,
    status: Option<StatusCode>,
    inner: T,
}

impl DeconzFrame<Bytes> {
    /// Consumes raw frame data and returns a parsed DeconzFrame with remaining payload bytes.
    fn parse_incoming(mut frame: Bytes) -> Result<DeconzFrame<Bytes>, ProtocolError> {
        if frame.remaining() < 5 {
            return Err(ProtocolError::SmallFrame(frame.remaining()));
        }

        let command_id = frame.get_u8().try_into()?;
        let sequence_number = frame.get_u8();
        let status = Some(frame.get_u8().try_into()?);
        let reported_frame_length = frame.get_u16_le();

        if frame.len() > reported_frame_length as usize {
            return Err(ProtocolError::LargeFrame(frame.len()));
        }

        Ok(Self {
            command_id,
            sequence_number,
            status,
            inner: frame,
        })
    }

    /// Returns this frame's command type ID
    pub fn command_id(&self) -> CommandId {
        self.command_id
    }

    /// Returns this frame's sequence number
    pub fn sequence_id(&self) -> u8 {
        self.sequence_number
    }

    /// Returns the frame's status, if it exists.
    pub fn status(&self) -> StatusCode {
        self.status
            .expect("invariant: expected status code for immutable (incoming) deCONZ frame")
    }
}

impl Deref for DeconzFrame<Bytes> {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DeconzFrame<Bytes> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// An outgoing deCONZ packet.
#[derive(Debug, Clone)]
pub struct OutgoingPacket(Option<Bytes>);

impl DeconzFrame<OutgoingPacket> {
    /// From an outgoing payload
    pub fn new(
        command_id: CommandId,
        sequence_number: u8,
        command_payload: Option<BytesMut>,
    ) -> Self {
        Self {
            command_id,
            sequence_number,
            status: None,
            inner: OutgoingPacket(command_payload.map(|b| b.into())),
        }
    }

    fn header_bytes(&self) -> BytesMut {
        // the payload size field demands the header length + command payload length
        let mut frame_len = 5;
        if let Some(bytes) = &self.inner.0 {
            frame_len += 2;
            frame_len += bytes.len();
        }

        let mut buf = BytesMut::with_capacity(frame_len + 2); // + 2 bytes for the CRC value
        buf.put_u8(self.command_id as u8);
        buf.put_u8(self.sequence_number);
        buf.put_u8(0); // status field is 0 (reserved) for outgoing requests
        buf.put_u16_le(frame_len as u16);
        buf
    }

    fn packet_bytes(&self) -> BytesMut {
        let mut buf = self.header_bytes();
        if let Some(bytes) = &self.inner.0 {
            let bytes_len: u16 = match bytes.len().try_into() {
                Ok(b) => b,
                Err(_) => panic!(
                    "tried to pack bytes for packet {:?} that was too big ({} > {})!",
                    self,
                    bytes.len(),
                    u16::MAX
                ),
            };
            buf.put_u16_le(bytes_len);
            buf.put_slice(&bytes);
        }

        buf
    }

    /// Consumes the frame data and builds a finalized packet.
    /// This builds a header, appends the inner Bytes, and generates/appends the 2-byte CRC value.
    /// The bytes returned here are intended for the device sink.
    pub fn encode(self) -> Bytes {
        let mut packet_bytes = self.packet_bytes();
        packet_bytes.hex_dump();
        let crc = DeconzCrc::generate(&packet_bytes);
        packet_bytes.put_slice(&crc.as_slice());
        Bytes::from(packet_bytes)
    }
}

#[derive(Error, Debug)]
pub enum CrcError {
    #[error("invalid crc length (len={0})")]
    WrongSize(usize),
}

/// A deCONZ CRC value consisting of 2 bytes.
/// You may call verify_frame(payload) to validate a raw incoming packet and return a parsed immutable DeconzFrame
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeconzCrc(u8, u8);

impl DeconzCrc {
    /// Generates a CRC value given a variable length payload.
    pub(crate) fn generate<T: AsRef<[u8]>>(input: T) -> Self {
        let sum = input
            .as_ref()
            .iter()
            .fold(0u16, |acc, el| acc.wrapping_add(*el as _));
        Self((!sum + 1) as u8 & 0xFF, ((!sum + 1) >> 8) as u8 & 0xFF)
    }

    /// Creates a new instance of Self from a 2-byte buffer, otherwise None.
    pub(crate) fn from_values(buf: &[u8]) -> Result<Self, CrcError> {
        if buf.len() != 2 {
            return Err(CrcError::WrongSize(buf.len()));
        }
        Ok(Self(buf[0], buf[1]))
    }

    /// Validates a payload against the CRC value and returns a DeconzFrame if succesful.
    pub(crate) fn verify_frame<T: Into<Bytes>>(
        self,
        payload: T,
    ) -> Result<DeconzFrame<Bytes>, ProtocolError> {
        /*
            todo: validate incoming CRC :)
        */
        DeconzFrame::parse_incoming(payload.into())
    }

    /// Returns a 2-byte tuple containing the CRC value
    pub(crate) fn as_slice(&self) -> [u8; 2] {
        [self.0, self.1]
    }
}

#[cfg(test)]
pub mod test {
    use crate::deconz::protocol::{device::ReadFirmwareVersionRequest, DeconzCommandRequest};

    use super::*;

    #[test]
    pub fn test_frame() {
        let packet_bytes = ReadFirmwareVersionRequest.as_frame(0).packet_bytes();
        let crc = DeconzCrc::generate(packet_bytes);
        assert_eq!(crc, DeconzCrc(234, 255))
    }
}
