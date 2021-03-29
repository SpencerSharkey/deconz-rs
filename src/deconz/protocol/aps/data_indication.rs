use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::deconz::DeconzFrame;

use super::{
    super::{
        device::DeviceState, CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse,
    },
    DestinationAddress, SourceAddress,
};

#[derive(Debug)]
pub struct ReadReceivedData;

#[derive(Debug)]
pub struct ReadReceivedDataRequest;

#[derive(Debug)]
pub struct ReadReceivedDataResponse {
    pub device_state: DeviceState,
    pub destination_address: DestinationAddress,
    pub destination_endpoint: u8,
    pub source_address: SourceAddress,
    pub source_endpoint: u8,
    pub profile_id: u16,
    pub cluster_id: u16,
    pub application_specific_data_unit: Vec<u8>,
    pub link_quality_indication: u8,
    pub received_signal_strength_indication: i8,
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

    fn payload_data(&self) -> Option<BytesMut> {
        let mut payload = BytesMut::new();
        payload.put_u8(0x04);
        Some(payload)
    }
}

impl DeconzCommandResponse for ReadReceivedDataResponse {
    fn from_frame(mut frame: DeconzFrame<Bytes>) -> (Self, Option<DeviceState>) {
        let _payload_length = frame.get_u16_le();
        let flags = frame.get_u8();
        let device_state = flags.into();

        let destination_address_mode = frame.get_u8();
        let destination_address = match destination_address_mode {
            0x01 => DestinationAddress::GroupAddress(frame.get_u16_le()),
            0x02 => DestinationAddress::NetworkAddress(frame.get_u16_le()),
            0x03 => DestinationAddress::IEEEAddress(frame.get_u64_le()),
            other => panic!("Unexpected destination address mode: {:?}", other),
        };
        let destination_endpoint = frame.get_u8();

        // We are expecting 0x04 here, because of our request flags.
        let source_address = SourceAddress::from_frame(&mut frame);
        let source_endpoint = frame.get_u8();

        let profile_id = frame.get_u16_le();
        let cluster_id = frame.get_u16_le();

        let application_specific_data_unit_length = frame.get_u16_le();
        let mut application_specific_data_unit =
            vec![0u8; application_specific_data_unit_length as usize];
        frame.copy_to_slice(&mut application_specific_data_unit);

        frame.get_u8(); // Reserved
        frame.get_u8(); // Reserved

        let link_quality_indication = frame.get_u8();

        frame.get_u8(); // Reserved
        frame.get_u8(); // Reserved
        frame.get_u8(); // Reserved
        frame.get_u8(); // Reserved

        let received_signal_strength_indication = frame.get_i8();

        (
            Self {
                device_state,
                destination_address,
                destination_endpoint,
                source_address,
                source_endpoint,
                profile_id,
                cluster_id,
                application_specific_data_unit,
                link_quality_indication,
                received_signal_strength_indication,
            },
            Some(device_state),
        )
    }
}
