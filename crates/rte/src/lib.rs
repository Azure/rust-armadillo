#[macro_use]
pub mod utils;
pub mod acl;
pub mod ethdev;
pub mod ether;
pub mod launch;
pub mod lcore;
pub mod mbuf;
pub mod mempool;
pub mod timer;

type Result<T, E = rte_error::Error> = std::result::Result<T, E>;
