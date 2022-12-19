use std::ptr::NonNull;

use super::{metadata::MetadataPart, Allocator, MBuf};

pub trait AsPtr {
    fn as_ptr(&self) -> NonNull<ffi::rte_mbuf>;
}

impl<A: Allocator> AsPtr for MBuf<A> {
    #[inline]
    fn as_ptr(&self) -> NonNull<ffi::rte_mbuf> {
        self.ptr
    }
}

impl AsPtr for MetadataPart<'_> {
    #[inline]
    fn as_ptr(&self) -> NonNull<ffi::rte_mbuf> {
        self.ptr
    }
}
