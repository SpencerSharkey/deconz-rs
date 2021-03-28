use std::fmt::Debug;
use std::{any::type_name, marker::PhantomData};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::deconz::DeconzFrame;

use super::{CommandId, DeconzCommand, DeconzCommandRequest, DeconzCommandResponse};

mod sealed {
    pub trait Sealed {}
}

use sealed::Sealed;

pub trait Paramater: Sealed + Send + 'static {
    // todo: should we enumify this?
    const PARAMATER_ID: u8;

    fn from_frame(frame: DeconzFrame<Bytes>) -> Self;
}

mod paramater {
    use std::time::Duration;

    use bytes::Buf;

    use super::{Bytes, DeconzFrame, Paramater, Sealed};

    #[derive(Debug)]
    pub struct MacAddress(u64);

    impl Sealed for MacAddress {}
    impl Paramater for MacAddress {
        const PARAMATER_ID: u8 = 0x01;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
        }
    }

    #[derive(Debug)]
    pub struct NetworkPanId(u16);

    impl Sealed for NetworkPanId {}
    impl Paramater for NetworkPanId {
        const PARAMATER_ID: u8 = 0x05;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u16_le())
        }
    }

    #[derive(Debug)]
    pub struct NetworkAddress(u16);

    impl Sealed for NetworkAddress {}
    impl Paramater for NetworkAddress {
        const PARAMATER_ID: u8 = 0x07;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u16_le())
        }
    }

    #[derive(Debug)]
    pub struct NetworkExtendedPanId(u64);

    impl Sealed for NetworkExtendedPanId {}
    impl Paramater for NetworkExtendedPanId {
        const PARAMATER_ID: u8 = 0x08;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum APSDesignatedCoordinator {
        Coordinator = 0x01,
        Router = 0x00,
    }

    impl Sealed for APSDesignatedCoordinator {}
    impl Paramater for APSDesignatedCoordinator {
        const PARAMATER_ID: u8 = 0x09;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            match frame.get_u8() {
                x if x == Self::Coordinator as u8 => Self::Coordinator,
                x if x == Self::Router as u8 => Self::Router,
                x => panic!("Unexpected coordinator type: {:?}", x),
            }
        }
    }

    pub struct ChannelMask(u32);
    impl Sealed for ChannelMask {}
    impl Paramater for ChannelMask {
        const PARAMATER_ID: u8 = 0x0A;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u32_le())
        }
    }

    pub struct APSExtendedPanId(u64);
    impl Sealed for APSExtendedPanId {}
    impl Paramater for APSExtendedPanId {
        const PARAMATER_ID: u8 = 0x0B;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
        }
    }

    pub struct TrustCenterAddress(u64);
    impl Sealed for TrustCenterAddress {}
    impl Paramater for TrustCenterAddress {
        const PARAMATER_ID: u8 = 0x0E;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u64_le())
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
    impl Paramater for SecurityMode {
        const PARAMATER_ID: u8 = 0x10;

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
    }

    #[derive(Debug, PartialEq)]
    pub enum PredefinedNetworkPanId {
        /// The [`NetworkPanId`] will be selected or obtained dynamically.
        NotPredefined = 0x00,
        /// The value of the paramater [`NetworkPanId`] will be used to join or form a network.
        Predefined = 0x01,
    }
    impl Sealed for PredefinedNetworkPanId {}
    impl Paramater for PredefinedNetworkPanId {
        const PARAMATER_ID: u8 = 0x15;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            match frame.get_u8() {
                x if x == Self::NotPredefined as u8 => Self::NotPredefined,
                x if x == Self::Predefined as u8 => Self::Predefined,
                x => panic!("Unexpected security mode: {:?}", x),
            }
        }
    }

    #[derive(Debug)]
    pub enum NetworkKey {
        Unset,
        Set([u8; 16]),
    }
    impl Sealed for NetworkKey {}
    impl Paramater for NetworkKey {
        const PARAMATER_ID: u8 = 0x18;

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
    }

    // LinkKey - skipped because it doesnt look to be useful?

    pub struct CurrentChannel(u8);
    impl Sealed for CurrentChannel {}
    impl Paramater for CurrentChannel {
        const PARAMATER_ID: u8 = 0x1C;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u8())
        }
    }

    pub struct ProtocolVersion(u16);
    impl Sealed for ProtocolVersion {}
    impl Paramater for ProtocolVersion {
        const PARAMATER_ID: u8 = 0x22;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u16_le())
        }
    }
    pub struct NetworkUpdateId(u8);
    impl Sealed for NetworkUpdateId {}
    impl Paramater for NetworkUpdateId {
        const PARAMATER_ID: u8 = 0x24;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u8())
        }
    }
    pub struct WatchdogTtl(Duration);
    impl Sealed for WatchdogTtl {}
    impl Paramater for WatchdogTtl {
        const PARAMATER_ID: u8 = 0x26;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(Duration::from_secs(frame.get_u32_le() as _))
        }
    }

    pub struct NetworkFrameCounter(u32);
    impl Sealed for NetworkFrameCounter {}
    impl Paramater for NetworkFrameCounter {
        const PARAMATER_ID: u8 = 0x27;

        fn from_frame(mut frame: DeconzFrame<Bytes>) -> Self {
            Self(frame.get_u32_le())
        }
    }
}

pub type ReadMacAddress = ReadParameter<paramater::MacAddress>;
pub type ReadNetworkPanId = ReadParameter<paramater::NetworkPanId>;
pub type ReadNetworkAddress = ReadParameter<paramater::NetworkAddress>;
pub type ReadAPSDesignatedCoordinator = ReadParameter<paramater::APSDesignatedCoordinator>;
pub type ReadChannelMask = ReadParameter<paramater::ChannelMask>;
pub type ReadAPSExtendedPanId = ReadParameter<paramater::APSExtendedPanId>;
pub type ReadTrustCenterAddress = ReadParameter<paramater::TrustCenterAddress>;
pub type ReadSecurityMode = ReadParameter<paramater::SecurityMode>;
pub type ReadPredefinedNetworkPanId = ReadParameter<paramater::PredefinedNetworkPanId>;
pub type ReadNetworkKey = ReadParameter<paramater::NetworkKey>;
pub type ReadCurrentChannel = ReadParameter<paramater::CurrentChannel>;
pub type ReadProtocolVersion = ReadParameter<paramater::ProtocolVersion>;
pub type ReadNetworkUpdateId = ReadParameter<paramater::NetworkUpdateId>;
pub type ReadWatchdogTtl = ReadParameter<paramater::WatchdogTtl>;
pub type ReadNetworkFrameCounter = ReadParameter<paramater::NetworkFrameCounter>;

pub struct ReadParameter<T: Paramater> {
    _phantom: PhantomData<T>,
}

impl<T: Paramater> ReadParameter<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

pub struct ReadParameterRequest<T: Paramater> {
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct ReadParameterResponse<T: Paramater> {
    pub value: T,
}

impl<T: Paramater> Debug for ReadParameterRequest<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadParamaterRequest")
            .field("paramater", &type_name::<T>())
            .finish()
    }
}

impl<T: Paramater> DeconzCommand for ReadParameter<T> {
    type Request = ReadParameterRequest<T>;
    type Response = ReadParameterResponse<T>;

    fn into_request(self) -> Self::Request {
        Self::Request {
            _phantom: PhantomData,
        }
    }
}

impl<T: Paramater> DeconzCommandRequest for ReadParameterRequest<T> {
    fn command_id(&self) -> CommandId {
        CommandId::ReadParameter
    }

    fn payload_data(&self) -> bytes::BytesMut {
        let mut payload = BytesMut::new();
        payload.put_u8(T::PARAMATER_ID);
        payload
    }
}

impl<T: Paramater> DeconzCommandResponse for ReadParameterResponse<T> {
    fn from_frame(mut frame: DeconzFrame<bytes::Bytes>) -> Self {
        // todo: how do we handle this better? what if it doesn't match? should we crash?
        let _payload_length = frame.get_u16_le();
        let paramater_id = dbg!(frame.get_u8());
        assert_eq!(paramater_id, T::PARAMATER_ID);
        Self {
            value: T::from_frame(frame),
        }
    }
}

impl<T: Paramater> ReadParameterResponse<T> {
    pub fn into_inner(self) -> T {
        self.value
    }
}
