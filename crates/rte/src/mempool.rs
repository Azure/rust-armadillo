use crate::utils::{self, AsRaw};

pub type RawMemoryPool = ffi::rte_mempool;
pub type RawMemoryPoolPtr = *mut ffi::rte_mempool;

raw! {
    /// The RTE mempool structure.
    pub MemoryPool(RawMemoryPool)
}

impl MemoryPool {
    fn free(&mut self) {
        unsafe { ffi::rte_mempool_free(self.as_raw()) }
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        self.free()
    }
}
