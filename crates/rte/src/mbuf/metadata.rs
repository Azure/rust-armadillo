use std::{marker::PhantomData, ptr::NonNull};

use ffi::_bindgen_ty_13::{RTE_MBUF_L2_LEN_BITS, RTE_MBUF_L3_LEN_BITS};

use super::ptr::AsPtr;
use crate::flags::PktTxOffload;

/// A struct that only allows running [`MetadataExt`] methods on an [`MBuf`].
///
/// This struct does not allow modifying the mbuf in any way that would invalidate its current underlying buffer.
///
/// See: [`MBuf::split_metadata_mut`]
///
/// [`MBuf`]: super::MBuf
/// [`MBuf::split_metadata_mut`]: super::MBuf::split_metadata_mut
#[repr(transparent)]
#[derive(Debug)]
pub struct MetadataPart<'a> {
    pub(super) ptr: NonNull<ffi::rte_mbuf>,
    pub(super) _marker: PhantomData<&'a mut ()>,
}

pub trait MetadataExt: AsPtr {
    /// Sets the [`l2_len`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#aa25a7c259438b9eba28bcedc33846620) field.
    #[inline]
    fn set_l2_len(&mut self, len: u64) {
        assert!(len < 1 << RTE_MBUF_L2_LEN_BITS);
        unsafe {
            let mbuf = self.as_ptr().as_mut();
            mbuf.__bindgen_anon_3.__bindgen_anon_1.set_l2_len(len);
        }
    }

    /// Sets the [`l3_len`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#a82a34cb6d5935a8c0f043f2783d6b42d) field.
    #[inline]
    fn set_l3_len(&mut self, len: u64) {
        assert!(len < 1 << RTE_MBUF_L3_LEN_BITS);
        unsafe {
            let mbuf = self.as_ptr().as_mut();
            mbuf.__bindgen_anon_3.__bindgen_anon_1.set_l3_len(len);
        }
    }

    /// Enables (bitwise-or) the given flags on the [`ol_flags`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#a319d580a6e1ef13692631d7b0d6d5c98) field.
    ///
    /// See also: [`PktTxOffload`].
    #[inline]
    fn enable_ol_flags(&mut self, flags: PktTxOffload) {
        unsafe {
            let mbuf = self.as_ptr().as_mut();
            mbuf.ol_flags |= flags.bits();
        }
    }
}

impl<M> MetadataExt for M where M: AsPtr {}
