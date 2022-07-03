use std::{
    fmt,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

use ffi::_bindgen_ty_16::{RTE_MBUF_L2_LEN_BITS, RTE_MBUF_L3_LEN_BITS};
use rte_error::ReturnValue as _;

use crate::ethdev::PktTxOffloadHashFunc;

/// This struct is a Rust-y wrapper around a pointer to DPDK's [`rte_mbuf`](ffi::rte_mbuf) struct.
///
/// It mimics the behavior of the standard library's [`Box<[u8]>`](Box) in two important ways:
/// 1. It can be dereferenced into a `[u8]` slice to access the underlying buffer's data.
/// 2. **Ownership**: dropping this struct will call [`rte_pktmbuf_free`](ffi::_rte_pktmbuf_free) on the mbuf it points to,
///    similarly to how dropping a `Box` deallocates the memory it owns.
///
/// Otherwise, it implements a subset of the standard library's [`Vec<u8>`](Vec) API to allow manipulation of the buffer, and methods for manipulating the buffer's metadata using the DPDK library.
///
/// # Memory safety
/// An `MBuf` is, memory-wise, a transparent wrapper around the [`NonNull`] type, which means it can be safely transmuted with a raw pointer, `*mut rte_mbuf`, so long as that pointer is known to be non-zero.
///
/// # Thread safety
/// While in the future, it might be possible to mark `MBuf` as `Send`, right now it implements neither `Sync` nor `Send`,
/// which means it cannot be used in a threaded context in any way.
/// ```rust
/// # use static_assertions::assert_not_impl_any;
/// # use rte::mbuf::MBuf;
/// assert_not_impl_any!(MBuf: Send, Sync);
/// ```
///
/// # See also
/// - The DPDK documentation on the [Mbuf Library](https://doc.dpdk.org/guides-21.08/prog_guide/mbuf_lib.html).
///
/// # Implementation notes
/// - This wrapper completely ignores all but the first segment of an mbuf.
#[repr(transparent)]
pub struct MBuf(pub(crate) NonNull<ffi::rte_mbuf>);

impl MBuf {
    /// Returns the raw pointer to the `rte_mbuf` struct, pointed to by this `MBuf`.
    ///
    /// # Safety
    /// It is up to the caller to make sure to not use the raw pointer in a way that breaks the safety invariants the `MBuf` struct relies on.
    #[inline]
    pub unsafe fn as_raw(&self) -> *mut ffi::rte_mbuf {
        self.0.as_ptr()
    }

    /* Helper functions for Deref and DerefMut impls. */

    fn data_ptr(&self) -> *mut u8 {
        unsafe {
            let ffi::rte_mbuf { buf_addr, data_off, .. } = *self.0.as_ref();
            buf_addr.add(data_off.into()) as *mut _
        }
    }

    fn data_len(&self) -> usize {
        unsafe { self.0.as_ref() }.data_len.into()
    }
}

/// These method are equivilent to their [`Vec`] counterparts.
impl MBuf {
    fn capacity(&self) -> usize {
        unsafe {
            let ffi::rte_mbuf { data_off, buf_len, .. } = *self.0.as_ref();
            buf_len.checked_sub(data_off).unwrap_unchecked().into()
        }
    }

    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            let ffi::rte_mbuf { buf_addr, data_off, data_len, buf_len, .. } = *self.0.as_ref();
            let spare_cap = buf_addr.add(data_off.into()).add(data_len.into());
            let buf_end = spare_cap.add(buf_len.into());
            slice::from_raw_parts_mut(spare_cap as _, spare_cap.offset_from(buf_end) as usize)
        }
    }

    unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity());
        let mbuf = self.0.as_ptr();
        (*mbuf).data_len = len as u16;
        (*mbuf).pkt_len = len as u32;
    }

    /// See [`Vec::extend_from_slice`].
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[u8]) {
        // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout
        let uninit_other: &[MaybeUninit<u8>] = unsafe { mem::transmute(other) };

        self.spare_capacity_mut()[..other.len()].copy_from_slice(uninit_other);
        unsafe {
            self.set_len(self.len() + other.len());
        }
    }

    /// Extracts a slice containing the entire underlying buffer.
    ///
    /// Equivalent to `&mbuf[..]`.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self
    }

    /// Extracts a mutable slice of the entire vector.
    ///
    /// **TODO**: temporary hack, fix mbuf lifetimes <https://msazure.visualstudio.com/One/_workitems/edit/10357334>
    #[inline]
    pub fn as_mut_slice(&mut self) -> &'static mut [u8] {
        unsafe { mem::transmute::<&mut [u8], _>(self) }
    }
}

impl MBuf {
    /// Sets the [`l2_len`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#aa25a7c259438b9eba28bcedc33846620) field.
    #[inline]
    pub fn set_l2_len(&mut self, len: u64) {
        assert!(len < 1 << RTE_MBUF_L2_LEN_BITS);
        unsafe {
            let mbuf = self.0.as_mut();
            mbuf.__bindgen_anon_3.__bindgen_anon_1.set_l2_len(len);
        }
    }

    /// Sets the [`l3_len`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#a82a34cb6d5935a8c0f043f2783d6b42d) field.
    #[inline]
    pub fn set_l3_len(&mut self, len: u64) {
        assert!(len < 1 << RTE_MBUF_L3_LEN_BITS);
        unsafe {
            let mbuf = self.0.as_mut();
            mbuf.__bindgen_anon_3.__bindgen_anon_1.set_l3_len(len);
        }
    }

    /// Enables (bitwise-or) the given flags on the [`ol_flags`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#a319d580a6e1ef13692631d7b0d6d5c98) field.
    ///
    /// See also: [`PktTxOffloadHashFunc`].
    #[inline]
    pub fn enable_ol_flags(&mut self, flags: PktTxOffloadHashFunc) {
        unsafe {
            let mbuf = self.0.as_mut();
            mbuf.ol_flags |= flags.bits();
        }
    }
}

impl Drop for MBuf {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ffi::_rte_pktmbuf_free(self.as_raw());
        }
    }
}

impl Deref for MBuf {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.data_len()) }
    }
}

impl DerefMut for MBuf {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.data_ptr(), self.data_len()) }
    }
}

impl fmt::Debug for MBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <[u8] as fmt::Debug>::fmt(self, f)
    }
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
impl Clone for MBuf {
    #[track_caller]
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            let mbuf = self.0.as_ptr();
            let len = self.len() as u32;
            let mempool = (*mbuf).pool;
            ffi::rte_pktmbuf_copy(mbuf, mempool, 0, len).rte_ok().map(Self).expect("Failed to allocate mbuf clone")
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    use std::ops::{Deref, DerefMut};

    use ffi::RTE_MBUF_DEFAULT_BUF_SIZE;
    use uuid::Uuid;

    use super::MBuf;
    use crate::{
        lcore,
        mempool::{self, MemoryPool},
    };

    pub struct GeneratedMbufs {
        mbufs: Vec<MBuf>,
        _mbuf_pool: mempool::MemoryPool,
    }

    impl Deref for GeneratedMbufs {
        type Target = [MBuf];

        fn deref(&self) -> &Self::Target {
            &self.mbufs
        }
    }

    impl DerefMut for GeneratedMbufs {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.mbufs
        }
    }

    pub fn pool_create_from_bufs<B: AsRef<[u8]>>(bufs: &[B]) -> GeneratedMbufs {
        let memory_pool = {
            let mut uid_str = Uuid::new_v4().to_string();
            let id_len = uid_str.chars().count() - 10; // leave space for DPDK prefix
            uid_str.drain(0..id_len);

            // hack-y way of making sure benchmarks have large-enough mempools,
            // but that unit tests have ones that are not too expensive to allocate
            let pool_size = match cfg!(debug_assertions) {
                true => 0x800,
                false => 0x8020,
            };

            MemoryPool::new(
                format!("{uid_str:?}"),
                pool_size,
                0,
                0,
                RTE_MBUF_DEFAULT_BUF_SIZE as u16,
                lcore::SOCKET_ID_ANY,
            )
            .expect("fail to initialize mbuf pool")
        };

        let mbufs = bufs
            .iter()
            .map(|buf| {
                let mut mbuf = unsafe { memory_pool.alloc() }.unwrap();

                mbuf.extend_from_slice(buf.as_ref());
                mbuf
            })
            .collect();

        GeneratedMbufs { mbufs, _mbuf_pool: memory_pool }
    }
}
