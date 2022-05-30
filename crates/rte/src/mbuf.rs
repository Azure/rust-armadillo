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
    let ops = ffi::RTE_MBUF_DEFAULT_MEMPOOL_OPS;

    let ptr = unsafe {
        ffi::rte_pktmbuf_pool_create_by_ops(
            name.as_ptr(),
            n,
            cache_size,
            priv_size,
            data_room_size,
            socket_id,
            ops as *const u8 as *const i8,
        )
    }
    .rte_ok()?;
    Ok(mempool::MemoryPool::from(ptr.as_ptr()))
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    use std::{ops::Deref, ptr};

    use ffi::RTE_MBUF_DEFAULT_BUF_SIZE;
    use uuid::Uuid;

    use crate::{
        lcore,
        mbuf::{self, MBufPool as _},
        mempool,
        utils::AsRaw as _,
    };

    pub struct GeneratedMbufs {
        mbufs: Vec<mbuf::RawMBufPtr>,
        _mbuf_pool: mempool::MemoryPool,
    }

    impl Deref for GeneratedMbufs {
        type Target = [mbuf::RawMBufPtr];

        fn deref(&self) -> &Self::Target {
            self.mbufs.deref()
        }
    }

    pub fn pool_create_from_bufs<B: AsRef<[u8]>>(bufs: &[B]) -> GeneratedMbufs {
        let mut memory_pool = {
            let mut uid_str = Uuid::new_v4().to_string();
            let id_len = uid_str.chars().count() - 10; // leave space for DPDK prefix
            uid_str.drain(0..id_len);

            mbuf::pool_create(
                format!("{uid_str:?}"),
                bufs.len().try_into().unwrap(),
                0,
                0,
                RTE_MBUF_DEFAULT_BUF_SIZE as u16,
                lcore::SOCKET_ID_ANY,
            )
            .expect("fail to initialize mbuf pool")
        };

        let mbufs = bufs
            .iter()
            .map(AsRef::<[u8]>::as_ref)
            .map(|buf| {
                let mut mbuf = memory_pool.alloc().unwrap();
                let pkt_len: u16 = buf.len().try_into().unwrap();

                let buf_ptr = mbuf.as_mut_ptr_offset(0);
                assert!(pkt_len <= mbuf.buf_len - mbuf.data_off);
                unsafe {
                    ptr::copy(buf.as_ptr(), buf_ptr, buf.len());
                }

                mbuf.data_len = pkt_len;
                mbuf.as_raw()
            })
            .collect::<Vec<_>>();

        GeneratedMbufs { mbufs, _mbuf_pool: memory_pool }
    }
}
