mod allocator;

use std::{
    fmt,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

use ffi::_bindgen_ty_14::{RTE_MBUF_L2_LEN_BITS, RTE_MBUF_L3_LEN_BITS};

pub use self::allocator::Allocator;
#[cfg(any(test, feature = "test-utils"))]
pub use self::allocator::GlobalAllocator;
use crate::{flags::PktTxOffload, mempool::MemoryPool};

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
/// # Allocators
/// `MBuf` is generic over a type implementing the [`Allocator`] trait.
///
/// The default allocator is [`MemoryPool`], which uses RTE's memory pool to allocate and manage mbufs.
#[cfg_attr(
    any(test, feature = "test-utils"),
    doc = "For testing, the [`GlobalAllocator`] can be used creating mbufs without relying on the EAL (see also: [`alloc_mbufs`])."
)]
///
/// # See also
/// - The DPDK documentation on the [Mbuf Library](https://doc.dpdk.org/guides-21.08/prog_guide/mbuf_lib.html).
///
/// # Implementation notes
/// - This wrapper completely ignores all but the first segment of an mbuf.
#[repr(transparent)]
pub struct MBuf<A = MemoryPool>
where
    A: Allocator,
{
    ptr: NonNull<ffi::rte_mbuf>,
    _marker: PhantomData<A>,
}

impl<A> MBuf<A>
where
    A: Allocator + Default,
{
    /// Allocate an empty mbuf with a default [allocator](Allocator).
    #[inline]
    pub fn new() -> Self {
        Self::new_with_provider(&mut A::default())
    }

    /// Allocate an mbuf with a default [allocator](Allocator).
    #[inline]
    pub fn new_with_data<T: AsRef<[u8]>>(data: T) -> Self {
        Self::new_with_provider_and_data(&mut A::default(), data)
    }
}

impl<A> MBuf<A>
where
    A: Allocator,
{
    /// Allocate an empty mbuf with the given [allocator](Allocator).
    #[track_caller]
    #[inline]
    pub fn new_with_provider(provider: &mut A) -> Self {
        let ptr = provider.alloc().expect("Could not allocate mbuf");
        Self { ptr, _marker: Default::default() }
    }

    /// Allocate an mbuf with the given [allocator](Allocator).
    #[inline]
    pub fn new_with_provider_and_data<T: AsRef<[u8]>>(provider: &mut A, data: T) -> Self {
        let mut mbuf = Self::new_with_provider(provider);
        mbuf.extend_from_slice(data.as_ref());
        mbuf
    }
}

impl<A> MBuf<A>
where
    A: Allocator,
{
    /// Returns the raw pointer to the `rte_mbuf` struct, pointed to by this `MBuf`.
    ///
    /// # Safety
    /// It is up to the caller to make sure to not use the raw pointer in a way that breaks the safety invariants the `MBuf` struct relies on.
    #[inline]
    pub unsafe fn as_raw(&self) -> *mut ffi::rte_mbuf {
        self.ptr.as_ptr()
    }

    /* Helper functions for Deref and DerefMut impls. */

    fn data_ptr(&self) -> *mut u8 {
        unsafe {
            let ffi::rte_mbuf { buf_addr, data_off, .. } = *self.ptr.as_ref();
            buf_addr.add(data_off.into()) as *mut _
        }
    }

    fn data_len(&self) -> usize {
        unsafe { self.ptr.as_ref() }.data_len.into()
    }
}

/// These method are equivilent to their [`Vec`] counterparts.
impl<A> MBuf<A>
where
    A: Allocator,
{
    fn capacity(&self) -> usize {
        unsafe {
            let ffi::rte_mbuf { data_off, buf_len, .. } = *self.ptr.as_ref();
            buf_len.checked_sub(data_off).unwrap_unchecked().into()
        }
    }

    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            let ffi::rte_mbuf { buf_addr, data_off, data_len, buf_len, .. } = *self.ptr.as_ref();
            let spare_cap = buf_addr.add(data_off.into()).add(data_len.into());
            let buf_end = spare_cap.add(buf_len.into());
            slice::from_raw_parts_mut(spare_cap as _, spare_cap.offset_from(buf_end) as usize)
        }
    }

    unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity());
        let mbuf = self.ptr.as_ptr();
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

impl<A> MBuf<A>
where
    A: Allocator,
{
    /// Sets the [`l2_len`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#aa25a7c259438b9eba28bcedc33846620) field.
    #[inline]
    pub fn set_l2_len(&mut self, len: u64) {
        assert!(len < 1 << RTE_MBUF_L2_LEN_BITS);
        unsafe {
            let mbuf = self.ptr.as_mut();
            mbuf.__bindgen_anon_3.__bindgen_anon_1.set_l2_len(len);
        }
    }

    /// Sets the [`l3_len`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#a82a34cb6d5935a8c0f043f2783d6b42d) field.
    #[inline]
    pub fn set_l3_len(&mut self, len: u64) {
        assert!(len < 1 << RTE_MBUF_L3_LEN_BITS);
        unsafe {
            let mbuf = self.ptr.as_mut();
            mbuf.__bindgen_anon_3.__bindgen_anon_1.set_l3_len(len);
        }
    }

    /// Enables (bitwise-or) the given flags on the [`ol_flags`](https://doc.dpdk.org/api-2.2/structrte__mbuf.html#a319d580a6e1ef13692631d7b0d6d5c98) field.
    ///
    /// See also: [`PktTxOffload`].
    #[inline]
    pub fn enable_ol_flags(&mut self, flags: PktTxOffload) {
        unsafe {
            let mbuf = self.ptr.as_mut();
            mbuf.ol_flags |= flags.bits();
        }
    }
}

impl<A> Drop for MBuf<A>
where
    A: Allocator,
{
    #[inline]
    fn drop(&mut self) {
        unsafe {
            A::free(self.ptr);
        }
    }
}

impl<A> Deref for MBuf<A>
where
    A: Allocator,
{
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.data_len()) }
    }
}

impl<A> DerefMut for MBuf<A>
where
    A: Allocator,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.data_ptr(), self.data_len()) }
    }
}

impl<A> fmt::Debug for MBuf<A>
where
    A: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <[u8] as fmt::Debug>::fmt(self, f)
    }
}

impl<A> Default for MBuf<A>
where
    A: Allocator + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Clone for MBuf<A>
where
    A: Allocator,
{
    #[track_caller]
    #[inline]
    fn clone(&self) -> Self {
        let ptr = unsafe { A::clone(self.ptr) }.expect("Failed to allocate mbuf clone");
        Self { ptr, _marker: Default::default() }
    }
}

#[cfg(any(test, feature = "test-utils"))]
/// Small helper for allocating and collecting an [`ArrayVec<MBuf>`](arrayvec::ArrayVec) from an iterator over byte slices,
/// using a [`GlobalAllocator`] as the mbuf allocator.
///
/// # Example
/// ```rust
/// # use arrayvec::ArrayVec;
/// # use rte::mbuf::{alloc_mbufs, MBuf};
/// #
/// let packets = [b"\x00\x01", b"\x02\x03"];
/// let mbufs: ArrayVec<MBuf<_>, 2> = alloc_mbufs(packets);
/// assert_eq!(&mbufs[0][..], b"\x00\x01");
/// assert_eq!(&mbufs[1][..], b"\x02\x03");
/// ```
pub fn alloc_mbufs<const CAP: usize, B: AsRef<[u8]>, I: IntoIterator<Item = B>>(
    iter: I,
) -> arrayvec::ArrayVec<MBuf<GlobalAllocator>, CAP> {
    iter.into_iter().map(MBuf::<GlobalAllocator>::new_with_data).collect()
}
