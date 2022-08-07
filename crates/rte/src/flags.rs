use bitflags::bitflags;

bitflags! {
    /// Definitions used for VMDQ pool rx mode setting
    pub struct EthVmdq : u32 {
        /// accept untagged packets.
        const ACCEPT_UNTAG     = ffi::ETH_VMDQ_ACCEPT_UNTAG;
        /// accept packets in multicast table .
        const ACCEPT_HASH_MC   = ffi::ETH_VMDQ_ACCEPT_HASH_MC;
        /// accept packets in unicast table.
        const ACCEPT_HASH_UC   = ffi::ETH_VMDQ_ACCEPT_HASH_UC;
        /// accept broadcast packets.
        const ACCEPT_BROADCAST = ffi::ETH_VMDQ_ACCEPT_BROADCAST;
        /// multicast promiscuous.
        const ACCEPT_MULTICAST = ffi::ETH_VMDQ_ACCEPT_MULTICAST;
    }
}

bitflags! {
    /// A set of values to identify what method is to be used to route packets to multiple queues.
    pub struct EthMqRxMode: u32 {
        const RSS_FLAG    = ffi::ETH_MQ_RX_RSS_FLAG;
        const DCB_FLAG    = ffi::ETH_MQ_RX_DCB_FLAG;
        const VMDQ_FLAG   = ffi::ETH_MQ_RX_VMDQ_FLAG;
    }
}

bitflags! {
    /// Definitions used for VLAN Offload functionality
    pub struct EthVlanOffload: u32 {
        /// VLAN Strip  On/Off
        const STRIP_OFFLOAD  = ffi::ETH_VLAN_STRIP_OFFLOAD;
        /// VLAN Filter On/Off
        const FILTER_OFFLOAD = ffi::ETH_VLAN_FILTER_OFFLOAD;
        /// VLAN Extend On/Off
        const EXTEND_OFFLOAD = ffi::ETH_VLAN_EXTEND_OFFLOAD;
    }
}

bitflags! {
    pub struct DevRxOffload: u32 {
        const VLAN_STRIP        = ffi::DEV_RX_OFFLOAD_VLAN_STRIP;
        const IPV4_CKSUM        = ffi::DEV_RX_OFFLOAD_IPV4_CKSUM;
        const UDP_CKSUM         = ffi::DEV_RX_OFFLOAD_UDP_CKSUM;
        const TCP_CKSUM         = ffi::DEV_RX_OFFLOAD_TCP_CKSUM;
        const TCP_LRO           = ffi::DEV_RX_OFFLOAD_TCP_LRO;
        const QINQ_STRIP        = ffi::DEV_RX_OFFLOAD_QINQ_STRIP;
        const OUTER_IPV4_CKS    = ffi::DEV_RX_OFFLOAD_OUTER_IPV4_CKSUM;
        const MACSEC_STRIP      = ffi::DEV_RX_OFFLOAD_MACSEC_STRIP;
        const HEADER_SPLIT      = ffi::DEV_RX_OFFLOAD_HEADER_SPLIT;
        const VLAN_FILTER       = ffi::DEV_RX_OFFLOAD_VLAN_FILTER;
        const VLAN_EXTEND       = ffi::DEV_RX_OFFLOAD_VLAN_EXTEND;
        const JUMBO_FRAME       = ffi::DEV_RX_OFFLOAD_JUMBO_FRAME;
        const SCATTER           = ffi::DEV_RX_OFFLOAD_SCATTER;
        const TIMESTAMP         = ffi::DEV_RX_OFFLOAD_TIMESTAMP;
        const SECURITY          = ffi::DEV_RX_OFFLOAD_SECURITY;
        const KEEP_CRC          = ffi::DEV_RX_OFFLOAD_KEEP_CRC;
        const SCTP_CKSUM        = ffi::DEV_RX_OFFLOAD_SCTP_CKSUM;
        const OUTER_UDP_CKSU    = ffi::DEV_RX_OFFLOAD_OUTER_UDP_CKSUM;
        const RSS_HASH          = ffi::DEV_RX_OFFLOAD_RSS_HASH;
        const BUFFER_SPLIT      = ffi::RTE_ETH_RX_OFFLOAD_BUFFER_SPLIT;
        const OFFLOAD_CHECKSUM  = ffi::DEV_RX_OFFLOAD_CHECKSUM;
        const OFFLOAD_VLAN      = ffi::DEV_RX_OFFLOAD_VLAN;
    }
}

bitflags! {
    pub struct DevTxOffload: u32 {
        const VLAN_INSERT       = ffi::DEV_TX_OFFLOAD_VLAN_INSERT;
        const IPV4_CKSUM        = ffi::DEV_TX_OFFLOAD_IPV4_CKSUM;
        const UDP_CKSUM         = ffi::DEV_TX_OFFLOAD_UDP_CKSUM;
        const TCP_CKSUM         = ffi::DEV_TX_OFFLOAD_TCP_CKSUM;
        const SCTP_CKSUM        = ffi::DEV_TX_OFFLOAD_SCTP_CKSUM;
        const TCP_TSO           = ffi::DEV_TX_OFFLOAD_TCP_TSO;
        const UDP_TSO           = ffi::DEV_TX_OFFLOAD_UDP_TSO;
        const OUTER_IPV4_CKSUM  = ffi::DEV_TX_OFFLOAD_OUTER_IPV4_CKSUM;
        const QINQ_INSERT       = ffi::DEV_TX_OFFLOAD_QINQ_INSERT;
        const VXLAN_TNL_TSO     = ffi::DEV_TX_OFFLOAD_VXLAN_TNL_TSO;
        const GRE_TNL_TSO       = ffi::DEV_TX_OFFLOAD_GRE_TNL_TSO;
        const IPIP_TNL_TSO      = ffi::DEV_TX_OFFLOAD_IPIP_TNL_TSO;
        const GENEVE_TNL_TSO    = ffi::DEV_TX_OFFLOAD_GENEVE_TNL_TSO;
        const MACSEC_INSERT     = ffi::DEV_TX_OFFLOAD_MACSEC_INSERT;
        const MT_LOCKFREE       = ffi::DEV_TX_OFFLOAD_MT_LOCKFREE;
        const MULTI_SEGS        = ffi::DEV_TX_OFFLOAD_MULTI_SEGS;
        const MBUF_FAST_FREE    = ffi::DEV_TX_OFFLOAD_MBUF_FAST_FREE;
        const SECURITY          = ffi::DEV_TX_OFFLOAD_SECURITY;
        const UDP_TNL_TSO       = ffi::DEV_TX_OFFLOAD_UDP_TNL_TSO;
        const IP_TNL_TSO        = ffi::DEV_TX_OFFLOAD_IP_TNL_TSO;
        const OUTER_UDP_CKSUM   = ffi::DEV_TX_OFFLOAD_OUTER_UDP_CKSUM;
        const SEND_ON_TIMESTAMP = ffi::DEV_TX_OFFLOAD_SEND_ON_TIMESTAMP;
    }
}

bitflags! {
    pub struct PktTxOffload: u64 {
        const OUTER_UDP_CKSUM    = ffi::PKT_TX_OUTER_UDP_CKSUM;
        const UDP_SEG            = ffi::PKT_TX_UDP_SEG;
        const SEC_OFFLOAD        = ffi::PKT_TX_SEC_OFFLOAD;
        const MACSEC             = ffi::PKT_TX_MACSEC;
        const TUNNEL_VXLAN       = ffi::PKT_TX_TUNNEL_VXLAN;
        const TUNNEL_GRE         = ffi::PKT_TX_TUNNEL_GRE;
        const TUNNEL_IPIP        = ffi::PKT_TX_TUNNEL_IPIP;
        const TUNNEL_GENEVE      = ffi::PKT_TX_TUNNEL_GENEVE;
        const TUNNEL_MPLSINUDP   = ffi::PKT_TX_TUNNEL_MPLSINUDP;
        const TUNNEL_VXLAN_GPE   = ffi::PKT_TX_TUNNEL_VXLAN_GPE;
        const TUNNEL_GTP         = ffi::PKT_TX_TUNNEL_GTP;
        const TUNNEL_IP          = ffi::PKT_TX_TUNNEL_IP;
        const TUNNEL_UDP         = ffi::PKT_TX_TUNNEL_UDP;
        const TUNNEL_MASK        = ffi::PKT_TX_TUNNEL_MASK;
        const QINQ               = ffi::PKT_TX_QINQ;
        const QINQ_PKT           = ffi::PKT_TX_QINQ_PKT;
        const TCP_SEG            = ffi::PKT_TX_TCP_SEG;
        const IEEE1588_TMST      = ffi::PKT_TX_IEEE1588_TMST;
        const TCP_CKSUM          = ffi::PKT_TX_TCP_CKSUM;
        const SCTP_CKSUM         = ffi::PKT_TX_SCTP_CKSUM;
        const UDP_CKSUM          = ffi::PKT_TX_UDP_CKSUM;
        const L4_MASK            = ffi::PKT_TX_L4_MASK;
        const IP_CKSUM           = ffi::PKT_TX_IP_CKSUM;
        const IPV4               = ffi::PKT_TX_IPV4;
        const IPV6               = ffi::PKT_TX_IPV6;
        const VLAN               = ffi::PKT_TX_VLAN;
        const VLAN_PKT           = ffi::PKT_TX_VLAN_PKT;
        const OUTER_IP_CKSUM     = ffi::PKT_TX_OUTER_IP_CKSUM;
        const OUTER_IPV4         = ffi::PKT_TX_OUTER_IPV4;
        const OUTER_IPV6         = ffi::PKT_TX_OUTER_IPV6;
        const OFFLOAD_MASK       = ffi::PKT_TX_OFFLOAD_MASK;

        // Flags to enable offloading of IPv4 header checksum calculation
        const IPV4_HDR_CHECKSUM =
            Self::IP_CKSUM.bits |
            Self::IPV4.bits;

        // TODO - offload TCP checksum calculation https://msazure.visualstudio.com/One/_workitems/edit/14335353
        // Flags to enable offloading of TCP (over IPv4) and IPv4 header checksums calculations
        const IPV4_HDR_TCP_CHECKSUMS =
            Self::IP_CKSUM.bits |
            Self::TCP_CKSUM.bits |
            Self::IPV4.bits;

        // TODO - offload TCP checksum calculation https://msazure.visualstudio.com/One/_workitems/edit/14335353
        // Flags to enable offloading of TCP (over IPv6) checksums calculation (there is no checksum in IPv6 headers)
        const IPV6_TCP_CHECKSUMS =
            Self::TCP_CKSUM.bits |
            Self::IPV6.bits;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct EthRss: u64 {
        const IPV4                  = ffi::ETH_RSS_IPV4 as u64;
        const FRAG_IPV4             = ffi::ETH_RSS_FRAG_IPV4 as u64;
        const NONFRAG_IPV4_TCP      = ffi::ETH_RSS_NONFRAG_IPV4_TCP as u64;
        const NONFRAG_IPV4_UDP      = ffi::ETH_RSS_NONFRAG_IPV4_UDP as u64;
        const NONFRAG_IPV4_SCTP     = ffi::ETH_RSS_NONFRAG_IPV4_SCTP as u64;
        const NONFRAG_IPV4_OTHER    = ffi::ETH_RSS_NONFRAG_IPV4_OTHER as u64;
        const IPV6                  = ffi::ETH_RSS_IPV6 as u64;
        const FRAG_IPV6             = ffi::ETH_RSS_FRAG_IPV6 as u64;
        const NONFRAG_IPV6_TCP      = ffi::ETH_RSS_NONFRAG_IPV6_TCP as u64;
        const NONFRAG_IPV6_UDP      = ffi::ETH_RSS_NONFRAG_IPV6_UDP as u64;
        const NONFRAG_IPV6_SCTP     = ffi::ETH_RSS_NONFRAG_IPV6_SCTP as u64;
        const NONFRAG_IPV6_OTHER    = ffi::ETH_RSS_NONFRAG_IPV6_OTHER as u64;
        const L2_PAYLOAD            = ffi::ETH_RSS_L2_PAYLOAD as u64;
        const IPV6_EX               = ffi::ETH_RSS_IPV6_EX as u64;
        const IPV6_TCP_EX           = ffi::ETH_RSS_IPV6_TCP_EX as u64;
        const IPV6_UDP_EX           = ffi::ETH_RSS_IPV6_UDP_EX as u64;
        const PORT                  = ffi::ETH_RSS_PORT as u64;
        const VXLAN                 = ffi::ETH_RSS_VXLAN as u64;
        const GENEVE                = ffi::ETH_RSS_GENEVE as u64;
        const NVGRE                 = ffi::ETH_RSS_NVGRE as u64;
        const GTPU                  = ffi::ETH_RSS_GTPU as u64;
        const ETH                   = ffi::ETH_RSS_ETH as u64;
        const S_VLAN                = ffi::ETH_RSS_S_VLAN as u64;
        const C_VLAN                = ffi::ETH_RSS_C_VLAN as u64;
        const ESP                   = ffi::ETH_RSS_ESP as u64;
        const AH                    = ffi::ETH_RSS_AH as u64;
        const L2TPV3                = ffi::ETH_RSS_L2TPV3 as u64;
        const PFCP                  = ffi::ETH_RSS_PFCP as u64;
        const PPPOE                 = ffi::ETH_RSS_PPPOE as u64;
        const ECPRI                 = ffi::ETH_RSS_ECPRI as u64;
        const MPLS                  = ffi::ETH_RSS_MPLS as u64;
    }
}

bitflags! {
    /// Device supported speeds bitmap flags
    pub struct EthLinkSpeed: u32 {
        /// Autonegotiate (all speeds)
        const SPEED_AUTONEG  = ffi::ETH_LINK_SPEED_AUTONEG;
        /// Disable autoneg (fixed speed)
        const SPEED_FIXED    = ffi::ETH_LINK_SPEED_FIXED;
        ///  10 Mbps half-duplex
        const SPEED_10M_HD   = ffi::ETH_LINK_SPEED_10M_HD;
        ///  10 Mbps full-duplex
        const SPEED_10M      = ffi::ETH_LINK_SPEED_10M;
        /// 100 Mbps half-duplex
        const SPEED_100M_HD  = ffi::ETH_LINK_SPEED_100M_HD;
        /// 100 Mbps full-duplex
        const SPEED_100M     = ffi::ETH_LINK_SPEED_100M;
        /// 1 Gbps
        const SPEED_1G       = ffi::ETH_LINK_SPEED_1G;
        /// 2.5 Gbps
        const SPEED_2_5G     = ffi::ETH_LINK_SPEED_2_5G;
        /// 5 Gbps
        const SPEED_5G       = ffi::ETH_LINK_SPEED_5G;
        /// 10 Gbps
        const SPEED_10G      = ffi::ETH_LINK_SPEED_10G;
        /// 20 Gbps
        const SPEED_20G      = ffi::ETH_LINK_SPEED_20G;
        /// 25 Gbps
        const SPEED_25G      = ffi::ETH_LINK_SPEED_25G;
        /// 40 Gbps
        const SPEED_40G      = ffi::ETH_LINK_SPEED_40G;
        /// 50 Gbps
        const SPEED_50G      = ffi::ETH_LINK_SPEED_50G;
        /// 56 Gbps
        const SPEED_56G      = ffi::ETH_LINK_SPEED_56G;
        /// 100 Gbps
        const SPEED_100G     = ffi::ETH_LINK_SPEED_100G;
        /// 200 Gbps
        const SPEED_200G     = ffi::ETH_LINK_SPEED_200G;
    }
}

impl Default for EthLinkSpeed {
    fn default() -> Self {
        EthLinkSpeed::SPEED_AUTONEG
    }
}
