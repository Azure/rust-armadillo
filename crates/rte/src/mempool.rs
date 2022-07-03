use std::{
    ffi::CString,
    fmt,
    mem::size_of_val,
    ptr::{addr_of, NonNull},
    slice,
};

use rte_error::ReturnValue as _;

use crate::{mbuf::MBuf, Result};

#[repr(transparent)]
pub struct MemoryPool(pub(crate) NonNull<ffi::rte_mempool>);

// # Safety
// All operations that can be performed on a memory pool are implemented in a thread-safe manner by DPDK, as mentioned (briefly) here:
// [Thread Safety of DPDK Functions](https://doc.dpdk.org/guides-20.08/prog_guide/thread_safety_dpdk_functions.html#fast-path-apis)
unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}

impl MemoryPool {
    /// Creates a new memory pool.
    ///
    /// Uses the [`ffi::rte_pktmbuf_pool_create_by_ops`] function under the hood.
    ///
    /// See also: <https://doc.dpdk.org/api-21.08/rte__mbuf_8h.html#a9e4bd0ae9e01d0f4dfe7d27cfb0d9a7f>
    #[inline]
    pub fn new<S: Into<Vec<u8>>>(
        name: S,
        size: u32,
        cache_size: u32,
        private_size: u16,
        data_room_size: u16,
        socket_id: i32,
    ) -> Result<Self> {
        let name = CString::new(name).unwrap();
        let ops = ffi::RTE_MBUF_DEFAULT_MEMPOOL_OPS;

        unsafe {
            ffi::rte_pktmbuf_pool_create_by_ops(
                name.as_ptr(),
                size,
                cache_size,
                private_size,
                data_room_size,
                socket_id,
                ops.as_ptr() as _,
            )
        }
        .rte_ok()
        .map(Self)
    }

    /// Allocates a new [`MBuf`] from this memory pool.
    ///
    /// # Safety
    /// The allocated `MBuf` is not tied this `MemoryPool`'s lifetime,
    /// so it is up to the caller to ensure it is not used after the memory pool has been freed (i.e. dropped).
    #[inline]
    pub unsafe fn alloc(&self) -> Result<MBuf> {
        ffi::_rte_pktmbuf_alloc(self.0.as_ptr()).rte_ok().map(MBuf)
    }

    #[inline]
    pub fn name(&self) -> &[u8] {
        let name = unsafe {
            let name = addr_of!((*self.0.as_ptr()).name);
            slice::from_raw_parts::<'_, u8>(name as _, size_of_val(&*name))
        };

        // trim slice after the first nul character found
        if let Some((null_pos, _)) = name.iter().enumerate().find(|(_, c)| **c == 0) {
            &name[..null_pos]
        } else {
            name
        }
    }

    /// Returns the size of this memory pool, i.e. the number of mbufs it has capacity for.
    ///
    /// See also: <https://doc.dpdk.org/api-21.08/structrte__mempool.html#ab2c6b258f02add8fdf4cfc7c371dd772>
    #[inline]
    pub fn size(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).size }
    }

    /// See also: <https://doc.dpdk.org/api-21.08/structrte__mempool.html#ac0fc8e6a5ca95e81e5d94522c86cfc9c>
    #[inline]
    pub fn cache_size(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).cache_size }
    }

    /// See also: <https://doc.dpdk.org/api-21.08/rte__mbuf_8h.html#afc63705bb85669e2a1ea17e3279d59ce>
    #[inline]
    pub fn private_data_size(&self) -> u16 {
        unsafe { ffi::_rte_pktmbuf_priv_size(self.0.as_ptr()) }
    }

    /// See also: <https://doc.dpdk.org/api-21.08/rte__mbuf_8h.html#ac8fe14dae4b72eeecadcb684af5a9703>
    #[inline]
    pub fn data_room_size(&self) -> u16 {
        unsafe { ffi::_rte_pktmbuf_data_room_size(self.0.as_ptr()) }
    }

    /// Returns the number of free mbufs in this memory pool's capacity.
    ///
    /// Equivalent to `mempool.size() - mempool.get_in_use_count()`.
    ///
    /// See also: <https://doc.dpdk.org/api-21.08/rte__mempool_8h.html#a505a815fc46e027a0a2054df124bc514>
    #[inline]
    pub fn get_available_count(&self) -> u32 {
        unsafe { ffi::rte_mempool_avail_count(self.0.as_ptr()) }
    }

    /// Returns the number of used mbufs in this memory pool's capacity.
    ///
    /// Equivalent to `mempool.size() - mempool.get_available_count()`.
    ///
    /// See also: <https://doc.dpdk.org/api-21.08/rte__mempool_8h.html#abce09dff484b6726ced4da3bbe3b2e55>
    #[inline]
    pub fn get_in_use_count(&self) -> u32 {
        unsafe { ffi::rte_mempool_in_use_count(self.0.as_ptr()) }
    }
}

impl fmt::Debug for MemoryPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MemoryPool")
            .field("name", &String::from_utf8_lossy(self.name()))
            .field("size", &self.size())
            .field("cache_size", &self.cache_size())
            .field("in_use", &self.get_in_use_count())
            .field("private_data_size", &self.private_data_size())
            .field("data_room_size", &self.data_room_size())
            .finish()
    }
}

impl Drop for MemoryPool {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::rte_mempool_free(self.0.as_ptr()) }
    }
}
