use nonmax::NonMaxU32;

/// NUMA socket identifier
///
/// Using [`NonMaxU32`] since in DPDK the max value (actually -1) represents ANY socket id but in Rust we prefer [`None`] instead.
///
/// See also: <https://doc.dpdk.org/api-21.08/rte__memory_8h.html#a0307f4470d3f391102b0f489fc7d91b5>
#[derive(Debug, PartialEq, Eq)]
pub struct SocketId(NonMaxU32);

impl SocketId {
    #[inline]
    pub fn new(id: u32) -> Option<Self> {
        NonMaxU32::new(id).map(SocketId)
    }

    #[inline]
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}
