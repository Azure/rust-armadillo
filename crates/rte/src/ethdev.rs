use std::{
    mem::{self, MaybeUninit},
    ops::Range,
    ptr, slice,
};

use arrayvec::ArrayVec;
use ffi::RTE_MAX_ETHPORTS;
use mac_addr::MacAddr;
use rte_error::{Error, ReturnValue as _};

use crate::{
    flags::{EthLinkSpeed, EthRss},
    mbuf::MBuf,
    memory::SocketId,
    mempool, Result,
};

pub type PortId = u16;
pub type QueueId = u16;

pub const NO_QUEUE: QueueId = u16::MAX;

/// A structure used to retrieve link-level information of an Ethernet port.
pub struct EthLink {
    pub speed: u32,
    pub duplex: bool,
    pub autoneg: bool,
    pub up: bool,
}

pub trait EthDevice {
    fn portid(&self) -> PortId;

    /// Configure an Ethernet device.
    ///
    /// This function must be invoked first before any other function in the Ethernet API.
    /// This function can also be re-invoked when a device is in the stopped state.
    ///
    fn configure(&self, nb_rx_queue: QueueId, nb_tx_queue: QueueId, conf: &EthConf) -> Result<&Self>;

    /// Retrieve the contextual information of an Ethernet device.
    fn info(&self) -> Result<RawEthDeviceInfo>;

    /// Retrieve the general I/O statistics of an Ethernet device.
    fn stats(&self) -> Result<RawEthDeviceStats>;

    /// Reset the general I/O statistics of an Ethernet device.
    fn reset_stats(&self) -> &Self;

    /// Retrieve the Ethernet address of an Ethernet device.
    fn mac_addr(&self) -> Result<MacAddr>;

    /// Set the default MAC address.
    fn set_mac_addr(&self, addr: MacAddr) -> Result<&Self>;

    /// Return the NUMA socket to which an Ethernet device is connected
    fn socket_id(&self) -> Result<SocketId>;

    /// Check if port_id of device is attached
    fn is_valid(&self) -> bool;

    /// Allocate and set up a receive queue for an Ethernet device.
    ///
    /// The function allocates a contiguous block of memory for *nb_rx_desc*
    /// receive descriptors from a memory zone associated with *socket_id*
    /// and initializes each receive descriptor with a network buffer allocated
    /// from the memory pool *mb_pool*.
    fn rx_queue_setup(
        &self,
        rx_queue_id: QueueId,
        nb_rx_desc: u16,
        rx_conf: Option<ffi::rte_eth_rxconf>,
        mb_pool: &mut mempool::MemoryPool,
    ) -> Result<&Self>;

    /// Allocate and set up a transmit queue for an Ethernet device.
    fn tx_queue_setup(
        &self,
        tx_queue_id: QueueId,
        nb_tx_desc: u16,
        tx_conf: Option<ffi::rte_eth_txconf>,
    ) -> Result<&Self>;

    /// Enable receipt in promiscuous mode for an Ethernet device.
    fn promiscuous_enable(&self) -> Result<&Self>;

    /// Disable receipt in promiscuous mode for an Ethernet device.
    fn promiscuous_disable(&self) -> Result<&Self>;

    /// Return the value of promiscuous mode for an Ethernet device.
    fn is_promiscuous_enabled(&self) -> Result<bool>;

    /// Retrieve the MTU of an Ethernet device.
    fn mtu(&self) -> Result<u16>;

    /// Change the MTU of an Ethernet device.
    fn set_mtu(&self, mtu: u16) -> Result<&Self>;

    /// Retrieve the Ethernet device link status
    #[inline]
    fn is_up(&self) -> bool {
        self.link().up
    }

    /// Retrieve the status (ON/OFF), the speed (in Mbps) and
    /// the mode (HALF-DUPLEX or FULL-DUPLEX) of the physical link of an Ethernet device.
    ///
    /// It might need to wait up to 9 seconds in it.
    ///
    fn link(&self) -> EthLink;

    /// Retrieve the status (ON/OFF), the speed (in Mbps) and
    /// the mode (HALF-DUPLEX or FULL-DUPLEX) of the physical link of an Ethernet device.
    ///
    /// It is a no-wait version of rte_eth_link_get().
    ///
    fn link_nowait(&self) -> EthLink;

    /// Link up an Ethernet device.
    fn set_link_up(&self) -> Result<&Self>;

    /// Link down an Ethernet device.
    fn set_link_down(&self) -> Result<&Self>;

    ///Check that numbers of Rx and Tx descriptors satisfy descriptors limits from the ethernet device information,
    /// otherwise adjust them to boundaries.
    fn adjust_nb_rx_tx_desc(&self, nb_rx_desc: &mut u16, nb_tx_desc: &mut u16) -> Result<&Self>;

    /// Allocate mbuf from mempool, setup the DMA physical address
    /// and then start RX for specified queue of a port. It is used
    /// when rx_deferred_start flag of the specified queue is true.
    fn rx_queue_start(&self, rx_queue_id: QueueId) -> Result<&Self>;

    /// Stop specified RX queue of a port
    fn rx_queue_stop(&self, rx_queue_id: QueueId) -> Result<&Self>;

    /// Start TX for specified queue of a port.
    /// It is used when tx_deferred_start flag of the specified queue is true.
    fn tx_queue_start(&self, tx_queue_id: QueueId) -> Result<&Self>;

    /// Stop specified TX queue of a port
    fn tx_queue_stop(&self, tx_queue_id: QueueId) -> Result<&Self>;

    /// Start an Ethernet device.
    fn start(&self) -> Result<&Self>;

    /// Stop an Ethernet device.
    fn stop(&self) -> &Self;

    /// Close a stopped Ethernet device. The device cannot be restarted!
    fn close(&self) -> &Self;

    /// Retrieve a burst of input packets from a receive queue of an Ethernet device.
    ///
    /// The received packets will be appended to `rx_pkts`. This method uses the array's current capacity
    /// (i.e. `CAP - rx_pkts.len()`) as a buffer for the DPDK library to write the received packets into,
    /// so in order to utilize the array's entire capacity, it should be empty when calling this function.
    fn rx_burst<const CAP: usize>(&self, queue_id: QueueId, rx_pkts: &mut ArrayVec<MBuf, CAP>);

    /// Send a burst of output packets on a transmit queue of an Ethernet device.
    ///
    /// Packets that have been successfully sent will be removed from `tx_pkts`, any `MBufs` remaining in the array
    /// after this method has completed are packets that were NOT sent.
    fn tx_burst<const CAP: usize>(&self, queue_id: QueueId, tx_pkts: &mut ArrayVec<MBuf, CAP>);

    fn get_owner_id(&self) -> u64;

    fn find_next(&self) -> PortId;
}

/// Get the total number of Ethernet devices that have been successfully initialized
/// by the matching Ethernet driver during the PCI probing phase.
///
/// All devices whose port identifier is in the range [0, rte::ethdev::count() - 1]
/// can be operated on by network applications immediately after invoking rte_eal_init().
/// If the application unplugs a port using hotplug function,
/// The enabled port numbers may be noncontiguous.
/// In the case, the applications need to manage enabled port by themselves.
pub fn avail_count() -> u16 {
    unsafe { ffi::rte_eth_dev_count_avail() }
}

pub fn avail_devices() -> Range<PortId> {
    0..avail_count()
}

impl EthDevice for PortId {
    fn portid(&self) -> PortId {
        *self
    }

    fn configure(&self, nb_rx_queue: QueueId, nb_tx_queue: QueueId, conf: &EthConf) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_configure(*self, nb_rx_queue, nb_tx_queue, RawEthConf::from(conf).as_raw()) }
            .rte_ok()?;
        Ok(self)
    }

    fn info(&self) -> Result<RawEthDeviceInfo> {
        let mut info: RawEthDeviceInfo = Default::default();

        unsafe { ffi::rte_eth_dev_info_get(*self, &mut info) }.rte_ok()?;
        Ok(info)
    }

    fn stats(&self) -> Result<RawEthDeviceStats> {
        let mut stats: RawEthDeviceStats = Default::default();

        unsafe { ffi::rte_eth_stats_get(*self, &mut stats) }.rte_ok()?;
        Ok(stats)
    }

    fn reset_stats(&self) -> &Self {
        unsafe { ffi::rte_eth_stats_reset(*self) };

        self
    }

    fn mac_addr(&self) -> Result<MacAddr> {
        let mut addr: ffi::rte_ether_addr = Default::default();
        unsafe { ffi::rte_eth_macaddr_get(*self, &mut addr) }.rte_ok()?;

        Ok(MacAddr::from(addr.addr_bytes))
    }

    fn set_mac_addr(&self, addr: MacAddr) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_default_mac_addr_set(*self, addr.as_ptr() as *mut _) }.rte_ok()?;
        Ok(self)
    }

    fn socket_id(&self) -> Result<SocketId> {
        // -1 is returned if the port_id (self) is out of range
        let ret = unsafe { ffi::rte_eth_dev_socket_id(*self) };
        // cast from i32 to u32 (e.g., -1 == u32::MAX)
        let id = unsafe { *(&ret as *const _ as *const u32) };
        SocketId::new(id).ok_or(Error(ret))
    }

    fn is_valid(&self) -> bool {
        unsafe { ffi::rte_eth_dev_is_valid_port(*self) != 0 }
    }

    fn rx_queue_setup(
        &self,
        rx_queue_id: QueueId,
        nb_rx_desc: u16,
        rx_conf: Option<ffi::rte_eth_rxconf>,
        mb_pool: &mut mempool::MemoryPool,
    ) -> Result<&Self> {
        unsafe {
            ffi::rte_eth_rx_queue_setup(
                *self,
                rx_queue_id,
                nb_rx_desc,
                self.socket_id()?.get(),
                rx_conf.as_ref().map(|conf| conf as *const _).unwrap_or(ptr::null()),
                mb_pool.0.as_ptr(),
            )
        }
        .rte_ok()?;
        Ok(self)
    }

    fn tx_queue_setup(
        &self,
        tx_queue_id: QueueId,
        nb_tx_desc: u16,
        tx_conf: Option<ffi::rte_eth_txconf>,
    ) -> Result<&Self> {
        unsafe {
            ffi::rte_eth_tx_queue_setup(
                *self,
                tx_queue_id,
                nb_tx_desc,
                self.socket_id()?.get(),
                tx_conf.as_ref().map(|conf| conf as *const _).unwrap_or(ptr::null()),
            )
        }
        .rte_ok()?;
        Ok(self)
    }

    fn promiscuous_enable(&self) -> Result<&Self> {
        unsafe { ffi::rte_eth_promiscuous_enable(*self) }.rte_ok()?;
        Ok(self)
    }

    fn promiscuous_disable(&self) -> Result<&Self> {
        unsafe { ffi::rte_eth_promiscuous_disable(*self) }.rte_ok()?;
        Ok(self)
    }

    fn is_promiscuous_enabled(&self) -> Result<bool> {
        let ret = unsafe { ffi::rte_eth_promiscuous_get(*self) }.rte_ok()?;
        Ok(ret.is_positive())
    }

    fn mtu(&self) -> Result<u16> {
        let mut mtu: u16 = 0;

        unsafe { ffi::rte_eth_dev_get_mtu(*self, &mut mtu) }.rte_ok()?;
        Ok(mtu)
    }

    fn set_mtu(&self, mtu: u16) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_set_mtu(*self, mtu) }.rte_ok()?;
        Ok(self)
    }

    fn link(&self) -> EthLink {
        let mut link = ffi::rte_eth_link::default();

        unsafe { ffi::rte_eth_link_get(*self, &mut link as *mut _) };

        EthLink {
            speed: link.link_speed,
            duplex: link.link_duplex() != 0,
            autoneg: link.link_autoneg() != 0,
            up: link.link_status() != 0,
        }
    }

    fn link_nowait(&self) -> EthLink {
        let mut link = ffi::rte_eth_link::default();

        unsafe { ffi::rte_eth_link_get_nowait(*self, &mut link as *mut _) };

        EthLink {
            speed: link.link_speed,
            duplex: link.link_duplex() != 0,
            autoneg: link.link_autoneg() != 0,
            up: link.link_status() != 0,
        }
    }

    fn set_link_up(&self) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_set_link_up(*self) }.rte_ok()?;
        Ok(self)
    }

    fn set_link_down(&self) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_set_link_down(*self) }.rte_ok()?;
        Ok(self)
    }

    fn rx_queue_start(&self, rx_queue_id: QueueId) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_rx_queue_start(*self, rx_queue_id) }.rte_ok()?;
        Ok(self)
    }

    fn adjust_nb_rx_tx_desc(&self, nb_rx_desc: &mut u16, nb_tx_desc: &mut u16) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_adjust_nb_rx_tx_desc(*self, nb_rx_desc, nb_tx_desc) }.rte_ok()?;
        Ok(self)
    }

    fn rx_queue_stop(&self, rx_queue_id: QueueId) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_rx_queue_stop(*self, rx_queue_id) }.rte_ok()?;
        Ok(self)
    }

    fn tx_queue_start(&self, tx_queue_id: QueueId) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_tx_queue_start(*self, tx_queue_id) }.rte_ok()?;
        Ok(self)
    }

    fn tx_queue_stop(&self, tx_queue_id: QueueId) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_tx_queue_stop(*self, tx_queue_id) }.rte_ok()?;
        Ok(self)
    }

    fn start(&self) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_start(*self) }.rte_ok()?;
        Ok(self)
    }

    fn stop(&self) -> &Self {
        unsafe { ffi::rte_eth_dev_stop(*self) };

        self
    }

    fn close(&self) -> &Self {
        unsafe { ffi::rte_eth_dev_close(*self) };

        self
    }

    #[inline]
    fn rx_burst<const CAP: usize>(&self, queue_id: QueueId, rx_pkts: &mut ArrayVec<MBuf, CAP>) {
        let old_len = rx_pkts.len();

        // this code was adapted from the Vec::spare_capacity_mut method, which ArrayVec unfortunately does not have
        let spare_cap = unsafe {
            slice::from_raw_parts_mut(
                rx_pkts.as_mut_ptr().add(old_len) as *mut MaybeUninit<MBuf>,
                rx_pkts.remaining_capacity(),
            )
        };

        unsafe {
            let received =
                ffi::_rte_eth_rx_burst(*self, queue_id, spare_cap.as_mut_ptr() as _, spare_cap.len() as u16) as usize;
            rx_pkts.set_len(old_len + received);
        }
    }

    #[inline]
    fn tx_burst<const CAP: usize>(&self, queue_id: QueueId, tx_pkts: &mut ArrayVec<MBuf, CAP>) {
        let transmitted = unsafe {
            ffi::_rte_eth_tx_burst(*self, queue_id, tx_pkts.as_mut_ptr() as _, tx_pkts.len() as u16) as usize
        };

        // rte_eth_tx_burst assumes ownership of the mbufs that were successfuly transmitted,
        // so we remove them from tx_pkts and use mem::forget to prevent dropping (and freeing) them ourselves
        tx_pkts.drain(..transmitted).for_each(mem::forget);
    }

    fn get_owner_id(&self) -> u64 {
        let mut dev_owner = ffi::rte_eth_dev_owner::default();
        unsafe { ffi::rte_eth_dev_owner_get(*self, &mut dev_owner) };
        dev_owner.id
    }

    fn find_next(&self) -> PortId {
        unsafe { ffi::rte_eth_find_next(*self) }
    }
}

pub type RawEthDeviceInfo = ffi::rte_eth_dev_info;

pub type RawEthDeviceStats = ffi::rte_eth_stats;

#[derive(Default, Clone, Copy)]
pub struct EthRssConf {
    pub key: Option<[u8; 40]>,
    pub hash: EthRss,
}

#[derive(Default, Clone, Copy)]
pub struct RxAdvConf {
    /// Port RSS configuration
    pub rss_conf: Option<EthRssConf>,
    pub vmdq_dcb_conf: Option<ffi::rte_eth_vmdq_dcb_conf>,
    pub dcb_rx_conf: Option<ffi::rte_eth_dcb_rx_conf>,
    pub vmdq_rx_conf: Option<ffi::rte_eth_vmdq_rx_conf>,
}

#[derive(Clone, Copy)]
pub enum TxAdvConf {}

impl Default for EthLinkSpeed {
    fn default() -> Self {
        EthLinkSpeed::SPEED_AUTONEG
    }
}

pub type EthRxMode = ffi::rte_eth_rxmode;
pub type EthTxMode = ffi::rte_eth_txmode;

#[derive(Default, Clone, Copy)]
pub struct EthConf {
    /// bitmap of ETH_LINK_SPEED_XXX of speeds to be used.
    ///
    /// ETH_LINK_SPEED_FIXED disables link autonegotiation, and a unique speed shall be set.
    /// Otherwise, the bitmap defines the set of speeds to be advertised.
    /// If the special value ETH_LINK_SPEED_AUTONEG (0) is used,
    /// all speeds supported are advertised.
    pub link_speeds: EthLinkSpeed,
    /// Port RX configuration.
    pub rxmode: Option<EthRxMode>,
    /// Port TX configuration.
    pub txmode: Option<EthTxMode>,
    /// Loopback operation mode.
    ///
    /// By default the value is 0, meaning the loopback mode is disabled.
    /// Read the datasheet of given ethernet controller for details.
    /// The possible values of this field are defined in implementation of each driver.
    pub lpbk_mode: u32,
    /// Port RX filtering configuration (union).
    pub rx_adv_conf: Option<RxAdvConf>,
    /// Port TX DCB configuration (union).
    pub tx_adv_conf: Option<TxAdvConf>,
    /// Currently,Priority Flow Control(PFC) are supported,
    /// if DCB with PFC is needed, and the variable must be set ETH_DCB_PFC_SUPPORT.
    pub dcb_capability_en: u32,
    pub fdir_conf: Option<ffi::rte_fdir_conf>,
    pub intr_conf: Option<ffi::rte_intr_conf>,
}

pub type RawEthConfPtr = *const ffi::rte_eth_conf;

pub struct RawEthConf(ffi::rte_eth_conf);

impl RawEthConf {
    fn as_raw(&self) -> RawEthConfPtr {
        &self.0
    }
}

impl From<&EthConf> for RawEthConf {
    fn from(c: &EthConf) -> Self {
        let mut conf: ffi::rte_eth_conf = Default::default();

        if let Some(ref rxmode) = c.rxmode {
            conf.rxmode = *rxmode
        }

        if let Some(ref txmode) = c.txmode {
            conf.txmode = *txmode
        }

        if let Some(ref adv_conf) = c.rx_adv_conf {
            if let Some(ref rss_conf) = adv_conf.rss_conf {
                let (rss_key, rss_key_len) =
                    rss_conf.key.map_or_else(|| (ptr::null(), 0), |key| (key.as_ptr(), key.len() as u8));

                conf.rx_adv_conf.rss_conf.rss_key = rss_key as *mut _;
                conf.rx_adv_conf.rss_conf.rss_key_len = rss_key_len;
                conf.rx_adv_conf.rss_conf.rss_hf = rss_conf.hash.bits();
            }
        }

        RawEthConf(conf)
    }
}

pub fn foreach() -> impl Iterator<Item = PortId> {
    // todo: discuss with dolev
    (0..RTE_MAX_ETHPORTS as PortId)
        .filter(|port_id| port_id.is_valid() && port_id.get_owner_id() == ffi::RTE_ETH_DEV_NO_OWNER as u64)
}
