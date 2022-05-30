use std::{
    fmt,
    ops::{Deref, DerefMut},
    result, str,
};

use zerocopy::{AsBytes, FromBytes, Unaligned};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("invalid MAC address syntax")]
pub struct AddrParseError(());

pub const ETHER_ADDR_LEN: u8 = 6;
type MacAddrBuf = [u8; ETHER_ADDR_LEN as usize];

/// A 48-bit (6 byte) buffer containing the MAC address
#[derive(Debug, FromBytes, AsBytes, Unaligned, Copy, Clone, Default, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct MacAddr(MacAddrBuf);

impl Deref for MacAddr {
    type Target = MacAddrBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MacAddr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MacAddr {
    /// Creates a new MAC address from six eight-bit octets.
    ///
    /// The result will represent the MAC address a:b:c:d:e:f.
    #[allow(clippy::many_single_char_names)]
    #[inline]
    pub const fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> MacAddr {
        MacAddr([a, b, c, d, e, f])
    }

    /// Returns the six eight-bit integers that make up this address.
    #[inline]
    pub const fn octets(&self) -> MacAddrBuf {
        self.0
    }

    #[inline]
    pub fn from_bytes(b: &[u8]) -> result::Result<Self, AddrParseError> {
        b.try_into().map(Self).map_err(|_| AddrParseError(()))
    }

    #[inline]
    pub fn zeroed() -> Self {
        Self::default()
    }

    /// Check if an Ethernet address is filled with zeros.
    #[inline]
    pub fn is_zero(&self) -> bool {
        *self == Self::zeroed()
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl From<MacAddrBuf> for MacAddr {
    fn from(addr: MacAddrBuf) -> MacAddr {
        MacAddr(addr)
    }
}

impl str::FromStr for MacAddr {
    type Err = AddrParseError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let addr: Vec<u8> = s
            .split(':')
            .map(|part| u8::from_str_radix(part, 16))
            .collect::<result::Result<_, _>>()
            .map_err(|_| AddrParseError(()))?;

        MacAddr::from_bytes(addr.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_macaddr() {
        let addr = MacAddr::new(0x18, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f);

        assert_eq!(addr.octets(), [0x18, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]);
        assert_eq!(addr.to_string(), "18:2b:3c:4d:5e:6f");

        assert_eq!(addr, MacAddr::from([0x18, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]));
        assert_eq!(addr, MacAddr::from_str("18:2b:3c:4d:5e:6f").unwrap());

        MacAddr::from_str("18:2b:3c:4d:5e:6f:XX").unwrap_err();

        assert!(!addr.is_zero());
        assert!(MacAddr::zeroed().is_zero());
    }
}
