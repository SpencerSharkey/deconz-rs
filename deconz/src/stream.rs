use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use slip_codec::{SlipError, tokio::SlipCodec};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

use super::frame::{DeconzCrc, DeconzFrame, OutgoingPacket, ProtocolError};

#[derive(Error, Debug)]
pub enum DeconzStreamError {
    #[error("read error (payload={0:?})")]
    Read(Bytes),
    #[error("codec error: {0:?}")]
    SlipCodec(SlipError),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}

/// Reads incoming bytes and returns a validated DeconzFrame
fn read_frame(mut bytes: BytesMut) -> Result<DeconzFrame<Bytes>, DeconzStreamError> {
    let bytes_len = bytes.len();
    if bytes_len < 2 {
        return Err(DeconzStreamError::Read(bytes.into()));
    }

    // Read the CRC value (last 2 bytes from of a frame) and use it to receive a verified DeconzFrame.
    let crc_bytes = bytes.split_off(bytes_len - 2);
    let crc =
        DeconzCrc::from_values(&crc_bytes).map_err(|e| DeconzStreamError::Protocol(e.into()))?;
    Ok(crc.verify_frame(bytes)?)
}

/// A wrapper for an AsyncRead + AsyncWrite that allows for reading and writing deCONZ protocol packets.
/// SLIP encapsulation and CRC generation/validation is built-in, so just send your structured payloads.
pub struct DeconzStream<S: AsyncRead + AsyncWrite> {
    slip_stream: Framed<S, SlipCodec>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> DeconzStream<S> {
    /// Creates a new deCONZ stream from anything that implements AsyncRead + AsyncWrite. For example, a tokio::fs::File.
    pub fn new(stream: S) -> Self {
        let slip_stream = tokio_util::codec::Framed::new(stream, slip_codec::tokio::SlipCodec::new());
        Self { slip_stream }
    }

    /// Reads until the next frame is received, where it will validate and yield a new DeconzFrame.
    /// Returns None if the underlying stream has ended.
    pub async fn next_frame(&mut self) -> Option<Result<DeconzFrame<Bytes>, DeconzStreamError>> {
        let a = self.slip_stream.next().await;
        match a {
            Some(Ok(bytes)) => Some(read_frame(bytes)),
            Some(Err(e)) => Some(Err(DeconzStreamError::SlipCodec(e))),
            None => None,
        }
    }

    /// Writes a frame to the stream, encoding it on the way out.
    pub async fn write_frame(
        &mut self,
        payload: DeconzFrame<OutgoingPacket>,
    ) -> Result<(), DeconzStreamError> {
        self.slip_stream
            .send(payload.encode())
            .await
            .map_err(DeconzStreamError::SlipCodec)?;

        Ok(())
    }
}
