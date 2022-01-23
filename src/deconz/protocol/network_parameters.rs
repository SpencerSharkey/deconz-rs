use std::fmt::Debug;
use std::{any::type_name, marker::PhantomData};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::deconz::DeconzFrame;

use super::{
    device::DeviceState, CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse,
};

mod sealed {
    pub trait Sealed {}
}

use sealed::Sealed;

pub trait Parameter: Debug + Sealed + Send + 'static {
    // todo: should we enumify this?
    const PARAMETER_ID: u8;

    fn from_frame(frame: DeconzFrame<Bytes>) -> Self;

    fn write_frame(&self, payload: &mut BytesMut);
}

mod parameters {
    use std::{fmt::Display, ops::Deref, time::Duration};

    use bytes::{Buf, BufMut};

    use super::{Bytes, DeconzFrame, Parameter, Sealed};

    #[derive(Debug)]
    pub struct MacAddress(u64);

    impl Sealed for MacAddress {}
    impl Parameter for MacAddress {
        const PARAMETER_ID: u8 = 0x01;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u64_le(self.0);
        }
    }

    impl Deref for MacAddress {
        type Target = u64;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for MacAddress {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let bytes = self.0.to_be_bytes();
            write!(
                f,
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
            )
        }
    }

    impl From<u64> for MacAddress {
        fn from(v: u64) -> Self {
            Self(v)
        }
    }

    #[derive(Debug)]
    pub struct NetworkPanId(u16);

    impl Sealed for NetworkPanId {}
    impl Parameter for NetworkPanId {
        const PARAMETER_ID: u8 = 0x05;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u16_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u16_le(self.0)
        }
    }

    impl std::ops::Deref for NetworkPanId {
        type Target = u16;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for NetworkPanId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "0x{:04x}", self.0)
        }
    }

    #[derive(Debug)]
    pub struct NetworkAddress(u16);

    impl Sealed for NetworkAddress {}
    impl Parameter for NetworkAddress {
        const PARAMETER_ID: u8 = 0x07;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u16_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u16_le(self.0)
        }
    }

    impl std::ops::Deref for NetworkAddress {
        type Target = u16;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for NetworkAddress {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "0x{:04x}", self.0)
        }
    }
    #[derive(Debug)]
    pub struct NetworkExtendedPanId(u64);

    impl Sealed for NetworkExtendedPanId {}
    impl Parameter for NetworkExtendedPanId {
        const PARAMETER_ID: u8 = 0x08;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u64_le(self.0)
        }
    }

    impl std::ops::Deref for NetworkExtendedPanId {
        type Target = u64;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for NetworkExtendedPanId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "0x{:16x}", self.0)
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum APSDesignatedCoordinator {
        Coordinator = 0x01,
        Router = 0x00,
    }

    impl Sealed for APSDesignatedCoordinator {}
    impl Parameter for APSDesignatedCoordinator {
        const PARAMETER_ID: u8 = 0x09;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            match frame.get_u8() {
                x if x == Self::Coordinator as u8 => Self::Coordinator,
                x if x == Self::Router as u8 => Self::Router,
                x => panic!("Unexpected coordinator type: {:?}", x),
            }
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u8((self as *const APSDesignatedCoordinator) as u8)
        }
    }

    impl Display for APSDesignatedCoordinator {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let repr_str = match self {
                APSDesignatedCoordinator::Coordinator => "Coodinator",
                APSDesignatedCoordinator::Router => "Router",
            };
            write!(f, "{}", repr_str)
        }
    }

    #[derive(Debug)]
    pub struct ChannelMask(u32);

    impl Sealed for ChannelMask {}
    impl Parameter for ChannelMask {
        const PARAMETER_ID: u8 = 0x0A;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u32_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u32_le(self.0)
        }
    }

    impl std::ops::Deref for ChannelMask {
        type Target = u32;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<u32> for ChannelMask {
        fn from(v: u32) -> Self {
            Self(v)
        }
    }

    #[derive(Debug)]
    pub struct APSExtendedPanId(u64);

    impl Sealed for APSExtendedPanId {}
    impl Parameter for APSExtendedPanId {
        const PARAMETER_ID: u8 = 0x0B;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u64_le(self.0);
        }
    }

    impl std::ops::Deref for APSExtendedPanId {
        type Target = u64;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for APSExtendedPanId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "0x{:016x}", self.0)
        }
    }

    impl From<u64> for APSExtendedPanId {
        fn from(v: u64) -> Self {
            APSExtendedPanId(v)
        }
    }

    #[derive(Debug)]
    pub struct TrustCenterAddress(u64);

    impl Sealed for TrustCenterAddress {}
    impl Parameter for TrustCenterAddress {
        const PARAMETER_ID: u8 = 0x0E;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u64_le(self.0);
        }
    }

    impl std::ops::Deref for TrustCenterAddress {
        type Target = u64;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for TrustCenterAddress {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "0x{:016x}", self.0)
        }
    }

    impl From<u64> for TrustCenterAddress {
        fn from(v: u64) -> Self {
            Self(v)
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum SecurityMode {
        NoSecurity = 0x00,
        PreconfiguredNetworkKey = 0x01,
        NetworkKeyFromTrustCenter = 0x02,
        NoMasterButTrustCenterLinkKey = 0x03,
    }

    impl Sealed for SecurityMode {}
    impl Parameter for SecurityMode {
        const PARAMETER_ID: u8 = 0x10;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            match frame.get_u8() {
                x if x == Self::NoSecurity as u8 => Self::NoSecurity,
                x if x == Self::PreconfiguredNetworkKey as u8 => Self::PreconfiguredNetworkKey,
                x if x == Self::NetworkKeyFromTrustCenter as u8 => Self::NetworkKeyFromTrustCenter,
                x if x == Self::NoMasterButTrustCenterLinkKey as u8 => {
                    Self::NoMasterButTrustCenterLinkKey
                }
                x => panic!("Unexpected security mode: {:?}", x),
            }
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u8((self as *const SecurityMode) as u8);
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum PredefinedNetworkPanId {
        /// The [`NetworkPanId`] will be selected or obtained dynamically.
        NotPredefined = 0x00,
        /// The value of the paramater [`NetworkPanId`] will be used to join or form a network.
        Predefined = 0x01,
    }
    impl Sealed for PredefinedNetworkPanId {}
    impl Parameter for PredefinedNetworkPanId {
        const PARAMETER_ID: u8 = 0x15;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            match frame.get_u8() {
                x if x == Self::NotPredefined as u8 => Self::NotPredefined,
                x if x == Self::Predefined as u8 => Self::Predefined,
                x => panic!("Unexpected security mode: {:?}", x),
            }
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u8((self as *const PredefinedNetworkPanId) as u8);
        }
    }

    #[derive(Debug)]
    pub enum NetworkKey {
        Unset,
        Set([u8; 16]),
    }
    impl Sealed for NetworkKey {}
    impl Parameter for NetworkKey {
        const PARAMETER_ID: u8 = 0x18;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            // in testing, I found that if the frame is empty, then perhaps
            // there is no network key?
            if frame.has_remaining() {
                let mut buf = [0u8; 16];
                frame.copy_to_slice(&mut buf);
                Self::Set(buf)
            } else {
                Self::Unset
            }
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_slice(&match self {
                NetworkKey::Unset => [0; 16],
                NetworkKey::Set(buf) => *buf,
            })
        }
    }

    // LinkKey - skipped because it doesnt look to be useful?

    #[derive(Debug)]
    pub struct CurrentChannel(u8);

    impl Sealed for CurrentChannel {}
    impl Parameter for CurrentChannel {
        const PARAMETER_ID: u8 = 0x1C;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u8())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u8(self.0);
        }
    }

    impl std::ops::Deref for CurrentChannel {
        type Target = u8;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    pub struct ProtocolVersion(u16);

    impl Sealed for ProtocolVersion {}
    impl Parameter for ProtocolVersion {
        const PARAMETER_ID: u8 = 0x22;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u16_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u16_le(self.0);
        }
    }

    impl std::ops::Deref for ProtocolVersion {
        type Target = u16;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    #[derive(Debug)]
    pub struct NetworkUpdateId(u8);

    impl Sealed for NetworkUpdateId {}
    impl Parameter for NetworkUpdateId {
        const PARAMETER_ID: u8 = 0x24;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u8())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u8(self.0);
        }
    }

    impl std::ops::Deref for NetworkUpdateId {
        type Target = u8;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<u8> for NetworkUpdateId {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }

    #[derive(Debug)]
    pub struct WatchdogTtl(Duration);
    impl Sealed for WatchdogTtl {}
    impl Parameter for WatchdogTtl {
        const PARAMETER_ID: u8 = 0x26;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(Duration::from_secs(if frame.len() < 1 {
                0
            } else {
                frame.get_u32_le().into()
            }))
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u32_le(self.0.as_secs() as u32)
        }
    }

    impl Deref for WatchdogTtl {
        type Target = Duration;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Display for WatchdogTtl {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0.as_secs_f32())
        }
    }

    impl From<Duration> for WatchdogTtl {
        fn from(v: Duration) -> Self {
            Self(v)
        }
    }

    #[derive(Debug)]
    pub struct NetworkFrameCounter(u32);

    impl Sealed for NetworkFrameCounter {}
    impl Parameter for NetworkFrameCounter {
        const PARAMETER_ID: u8 = 0x27;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u32_le())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u32_le(self.0);
        }
    }

    impl std::ops::Deref for NetworkFrameCounter {
        type Target = u32;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<u32> for NetworkFrameCounter {
        fn from(v: u32) -> Self {
            Self(v)
        }
    }

    #[derive(Debug)]
    pub struct PermitJoin(u8);
    impl Sealed for PermitJoin {}
    impl Parameter for PermitJoin {
        const PARAMETER_ID: u8 = 0x21;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u8())
        }

        fn write_frame(&self, payload: &mut bytes::BytesMut) {
            payload.put_u8(self.0)
        }
    }

    impl From<u8> for PermitJoin {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
}

pub type ReadMacAddress = ReadParameter<parameters::MacAddress>;
pub type ReadNetworkPanId = ReadParameter<parameters::NetworkPanId>;
pub type ReadNetworkAddress = ReadParameter<parameters::NetworkAddress>;
pub type ReadAPSDesignatedCoordinator = ReadParameter<parameters::APSDesignatedCoordinator>;
pub type ReadChannelMask = ReadParameter<parameters::ChannelMask>;
pub type ReadAPSExtendedPanId = ReadParameter<parameters::APSExtendedPanId>;
pub type ReadTrustCenterAddress = ReadParameter<parameters::TrustCenterAddress>;
pub type ReadSecurityMode = ReadParameter<parameters::SecurityMode>;
pub type ReadPredefinedNetworkPanId = ReadParameter<parameters::PredefinedNetworkPanId>;
pub type ReadNetworkKey = ReadParameter<parameters::NetworkKey>;
pub type ReadCurrentChannel = ReadParameter<parameters::CurrentChannel>;
pub type ReadProtocolVersion = ReadParameter<parameters::ProtocolVersion>;
pub type ReadNetworkUpdateId = ReadParameter<parameters::NetworkUpdateId>;
pub type ReadWatchdogTtl = ReadParameter<parameters::WatchdogTtl>;
pub type ReadNetworkFrameCounter = ReadParameter<parameters::NetworkFrameCounter>;
pub type ReadPermitJoin = ReadParameter<parameters::PermitJoin>;

pub type WriteNetworkPanId = WriteParameter<parameters::MacAddress>;
pub type WriteAPSDesignatedCoordinator = WriteParameter<parameters::APSDesignatedCoordinator>;
pub type WriteChannelMask = WriteParameter<parameters::ChannelMask>;
pub type WriteAPSExtendedPanId = WriteParameter<parameters::APSExtendedPanId>;
pub type WriteTrustCenterAddress = WriteParameter<parameters::TrustCenterAddress>;
pub type WriteSecurityMode = WriteParameter<parameters::SecurityMode>;
pub type WritePredefinedNetworkPanId = WriteParameter<parameters::PredefinedNetworkPanId>;
pub type WriteNetworkKey = WriteParameter<parameters::NetworkKey>;
// pub type WriteLinkKey = WriteParameter<LinkKey>;
pub type WriteNetworkUpdateId = WriteParameter<parameters::NetworkUpdateId>;
pub type WriteWatchdogTtl = WriteParameter<parameters::WatchdogTtl>;
pub type WriteNetworkFrameCounter = WriteParameter<parameters::NetworkFrameCounter>;
pub type WritePermitJoin = WriteParameter<parameters::PermitJoin>;

pub struct ReadParameter<T: Parameter> {
    _phantom: PhantomData<T>,
}

impl<T: Parameter> ReadParameter<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Parameter> Default for ReadParameter<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReadParameterRequest<T: Parameter> {
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct ReadParameterResponse<T: Parameter> {
    pub value: T,
}

impl<T: Parameter> Debug for ReadParameterRequest<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadParamaterRequest")
            .field("paramater", &type_name::<T>())
            .finish()
    }
}

impl<T: Parameter> DeconzCommand for ReadParameter<T> {
    type Request = ReadParameterRequest<T>;
    type Response = ReadParameterResponse<T>;

    fn into_request(self) -> Self::Request {
        Self::Request {
            _phantom: PhantomData,
        }
    }
}

impl<T: Parameter> DeconzCommandRequest for ReadParameterRequest<T> {
    fn command_id(&self) -> CommandId {
        CommandId::ReadParameter
    }

    fn payload_data(&self) -> Option<BytesMut> {
        let mut payload = BytesMut::new();
        payload.put_u8(T::PARAMETER_ID);
        Some(payload)
    }
}

impl<T: Parameter> DeconzCommandResponse for ReadParameterResponse<T> {
    fn from_frame(mut frame: DeconzFrame<bytes::Bytes>) -> (Self, Option<DeviceState>) {
        // todo: how do we handle this better? what if it doesn't match? should we crash?
        let _payload_length = frame.get_u16_le();
        let paramater_id = frame.get_u8();
        assert_eq!(paramater_id, T::PARAMETER_ID);
        (
            Self {
                value: T::from_frame(frame),
            },
            None,
        )
    }
}

impl<T: Parameter> ReadParameterResponse<T> {
    pub fn into_inner(self) -> T {
        self.value
    }
}

pub struct WriteParameter<T: Parameter> {
    value: T,
}

impl<T: Parameter> WriteParameter<T> {
    pub fn new(value: impl Into<T>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

pub struct WriteParameterRequest<T: Parameter> {
    pub value: T,
}

#[derive(Debug)]
pub struct WriteParameterResponse<T: Parameter> {
    _phantom: PhantomData<T>,
}

impl<T: Parameter> Debug for WriteParameterRequest<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteParameterRequest")
            .field("data", &self.value)
            .finish()
    }
}

impl<T: Parameter> DeconzCommand for WriteParameter<T> {
    type Request = WriteParameterRequest<T>;
    type Response = WriteParameterResponse<T>;

    fn into_request(self) -> Self::Request {
        Self::Request { value: self.value }
    }
}

impl<T: Parameter> DeconzCommandRequest for WriteParameterRequest<T> {
    fn command_id(&self) -> CommandId {
        CommandId::WriteParameter
    }

    fn payload_data(&self) -> Option<BytesMut> {
        let mut payload = BytesMut::new();
        payload.put_u8(T::PARAMETER_ID);
        T::write_frame(&self.value, &mut payload);
        Some(payload)
    }
}

impl<T: Parameter> DeconzCommandResponse for WriteParameterResponse<T> {
    fn from_frame(mut frame: DeconzFrame<bytes::Bytes>) -> (Self, Option<DeviceState>) {
        let _payload_length = frame.get_u16_le();
        let paramater_id = frame.get_u8();
        assert_eq!(paramater_id, T::PARAMETER_ID);
        (
            Self {
                _phantom: Default::default(),
            },
            None,
        )
    }
}

impl<T: Parameter + Default> WriteParameterResponse<T> {
    pub fn into_inner(self) -> T {
        T::default()
    }
}
