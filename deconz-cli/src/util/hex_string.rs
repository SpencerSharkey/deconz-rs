use std::str::FromStr;

use hex::{FromHex, FromHexError};

#[derive(Debug)]
pub(crate) struct HexString<T>(T);

macro_rules! hex_string_impl {
    ($int:ty, $bytes:expr) => {
        impl FromStr for HexString<$int> {
            type Err = FromHexError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let hex = <[u8; $bytes]>::from_hex(s.trim_start_matches("0x"))?;
                Ok(Self(<$int>::from_be_bytes(hex)))
            }
        }
    };
}

impl<T> std::ops::Deref for HexString<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

hex_string_impl!(u16, 2);
hex_string_impl!(u32, 4);
hex_string_impl!(u64, 8);

impl FromStr for HexString<[u8; 16]> {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(<[u8; 16]>::from_hex(s)?))
    }
}
