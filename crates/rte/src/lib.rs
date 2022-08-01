#[cfg(test)]
extern crate self as rte;

pub mod ethdev;
pub mod launch;
pub mod lcore;
pub mod mbuf;
pub mod mempool;
pub mod timer;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

type Result<T, E = rte_error::Error> = std::result::Result<T, E>;

#[inline]
pub fn get_tsc_hz() -> u64 {
    unsafe { ffi::rte_get_tsc_hz() }
}

#[inline]
pub fn get_tsc_cycles() -> u64 {
    unsafe { ffi::_rte_get_tsc_cycles() }
}
