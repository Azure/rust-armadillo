mod xstats;

use std::{
    ffi::CStr,
    iter::from_fn,
    mem::{self, MaybeUninit},
    ptr, slice,
};

use arrayvec::ArrayVec;
use mac_addr::MacAddr;
use rte_error::{Error, ReturnValue as _};

pub use self::xstats::XStatsDefs;
use crate::{mbuf::MBuf, memory::SocketId, mempool::MemoryPool, Result};

pub const MAX_QUEUE: u16 = u16::MAX;

pub type DeviceInfo = ffi::rte_eth_dev_info;
pub type DeviceStats = ffi::rte_eth_stats;
pub type EthRxMode = ffi::rte_eth_rxmode;
pub type EthTxMode = ffi::rte_eth_txmode;
pub type FdirConf = ffi::rte_fdir_conf;
pub type IntrConf = ffi::rte_intr_conf;
pub type Conf = ffi::rte_eth_conf;

/// An ethernet device (port) and associated functionality from [here](https://doc.dpdk.org/api-21.08/rte__ethdev_8h.html)
#[derive(PartialEq, Eq, Clone)]
pub struct EthDev {
    port_id: u16,
}

impl EthDev {
    #[inline]
    pub fn new(port_id: u16) -> Self {
        EthDev { port_id }
    }

    #[inline]
    pub fn port_id(&self) -> u16 {
        self.port_id
    }

    /// Configure an Ethernet device.
    /// This function must be invoked first before any other function in the Ethernet API. This function can also be re-invoked when a device is in the stopped state.
    #[inline]
    pub fn configure(&self, nb_rx_queue: u16, nb_tx_queue: u16, conf: &Conf) -> Result<()> {
        unsafe { ffi::rte_eth_dev_configure(self.port_id, nb_rx_queue, nb_tx_queue, conf) }.rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn adjust_nb_rx_tx_desc(&self, nb_rx_desc: &mut u16, nb_tx_desc: &mut u16) -> Result<()> {
        unsafe { ffi::rte_eth_dev_adjust_nb_rx_tx_desc(self.port_id, nb_rx_desc, nb_tx_desc) }.rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn mac_addr(&self) -> Result<MacAddr> {
        let mut addr: ffi::rte_ether_addr = Default::default();
        unsafe { ffi::rte_eth_macaddr_get(self.port_id, &mut addr) }.rte_ok()?;
        Ok(MacAddr::from(addr.addr_bytes))
    }

    #[inline]
    pub fn info(&self) -> Result<DeviceInfo> {
        let mut info: DeviceInfo = Default::default();
        unsafe { ffi::rte_eth_dev_info_get(self.port_id, &mut info) }.rte_ok()?;
        Ok(info)
    }

    #[inline]
    pub fn stats(&self) -> Result<DeviceStats> {
        let mut stats: DeviceStats = Default::default();
        unsafe { ffi::rte_eth_stats_get(self.port_id, &mut stats) }.rte_ok()?;
        Ok(stats)
    }

    #[inline]
    fn socket_id(&self) -> Result<SocketId> {
        // -1 is returned if the port_id (self) is out of range
        let ret = unsafe { ffi::rte_eth_dev_socket_id(self.port_id) };
        // cast from i32 to u32 (e.g., -1 == u32::MAX)
        let id = unsafe { *(&ret as *const _ as *const u32) };
        SocketId::new(id).ok_or(Error(ret))
    }

    #[inline]
    pub fn start(&self) -> Result<()> {
        unsafe { ffi::rte_eth_dev_start(self.port_id) }.rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn stop(&self) -> Result<()> {
        unsafe { ffi::rte_eth_dev_stop(self.port_id) }.rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn close(&self) -> Result<()> {
        unsafe { ffi::rte_eth_dev_close(self.port_id) }.rte_ok()?;
        Ok(())
    }

    /// Retrieve a burst of input packets from a receive queue of an Ethernet device.
    ///
    /// The received packets will be appended to `rx_pkts`. This method uses the array's current capacity
    /// (i.e. `CAP - rx_pkts.len()`) as a buffer for the DPDK library to write the received packets into,
    /// so in order to utilize the array's entire capacity, it should be empty when calling this function.
    ///
    /// # Safety
    /// It is up to the caller to guarantee that `mempool` matches the memory pool
    /// used in the call to [`Self::rx_queue_setup`] for this queue.
    #[inline]
    pub unsafe fn rx_burst<'mempool, const CAP: usize>(
        &self,
        queue_id: u16,
        _mempool: &'mempool MemoryPool,
        rx_pkts: &mut ArrayVec<MBuf<&'mempool MemoryPool>, CAP>,
    ) {
        let old_len = rx_pkts.len();

        // this code was adapted from the Vec::spare_capacity_mut method, which ArrayVec unfortunately does not have
        let spare_cap = slice::from_raw_parts_mut(
            rx_pkts.as_mut_ptr().add(old_len) as *mut MaybeUninit<MBuf<&'mempool MemoryPool>>,
            rx_pkts.remaining_capacity(),
        );

        let received =
            ffi::_rte_eth_rx_burst(self.port_id, queue_id, spare_cap.as_mut_ptr() as _, spare_cap.len() as u16)
                as usize;
        rx_pkts.set_len(old_len + received);
    }

    /// Send a burst of output packets on a transmit queue of an Ethernet device.
    ///
    /// Packets that have been successfully sent will be removed from `tx_pkts`, any `MBufs` remaining in the array
    /// after this method has completed are packets that were NOT sent.
    ///
    /// # Safety
    /// It is up to the caller to guarantee that `mempool` matches the memory pool
    /// used in the call to [`Self::tx_queue_setup`] for this queue.
    #[inline]
    pub unsafe fn tx_burst<'mempool, const CAP: usize>(
        &self,
        queue_id: u16,
        _mempool: &'mempool MemoryPool,
        tx_pkts: &mut ArrayVec<MBuf<&'mempool MemoryPool>, CAP>,
    ) {
        let transmitted =
            ffi::_rte_eth_tx_burst(self.port_id, queue_id, tx_pkts.as_mut_ptr() as _, tx_pkts.len() as u16) as usize;

        // rte_eth_tx_burst assumes ownership of the mbufs that were successfully transmitted,
        // so we remove them from tx_pkts and use mem::forget to prevent dropping (and freeing) them ourselves
        tx_pkts.drain(..transmitted).for_each(mem::forget);
    }

    #[inline]
    pub fn rx_queue_setup(
        &self,
        rx_queue_id: u16,
        nb_rx_desc: u16,
        rx_conf: Option<ffi::rte_eth_rxconf>,
        mempool: &mut MemoryPool,
    ) -> Result<()> {
        unsafe {
            ffi::rte_eth_rx_queue_setup(
                self.port_id,
                rx_queue_id,
                nb_rx_desc,
                self.socket_id()?.get(),
                rx_conf.as_ref().map(|conf| conf as *const _).unwrap_or(ptr::null()),
                mempool.0.as_ptr(),
            )
        }
        .rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn tx_queue_setup(
        &self,
        tx_queue_id: u16,
        nb_tx_desc: u16,
        tx_conf: Option<ffi::rte_eth_txconf>,
    ) -> Result<()> {
        unsafe {
            ffi::rte_eth_tx_queue_setup(
                self.port_id,
                tx_queue_id,
                nb_tx_desc,
                self.socket_id()?.get(),
                tx_conf.as_ref().map(|conf| conf as *const _).unwrap_or(ptr::null()),
            )
        }
        .rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn promiscuous_enable(&self) -> Result<()> {
        unsafe { ffi::rte_eth_promiscuous_enable(self.port_id) }.rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn promiscuous_disable(&self) -> Result<()> {
        unsafe { ffi::rte_eth_promiscuous_disable(self.port_id) }.rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn promiscuous_get(&self) -> Result<bool> {
        let ret = unsafe { ffi::rte_eth_promiscuous_get(self.port_id) }.rte_ok()?;
        Ok(ret.is_positive())
    }

    /// Based on [RTE_ETH_FOREACH_DEV](https://doc.dpdk.org/api-21.08/rte__ethdev_8h.html#ad7b46c67203d37fe3a34f11076d970d6)
    #[inline]
    pub fn for_each() -> impl Iterator<Item = EthDev> {
        let mut next_port_id: u16 = 0;

        from_fn(move || {
            next_port_id =
                unsafe { ffi::rte_eth_find_next_owned_by(next_port_id, ffi::RTE_ETH_DEV_NO_OWNER as u64) } as u16;
            let cur_port_id = if next_port_id < ffi::RTE_MAX_ETHPORTS as u16 { Some(next_port_id) } else { None };
            next_port_id += 1;
            cur_port_id
        })
        .map(EthDev::new)
    }
}

pub trait DeviceInfoWrapper {
    fn get_device_name(&self) -> String;
    fn get_driver_name(&self) -> String;
}

impl DeviceInfoWrapper for DeviceInfo {
    fn get_device_name(&self) -> String {
        unsafe { CStr::from_ptr((*self.device).name).to_str().unwrap().to_string() }
    }
    fn get_driver_name(&self) -> String {
        unsafe { CStr::from_ptr(self.driver_name).to_str().unwrap().to_string() }
    }
}
