use std::{fmt, ops::Deref};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub struct Id(u32);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if **self == u32::MAX {
            f.write_str("no_lcore")
        } else {
            <u32 as fmt::Display>::fmt(&*self, f)
        }
    }
}

pub fn id(id: u32) -> Id {
    Id(id)
}

impl<T: Into<u32>> From<T> for Id {
    fn from(id: T) -> Self {
        Id(id.into())
    }
}

impl Deref for Id {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Id {
    /// Test if an lcore is enabled.
    pub fn is_enabled(self) -> bool {
        unsafe { ffi::rte_lcore_is_enabled(self.0) != 0 }
    }

    pub fn is_main(self) -> bool {
        unsafe { self.0 == ffi::rte_get_main_lcore() }
    }
}

/// Return the ID of the execution unit we are running on.
#[inline]
pub fn current() -> Id {
    unsafe { ffi::_rte_lcore_id() }.into()
}

/// All the enabled lcores.
pub fn enabled(skip_main: bool) -> Vec<Id> {
    foreach_lcores(skip_main).collect()
}

pub fn main_core() -> Id {
    unsafe { ffi::rte_get_main_lcore() }.into()
}

pub type SocketId = i32;

pub const SOCKET_ID_ANY: SocketId = -1;

/// Return the ID of the physical socket of the logical core we are running on.
pub fn socket_id() -> SocketId {
    unsafe { ffi::rte_socket_id() as SocketId }
}

/// Return number of physical sockets detected on the system.
///
/// Note that number of nodes may not be correspondent to their physical id's:
/// for example, a system may report two socket id's, but the actual socket id's
/// may be 0 and 8.
pub fn socket_count() -> u32 {
    unsafe { ffi::rte_socket_count() }
}

pub fn lcore_count() -> u32 {
    unsafe { ffi::rte_lcore_count() }
}

pub fn foreach_lcores(skip_main: bool) -> impl Iterator<Item = Id> {
    (0..ffi::RTE_MAX_LCORE)
        .map(Id)
        .filter(|lcore_id| lcore_id.is_enabled())
        .filter(move |lcore_id| !skip_main || !lcore_id.is_main())
}
