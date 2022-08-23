#[cfg(test)]
extern crate self as rte;

pub mod ethdev;
pub mod flags;
pub mod launch;
pub mod lcore;
pub mod mbuf;
pub mod memory;
pub mod mempool;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

type Result<T, E = rte_error::Error> = std::result::Result<T, E>;
