mod data_indication;

pub use data_indication::{ReadReceivedData, ReadReceivedDataRequest, ReadReceivedDataResponse};

#[derive(Debug)]
pub enum DestinationAddress {
    GroupAddress(u16),
    NetworkAddress(u16),
    IEEEAddress(u64),
}

#[derive(Debug)]
pub enum SourceAddress {
    NetworkAddress(u16),
    IEEEAddress(u64),
    Both {
        network_address: u16,
        ieee_address: u64,
    },
}
