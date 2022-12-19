mod allocator;
mod metadata;
mod ptr;

use std::{
    fmt,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
};

#[cfg(any(test, feature = "test-utils"))]
pub use self::allocator::GlobalAllocator;
pub use self::{
    allocator::Allocator,
    metadata::{MetadataExt, MetadataPart},
};

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
/// # use rte::{mempool::MemoryPool, mbuf::MBuf};
/// assert_not_impl_any!(MBuf<&MemoryPool>: Send, Sync);
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
pub struct MBuf<A>
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
        Self::new_with_provider(&A::default())
    }

    /// Allocate an mbuf with a default [allocator](Allocator).
    #[inline]
    pub fn new_with_data<T: AsRef<[u8]>>(data: T) -> Self {
        Self::new_with_provider_and_data(&A::default(), data)
    }
}

impl<A> MBuf<A>
where
    A: Allocator,
{
    /// Allocate an empty mbuf with the given [allocator](Allocator).
    #[track_caller]
    #[inline]
    pub fn new_with_provider(provider: &A) -> Self {
        let ptr = provider.alloc().expect("Could not allocate mbuf");
        Self { ptr, _marker: Default::default() }
    }

    /// Allocate an mbuf with the given [allocator](Allocator).
    #[inline]
    pub fn new_with_provider_and_data<T: AsRef<[u8]>>(provider: &A, data: T) -> Self {
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

    /// "Splits" a mutable reference to `self` into two disjoint mutable references, one to the underlying buffer,
    /// and one to a [`MetadataPart`], which (only) allows modifying metadata related to the `MBuf` pointed to by `self`.
    #[inline]
    pub fn split_metadata_mut<'a>(&'a mut self) -> (&'a mut [u8], MetadataPart<'a>) {
        let this = self.ptr;

        let data = self.deref_mut();

        // Sanity check:
        // make sure data buffer memory is completely nonoverlapping with mbuf struct,
        // i.e.: ends before beginning of mbuf struct, or starts after it.
        // this is the most important prerequisite for ensuring `MetadataPart<'a>` can
        // coexist with `&'a mut [u8]`, without violating stacked borrows
        debug_assert!(unsafe {
            let this_ptr_range = this.as_ptr() as *const u8..this.as_ptr().add(1) as *const u8;
            let data_ptr_range = data.as_ptr_range();

            data_ptr_range.end <= this_ptr_range.start || data_ptr_range.start >= this_ptr_range.end
        });

        let metadata = MetadataPart::<'a> { ptr: this, _marker: PhantomData };

        (data, metadata)
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

    /// Extracts a mutable slice of the entire underlying buffer.
    ///
    /// Equivalent to `&mut mbuf[..]`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self
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
