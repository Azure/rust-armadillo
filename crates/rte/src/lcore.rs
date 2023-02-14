use std::{fmt, iter::successors};

use crate::memory::SocketId;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub struct Id(u32);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.get() == ffi::LCORE_ID_ANY {
            f.write_str("any lcore")
        } else {
            <u32 as fmt::Display>::fmt(&self.get(), f)
        }
    }
}

impl Id {
    #[inline]
    pub fn new(id: u32) -> Self {
        Id(id)
    }

    #[inline]
    pub fn get(&self) -> u32 {
        self.0
    }

    /// See also: <https://doc.dpdk.org/api-21.08/rte__lcore_8h.html#a5404ee6ac26cbe5a4f4ddef44d690b76>
    #[inline]
    pub fn is_enabled(self) -> bool {
        unsafe { ffi::rte_lcore_is_enabled(self.0) != 0 }
    }

    #[inline]
    pub fn is_main(self) -> bool {
        self == main()
    }

    /// See also: <https://doc.dpdk.org/api-21.08/rte__lcore_8h.html#acab656f5b00c29090db4500efabedd98>
    fn get_next(self, skip_main: bool, wrap: bool) -> Id {
        Id::new(unsafe { ffi::rte_get_next_lcore(self.0, skip_main.into(), wrap.into()) })
    }

    /// Based on [RTE_LCORE_FOREACH](https://doc.dpdk.org/api-21.08/rte__lcore_8h.html#a034c95b6412f09e8de11d430267dc1ba)
    #[inline]
    pub fn iter_enabled(skip_main: bool) -> impl Iterator<Item = Id> {
        const MAX_ID: Id = Id(ffi::RTE_MAX_LCORE);
        const LAST_ID: Id = Id(u32::MAX); // DPDK expects -1 for wrapping

        successors(Some(LAST_ID), move |last_id| {
            let next = last_id.get_next(skip_main, false);
            if next < MAX_ID {
                Some(next)
            } else {
                None
            }
        })
        .skip(1) // ignore the first element which is LAST_ID
    }
}

/// See also: <https://doc.dpdk.org/api-21.08/rte__lcore_8h.html#adfb2b334e7e73f534f25e8888a8a775f>
#[inline]
pub fn current() -> Id {
    Id::new(unsafe { ffi::_rte_lcore_id() })
}

/// See also: <https://doc.dpdk.org/api-21.08/rte__lcore_8h.html#a5449c6ee062fe3641520374152ce6c67>
#[inline]
pub fn main() -> Id {
    Id::new(unsafe { ffi::rte_get_main_lcore() })
}

/// See also: <https://doc.dpdk.org/api-21.08/rte__lcore_8h.html#a1728dc7f14571ba778d3b5b41aa09283>
#[inline]
pub fn count() -> u32 {
    unsafe { ffi::rte_lcore_count() }
}

/// See also: <https://doc.dpdk.org/api-21.08/rte__lcore_8h.html#a7c8da4664df26a64cf05dc508a4f26df>
#[inline]
pub fn socket_id() -> Option<SocketId> {
    SocketId::new(unsafe { ffi::rte_socket_id() })
}
