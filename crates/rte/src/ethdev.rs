use std::{ffi::CStr, mem, ops::Range, ptr};

use bitflags::bitflags;
use ffi::RTE_MAX_ETHPORTS;
use mac_addr::MacAddr;
use rte_error::ReturnValue as _;

use crate::{ether, lcore::SocketId, mbuf, mempool, utils::AsRaw, Result};

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
    fn set_mac_addr(&self, addr: &[u8; ether::ETHER_ADDR_LEN]) -> Result<&Self>;

    /// Return the NUMA socket to which an Ethernet device is connected
    fn socket_id(&self) -> SocketId;

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
    fn rx_burst(&self, queue_id: QueueId, rx_pkts: &mut [mbuf::RawMBufPtr]) -> usize;

    /// Send a burst of output packets on a transmit queue of an Ethernet device.
    fn tx_burst(&self, queue_id: QueueId, tx_pkts: &mut [mbuf::RawMBufPtr]) -> usize;

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

    fn set_mac_addr(&self, addr: &[u8; ether::ETHER_ADDR_LEN]) -> Result<&Self> {
        unsafe { ffi::rte_eth_dev_default_mac_addr_set(*self, addr.as_ptr() as *mut _) }.rte_ok()?;
        Ok(self)
    }

    fn socket_id(&self) -> SocketId {
        unsafe { ffi::rte_eth_dev_socket_id(*self) }
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
                self.socket_id() as u32,
                rx_conf.as_ref().map(|conf| conf as *const _).unwrap_or(ptr::null()),
                mb_pool.as_raw(),
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
                self.socket_id() as u32,
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
    fn rx_burst(&self, queue_id: QueueId, rx_pkts: &mut [mbuf::RawMBufPtr]) -> usize {
        unsafe { ffi::_rte_eth_rx_burst(*self, queue_id, rx_pkts.as_mut_ptr() as _, rx_pkts.len() as u16) as usize }
    }

    #[inline]
    fn tx_burst(&self, queue_id: QueueId, tx_pkts: &mut [mbuf::RawMBufPtr]) -> usize {
        unsafe { ffi::_rte_eth_tx_burst(*self, queue_id, tx_pkts.as_mut_ptr() as _, tx_pkts.len() as u16) as usize }
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

pub trait EthDeviceInfo {
    /// Device Driver name.
    fn driver_name(&self) -> &str;
}

pub type RawEthDeviceInfo = ffi::rte_eth_dev_info;

impl EthDeviceInfo for RawEthDeviceInfo {
    #[inline]
    fn driver_name(&self) -> &str {
        unsafe { CStr::from_ptr(self.driver_name).to_str().unwrap() }
    }
}

pub trait EthDeviceStats {}

pub type RawEthDeviceStats = ffi::rte_eth_stats;

impl EthDeviceStats for RawEthDeviceStats {}

bitflags! {
    /// Definitions used for VMDQ pool rx mode setting
    pub struct EthVmdqRxMode : u16 {
        /// accept untagged packets.
        const ETH_VMDQ_ACCEPT_UNTAG     = 0x0001;
        /// accept packets in multicast table .
        const ETH_VMDQ_ACCEPT_HASH_MC   = 0x0002;
        /// accept packets in unicast table.
        const ETH_VMDQ_ACCEPT_HASH_UC   = 0x0004;
        /// accept broadcast packets.
        const ETH_VMDQ_ACCEPT_BROADCAST = 0x0008;
        /// multicast promiscuous.
        const ETH_VMDQ_ACCEPT_MULTICAST = 0x0010;
    }
}

bitflags! {
    /// A set of values to identify what method is to be used to route packets to multiple queues.
    pub struct EthRxMultiQueueMode: u32 {
        const ETH_MQ_RX_RSS_FLAG    = 0x1;
        const ETH_MQ_RX_DCB_FLAG    = 0x2;
        const ETH_MQ_RX_VMDQ_FLAG   = 0x4;
    }
}

bitflags! {
    /// Definitions used for VLAN Offload functionality
    pub struct EthVlanOffloadMode: i32 {
        /// VLAN Strip  On/Off
        const ETH_VLAN_STRIP_OFFLOAD  = 0x0001;
        /// VLAN Filter On/Off
        const ETH_VLAN_FILTER_OFFLOAD = 0x0002;
        /// VLAN Extend On/Off
        const ETH_VLAN_EXTEND_OFFLOAD = 0x0004;

        /// VLAN Strip  setting mask
        const ETH_VLAN_STRIP_MASK     = 0x0001;
        /// VLAN Filter  setting mask
        const ETH_VLAN_FILTER_MASK    = 0x0002;
        /// VLAN Extend  setting mask
        const ETH_VLAN_EXTEND_MASK    = 0x0004;
        /// VLAN ID is in lower 12 bits
        const ETH_VLAN_ID_MAX         = 0x0FFF;
    }
}

bitflags! {
    pub struct RxOffloadHashFunc: u32 {
        const RX_OFFLOAD_VLAN_STRIP = ffi::DEV_RX_OFFLOAD_VLAN_STRIP;
        const RX_OFFLOAD_IPV4_CKSUM = ffi::DEV_RX_OFFLOAD_IPV4_CKSUM;
        const RX_OFFLOAD_UDP_CKSUM = ffi::DEV_RX_OFFLOAD_UDP_CKSUM;
        const RX_OFFLOAD_TCP_CKSUM = ffi::DEV_RX_OFFLOAD_TCP_CKSUM;
        const RX_OFFLOAD_TCP_LRO = ffi::DEV_RX_OFFLOAD_TCP_LRO;
        const RX_OFFLOAD_QINQ_STRIP = ffi::DEV_RX_OFFLOAD_QINQ_STRIP;
        const RX_OFFLOAD_OUTER_IPV4_CKS = ffi::DEV_RX_OFFLOAD_OUTER_IPV4_CKSUM;
        const RX_OFFLOAD_MACSEC_STRIP = ffi::DEV_RX_OFFLOAD_MACSEC_STRIP;
        const RX_OFFLOAD_HEADER_SPLIT = ffi::DEV_RX_OFFLOAD_HEADER_SPLIT;
        const RX_OFFLOAD_VLAN_FILTER = ffi::DEV_RX_OFFLOAD_VLAN_FILTER;
        const RX_OFFLOAD_VLAN_EXTEND = ffi::DEV_RX_OFFLOAD_VLAN_EXTEND;
        const RX_OFFLOAD_JUMBO_FRAME = ffi::DEV_RX_OFFLOAD_JUMBO_FRAME;
        const RX_OFFLOAD_SCATTER = ffi::DEV_RX_OFFLOAD_SCATTER;
        const RX_OFFLOAD_TIMESTAMP = ffi::DEV_RX_OFFLOAD_TIMESTAMP;
        const RX_OFFLOAD_SECURITY = ffi::DEV_RX_OFFLOAD_SECURITY;
        const RX_OFFLOAD_KEEP_CRC = ffi::DEV_RX_OFFLOAD_KEEP_CRC;
        const RX_OFFLOAD_SCTP_CKSUM = ffi::DEV_RX_OFFLOAD_SCTP_CKSUM;
        const RX_OFFLOAD_OUTER_UDP_CKSU = ffi::DEV_RX_OFFLOAD_OUTER_UDP_CKSUM;
        const RX_OFFLOAD_RSS_HASH = ffi::DEV_RX_OFFLOAD_RSS_HASH;
    }
}

bitflags! {
    pub struct TxOffloadsHashFunc: u32 {
        const TX_OFFLOAD_VLAN_INSERT = ffi::DEV_TX_OFFLOAD_VLAN_INSERT;
        const TX_OFFLOAD_IPV4_CKSUM = ffi::DEV_TX_OFFLOAD_IPV4_CKSUM;
        const TX_OFFLOAD_UDP_CKSUM = ffi::DEV_TX_OFFLOAD_UDP_CKSUM;
        const TX_OFFLOAD_TCP_CKSUM = ffi::DEV_TX_OFFLOAD_TCP_CKSUM;
        const TX_OFFLOAD_SCTP_CKSUM = ffi::DEV_TX_OFFLOAD_SCTP_CKSUM;
        const TX_OFFLOAD_TCP_TSO = ffi::DEV_TX_OFFLOAD_TCP_TSO;
        const TX_OFFLOAD_UDP_TSO = ffi::DEV_TX_OFFLOAD_UDP_TSO;
        const TX_OFFLOAD_OUTER_IPV4_CKSUM = ffi::DEV_TX_OFFLOAD_OUTER_IPV4_CKSUM;
        const TX_OFFLOAD_QINQ_INSERT = ffi::DEV_TX_OFFLOAD_QINQ_INSERT;
        const TX_OFFLOAD_VXLAN_TNL_TSO = ffi::DEV_TX_OFFLOAD_VXLAN_TNL_TSO;
        const TX_OFFLOAD_GRE_TNL_TSO = ffi::DEV_TX_OFFLOAD_GRE_TNL_TSO;
        const TX_OFFLOAD_IPIP_TNL_TSO = ffi::DEV_TX_OFFLOAD_IPIP_TNL_TSO;
        const TX_OFFLOAD_GENEVE_TNL_TSO = ffi::DEV_TX_OFFLOAD_GENEVE_TNL_TSO;
        const TX_OFFLOAD_MACSEC_INSERT = ffi::DEV_TX_OFFLOAD_MACSEC_INSERT;
        const TX_OFFLOAD_MT_LOCKFREE = ffi::DEV_TX_OFFLOAD_MT_LOCKFREE;
        const TX_OFFLOAD_MULTI_SEGS = ffi::DEV_TX_OFFLOAD_MULTI_SEGS;
        const TX_OFFLOAD_MBUF_FAST_FREE = ffi::DEV_TX_OFFLOAD_MBUF_FAST_FREE;
        const TX_OFFLOAD_SECURITY = ffi::DEV_TX_OFFLOAD_SECURITY;
        const TX_OFFLOAD_UDP_TNL_TSO = ffi::DEV_TX_OFFLOAD_UDP_TNL_TSO;
        const TX_OFFLOAD_IP_TNL_TSO = ffi::DEV_TX_OFFLOAD_IP_TNL_TSO;
        const TX_OFFLOAD_OUTER_UDP_CKSUM = ffi::DEV_TX_OFFLOAD_OUTER_UDP_CKSUM;
        const TX_OFFLOAD_SEND_ON_TIMESTAMP = ffi::DEV_TX_OFFLOAD_SEND_ON_TIMESTAMP;
    }
}

bitflags! {
    pub struct PktTxOffloadHashFunc: u64 {
        const PKT_TX_OUTER_UDP_CKSUM = ffi::PKT_TX_OUTER_UDP_CKSUM;
        const PKT_TX_UDP_SEG = ffi::PKT_TX_UDP_SEG;
        const PKT_TX_SEC_OFFLOAD = ffi::PKT_TX_SEC_OFFLOAD;
        const PKT_TX_MACSEC = ffi::PKT_TX_MACSEC;
        const PKT_TX_TUNNEL_VXLAN = ffi::PKT_TX_TUNNEL_VXLAN;
        const PKT_TX_TUNNEL_GRE = ffi::PKT_TX_TUNNEL_GRE;
        const PKT_TX_TUNNEL_IPIP = ffi::PKT_TX_TUNNEL_IPIP;
        const PKT_TX_TUNNEL_GENEVE = ffi::PKT_TX_TUNNEL_GENEVE;
        const PKT_TX_TUNNEL_MPLSINUDP = ffi::PKT_TX_TUNNEL_MPLSINUDP;
        const PKT_TX_TUNNEL_VXLAN_GPE = ffi::PKT_TX_TUNNEL_VXLAN_GPE;
        const PKT_TX_TUNNEL_GTP = ffi::PKT_TX_TUNNEL_GTP;
        const PKT_TX_TUNNEL_IP = ffi::PKT_TX_TUNNEL_IP;
        const PKT_TX_TUNNEL_UDP = ffi::PKT_TX_TUNNEL_UDP;
        const PKT_TX_TUNNEL_MASK = ffi::PKT_TX_TUNNEL_MASK;
        const PKT_TX_QINQ = ffi::PKT_TX_QINQ;
        const PKT_TX_QINQ_PKT = ffi::PKT_TX_QINQ_PKT;
        const PKT_TX_TCP_SEG = ffi::PKT_TX_TCP_SEG;
        const PKT_TX_IEEE1588_TMST = ffi::PKT_TX_IEEE1588_TMST;
        const PKT_TX_TCP_CKSUM = ffi::PKT_TX_TCP_CKSUM;
        const PKT_TX_SCTP_CKSUM = ffi::PKT_TX_SCTP_CKSUM;
        const PKT_TX_UDP_CKSUM = ffi::PKT_TX_UDP_CKSUM;
        const PKT_TX_L4_MASK = ffi::PKT_TX_L4_MASK;
        const PKT_TX_IP_CKSUM = ffi::PKT_TX_IP_CKSUM;
        const PKT_TX_IPV4 = ffi::PKT_TX_IPV4;
        const PKT_TX_IPV6 = ffi::PKT_TX_IPV6;
        const PKT_TX_VLAN = ffi::PKT_TX_VLAN;
        const PKT_TX_VLAN_PKT = ffi::PKT_TX_VLAN_PKT;
        const PKT_TX_OUTER_IP_CKSUM = ffi::PKT_TX_OUTER_IP_CKSUM;
        const PKT_TX_OUTER_IPV4 = ffi::PKT_TX_OUTER_IPV4;
        const PKT_TX_OUTER_IPV6 = ffi::PKT_TX_OUTER_IPV6;
        const PKT_TX_OFFLOAD_MASK = ffi::PKT_TX_OFFLOAD_MASK;

        // Flags to enable offloading of IPv4 header checksum calculation
        const IPV4_HDR_CHECKSUM =
            Self::PKT_TX_IP_CKSUM.bits |
            Self::PKT_TX_IPV4.bits;

        // TODO - offload TCP checksum calculation https://msazure.visualstudio.com/One/_workitems/edit/14335353
        // Flags to enable offloading of TCP (over IPv4) and IPv4 header checksums calculations
        const IPV4_HDR_TCP_CHECKSUMS =
            Self::PKT_TX_IP_CKSUM.bits |
            Self::PKT_TX_TCP_CKSUM.bits |
            Self::PKT_TX_IPV4.bits;

        // TODO - offload TCP checksum calculation https://msazure.visualstudio.com/One/_workitems/edit/14335353
        // Flags to enable offloading of TCP (over IPv6) checksums calculation (there is no checksum in IPv6 headers)
        const IPV6_TCP_CHECKSUMS =
            Self::PKT_TX_TCP_CKSUM.bits |
            Self::PKT_TX_IPV6.bits;
    }
}
/**
 * A set of values to identify what method is to be used to transmit
 * packets using multi-TCs.
 */
pub type EthTxMultiQueueMode = ffi::rte_eth_tx_mq_mode::Type;

bitflags! {
    /// The RSS offload types are defined based on flow types which are defined
    /// in rte_eth_ctrl.h. Different NIC hardwares may support different RSS offload
    /// types. The supported flow types or RSS offload types can be queried by
    /// rte_eth_dev_info_get().
    pub struct RssHashFunc: u64 {
        const ETH_RSS_IPV4               = 1 << ffi::RTE_ETH_FLOW_IPV4;
        const ETH_RSS_FRAG_IPV4          = 1 << ffi::RTE_ETH_FLOW_FRAG_IPV4;
        const ETH_RSS_NONFRAG_IPV4_TCP   = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV4_TCP;
        const ETH_RSS_NONFRAG_IPV4_UDP   = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV4_UDP;
        const ETH_RSS_NONFRAG_IPV4_SCTP  = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV4_SCTP;
        const ETH_RSS_NONFRAG_IPV4_OTHER = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV4_OTHER;
        const ETH_RSS_IPV6               = 1 << ffi::RTE_ETH_FLOW_IPV6;
        const ETH_RSS_FRAG_IPV6          = 1 << ffi::RTE_ETH_FLOW_FRAG_IPV6;
        const ETH_RSS_NONFRAG_IPV6_TCP   = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV6_TCP;
        const ETH_RSS_NONFRAG_IPV6_UDP   = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV6_UDP;
        const ETH_RSS_NONFRAG_IPV6_SCTP  = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV6_SCTP;
        const ETH_RSS_NONFRAG_IPV6_OTHER = 1 << ffi::RTE_ETH_FLOW_NONFRAG_IPV6_OTHER;
        const ETH_RSS_L2_PAYLOAD         = 1 << ffi::RTE_ETH_FLOW_L2_PAYLOAD;
        const ETH_RSS_IPV6_EX            = 1 << ffi::RTE_ETH_FLOW_IPV6_EX;
        const ETH_RSS_IPV6_TCP_EX        = 1 << ffi::RTE_ETH_FLOW_IPV6_TCP_EX;
        const ETH_RSS_IPV6_UDP_EX        = 1 << ffi::RTE_ETH_FLOW_IPV6_UDP_EX;

        const ETH_RSS_IP =
            Self::ETH_RSS_IPV4.bits |
            Self::ETH_RSS_FRAG_IPV4.bits |
            Self::ETH_RSS_NONFRAG_IPV4_OTHER.bits |
            Self::ETH_RSS_IPV6.bits |
            Self::ETH_RSS_FRAG_IPV6.bits |
            Self::ETH_RSS_NONFRAG_IPV6_OTHER.bits |
            Self::ETH_RSS_IPV6_EX.bits;

        const ETH_RSS_UDP =
            Self::ETH_RSS_NONFRAG_IPV4_UDP.bits |
            Self::ETH_RSS_NONFRAG_IPV6_UDP.bits |
            Self::ETH_RSS_IPV6_UDP_EX.bits;

        const ETH_RSS_TCP =
            Self::ETH_RSS_NONFRAG_IPV4_TCP.bits |
            Self::ETH_RSS_NONFRAG_IPV6_TCP.bits |
            Self::ETH_RSS_IPV6_TCP_EX.bits;

        const ETH_RSS_SCTP =
            Self::ETH_RSS_NONFRAG_IPV4_SCTP.bits |
            Self::ETH_RSS_NONFRAG_IPV6_SCTP.bits;

        /**< Mask of valid RSS hash protocols */
        const ETH_RSS_PROTO_MASK =
            Self::ETH_RSS_IPV4.bits |
            Self::ETH_RSS_FRAG_IPV4.bits |
            Self::ETH_RSS_NONFRAG_IPV4_TCP.bits |
            Self::ETH_RSS_NONFRAG_IPV4_UDP.bits |
            Self::ETH_RSS_NONFRAG_IPV4_SCTP.bits |
            Self::ETH_RSS_NONFRAG_IPV4_OTHER.bits |
            Self::ETH_RSS_IPV6.bits |
            Self::ETH_RSS_FRAG_IPV6.bits |
            Self::ETH_RSS_NONFRAG_IPV6_TCP.bits |
            Self::ETH_RSS_NONFRAG_IPV6_UDP.bits |
            Self::ETH_RSS_NONFRAG_IPV6_SCTP.bits |
            Self::ETH_RSS_NONFRAG_IPV6_OTHER.bits |
            Self::ETH_RSS_L2_PAYLOAD.bits |
            Self::ETH_RSS_IPV6_EX.bits |
            Self::ETH_RSS_IPV6_TCP_EX.bits |
            Self::ETH_RSS_IPV6_UDP_EX.bits;
    }
}

#[derive(Clone, Copy)]
pub struct EthRssConf {
    pub key: Option<[u8; 40]>,
    pub hash: RssHashFunc,
}

impl Default for EthRssConf {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
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

bitflags! {
    /// Device supported speeds bitmap flags
    pub struct LinkSpeed: u32 {
        /**< Autonegotiate (all speeds) */
        const ETH_LINK_SPEED_AUTONEG  = 0;
        /**< Disable autoneg (fixed speed) */
        const ETH_LINK_SPEED_FIXED    = 1 <<  0;
        /**<  10 Mbps half-duplex */
        const ETH_LINK_SPEED_10M_HD   = 1 <<  1;
         /**<  10 Mbps full-duplex */
        const ETH_LINK_SPEED_10M      = 1 <<  2;
        /**< 100 Mbps half-duplex */
        const ETH_LINK_SPEED_100M_HD  = 1 <<  3;
        /**< 100 Mbps full-duplex */
        const ETH_LINK_SPEED_100M     = 1 <<  4;
        const ETH_LINK_SPEED_1G       = 1 <<  5;
        const ETH_LINK_SPEED_2_5G     = 1 <<  6;
        const ETH_LINK_SPEED_5G       = 1 <<  7;
        const ETH_LINK_SPEED_10G      = 1 <<  8;
        const ETH_LINK_SPEED_20G      = 1 <<  9;
        const ETH_LINK_SPEED_25G      = 1 << 10;
        const ETH_LINK_SPEED_40G      = 1 << 11;
        const ETH_LINK_SPEED_50G      = 1 << 12;
        const ETH_LINK_SPEED_56G      = 1 << 13;
        const ETH_LINK_SPEED_100G     = 1 << 14;
    }
}

impl Default for LinkSpeed {
    fn default() -> Self {
        LinkSpeed::ETH_LINK_SPEED_AUTONEG
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
    pub link_speeds: LinkSpeed,
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

impl<'a> From<&'a EthConf> for RawEthConf {
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
                conf.rx_adv_conf.rss_conf.rss_hf = rss_conf.hash.bits;
            }
        }

        RawEthConf(conf)
    }
}

pub fn foreach() -> impl Iterator<Item = PortId> {
    (0..RTE_MAX_ETHPORTS as PortId)
        .filter(|port_id| port_id.is_valid() && port_id.get_owner_id() == ffi::RTE_ETH_DEV_NO_OWNER as u64)
}
