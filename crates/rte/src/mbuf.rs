use std::{ptr::NonNull, slice};

use rte_error::ReturnValue as _;

use crate::{
    mempool,
    utils::{self, AsCString, AsRaw},
    Result,
};

pub type RawMBuf = ffi::rte_mbuf;
pub type RawMBufPtr = *mut ffi::rte_mbuf;

raw!(pub MBuf(RawMBuf));

impl MBuf {
    /// Get a pointer which points to an offset into the data in the mbuf.
    #[inline]
    pub fn mtod_offset<T>(&self, off: usize) -> NonNull<T> {
        NonNull::new(self.as_mut_ptr_offset(off)).unwrap().cast()
    }

    /// Get a pointer which points to the start of the data in the mbuf.
    #[inline]
    pub fn mtod<T>(&self) -> NonNull<T> {
        self.mtod_offset(0)
    }

    /// Prepend len bytes to an mbuf data area.
    pub fn prepend(&mut self, len: usize) -> Result<NonNull<u8>> {
        let ptr = unsafe { ffi::_rte_pktmbuf_prepend(self.as_raw(), len as u16) }.rte_ok()?;
        Ok(ptr.cast())
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.mtod().as_ptr(), self.data_len as usize) }
    }

    #[inline]
    // TODO: temporary hack, fix mbuf lifetimes https://msazure.visualstudio.com/One/_workitems/edit/10357334
    pub fn as_mut_slice(&mut self) -> &'static mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.mtod().as_ptr(), self.data_len as usize) }
    }

    #[inline]
    pub fn as_mut_ptr_offset(&self, off: usize) -> *mut u8 {
        unsafe { (self.buf_addr as *mut u8).add(self.data_off as usize + off) }
    }

    /// Free a packet mbuf back into its original mempool.
    ///
    /// Free an mbuf, and all its segments in case of chained buffers.
    /// Each segment is added back into its original mempool.
    pub fn free(&mut self) {
        #[cfg(not(feature = "benchmark"))]
        unsafe {
            ffi::_rte_pktmbuf_free(self.as_raw())
        }
    }
}

pub trait MBufPool {
    /// Allocate a new mbuf from a mempool.
    fn alloc(&mut self) -> Result<MBuf>;
}

impl MBufPool for mempool::MemoryPool {
    fn alloc(&mut self) -> Result<MBuf> {
        let ptr = unsafe { ffi::_rte_pktmbuf_alloc(self.as_raw()) }.rte_ok()?;
        Ok(MBuf(ptr))
    }
}

pub fn pool_create<S: AsRef<str>>(
    name: S,
    n: u32,
    cache_size: u32,
    priv_size: u16,
    data_room_size: u16,
    socket_id: i32,
) -> Result<mempool::MemoryPool> {
    let name = name.as_c_str();
    let ops = "ring_mp_mc".as_c_str();

    let ptr = unsafe {
        ffi::rte_pktmbuf_pool_create_by_ops(
            name.as_ptr(),
            n,
            cache_size,
            priv_size,
            data_room_size,
            socket_id,
            ops.as_ptr(),
        )
    }
    .rte_ok()?;
    Ok(mempool::MemoryPool::from(ptr.as_ptr()))
}
