use std::ptr::NonNull;

use rte_error::ReturnValue as _;

use crate::{mempool::MemoryPool, Result};

/// Trait for describing types that can be used as allocators for [`MBuf`](super::MBuf)s.
///
/// This trait is implemented by the following types:
/// - **[`MemoryPool`]**: the default implementation that uses an RTE `MemoryPool` and its pertaining library functions to manage mbuf allocations.
#[cfg_attr(
    any(test, feature = "test-utils"),
    doc = "- **[`GlobalAllocatorPool`]**: uses the default Rust global allocator to allocate `MBufs` using the OS's native heap capabilities: useful for testing code that depends on mbufs, but without depending on the RTE EAL."
)]
pub trait Allocator {
    fn alloc(&self) -> Result<NonNull<ffi::rte_mbuf>>;

    /// # Safety
    /// The caller must ensure `mbuf` points to a valid [`ffi::rte_mbuf`] that was allocated using [`Self::alloc`].
    unsafe fn clone(mbuf: NonNull<ffi::rte_mbuf>) -> Result<NonNull<ffi::rte_mbuf>>;

    /// # Safety
    /// The caller must ensure `mbuf` points to a valid [`ffi::rte_mbuf`] that was allocated using [`Self::alloc`],
    /// and that the pointer is not used after this function has returned.
    unsafe fn free(mbuf: NonNull<ffi::rte_mbuf>);
}

impl<'a> Allocator for &'a MemoryPool {
    fn alloc(&self) -> Result<NonNull<ffi::rte_mbuf>> {
        unsafe { ffi::_rte_pktmbuf_alloc(self.0.as_ptr()) }.rte_ok()
    }

    /// Clones this `MBuf` using [`rte_pktmbuf_copy`](ffi::rte_pktmbuf_copy).
    ///
    /// Notice that this creates a "deep" clone, including allocation a new data buffer and copying this buffer's contents over.
    ///
    /// See also: <http://doc.dpdk.org/api-21.08/rte__mbuf_8h.html#a04f6ba3f0f9afe72e21e3a3f8908e6ae>
    ///
    /// # Implementation notes
    /// Originally, the `Clone` implementation used [`rte_pktmbuf_clone`](http://doc.dpdk.org/api-21.08/rte__mbuf_8h.html#a5f1a5320fb96ff8c1a44be0aaec93856) to create
    /// a shallow clone (i.e. one where the original and the clone share the same underlying data buffer).
    ///
    /// While a shallow clone is cheaper, it allows violating Rust borrow checker rules, by allowing safe code to create non-mutually-exclusive references to the same memory buffer.
    unsafe fn clone(mbuf: NonNull<ffi::rte_mbuf>) -> Result<NonNull<ffi::rte_mbuf>> {
        let mbuf = mbuf.as_ptr();
        let ffi::rte_mbuf { data_len, pool, .. } = *mbuf;
        ffi::rte_pktmbuf_copy(mbuf, pool, 0, data_len.into()).rte_ok()
    }

    unsafe fn free(mbuf: NonNull<ffi::rte_mbuf>) {
        ffi::_rte_pktmbuf_free(mbuf.as_ptr());
    }
}

#[cfg(any(test, feature = "test-utils"))]
mod test_pool {
    use std::{
        alloc::{alloc, alloc_zeroed, dealloc, Layout},
        ptr::{self, NonNull},
    };

    use super::Allocator;
    use crate::Result;

    /// A struct implementing the [`Allocator`] trait using the [global Rust allocator](https://doc.rust-lang.org/stable/std/alloc/index.html).
    ///
    /// Allows testing code that uses [`MBuf`](crate::mbuf::MBuf)s without having to rely on the RTE memory pool, and without having to initialize the EAL.
    #[derive(Default, Clone, Copy)]
    pub struct GlobalAllocator<const BUF_SIZE: usize = { ffi::RTE_MBUF_DEFAULT_BUF_SIZE as usize }>;

    impl<const BUF_SIZE: usize> GlobalAllocator<BUF_SIZE> {
        fn data_layout() -> Layout {
            Layout::array::<u8>(BUF_SIZE).unwrap()
        }
    }

    impl<const BUF_SIZE: usize> Allocator for GlobalAllocator<BUF_SIZE> {
        fn alloc(&self) -> Result<NonNull<ffi::rte_mbuf>> {
            unsafe {
                let data = alloc(Self::data_layout()) as _;
                let mut mbuf =
                    NonNull::new(alloc_zeroed(Layout::new::<ffi::rte_mbuf>()) as *mut ffi::rte_mbuf).unwrap();
                {
                    let mbuf = mbuf.as_mut();
                    mbuf.buf_addr = data;
                    mbuf.buf_len = BUF_SIZE as u16;
                    mbuf.ol_flags &= ffi::EXT_ATTACHED_MBUF;
                    mbuf.port = ffi::RTE_MBUF_PORT_INVALID as u16;
                }

                Ok(mbuf)
            }
        }

        unsafe fn clone(mbuf: NonNull<ffi::rte_mbuf>) -> Result<NonNull<ffi::rte_mbuf>> {
            let mut clone = Self::alloc(&Self)?;

            {
                let clone = clone.as_mut();
                let mbuf = mbuf.as_ref();

                ptr::copy_nonoverlapping(mbuf.buf_addr, clone.buf_addr, mbuf.data_len.into());
                clone.data_len = mbuf.data_len;
                clone.pkt_len = mbuf.pkt_len;
            }

            Ok(clone)
        }

        unsafe fn free(mbuf: NonNull<ffi::rte_mbuf>) {
            dealloc(mbuf.as_ref().buf_addr as _, Self::data_layout());
            dealloc(mbuf.as_ptr() as _, Layout::new::<ffi::rte_mbuf>());
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub use self::test_pool::GlobalAllocator;
