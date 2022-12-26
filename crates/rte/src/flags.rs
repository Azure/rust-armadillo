use bitflags::bitflags;

bitflags! {
    /// A set of values to identify what method is to be used to route packets to multiple queues.
    pub struct EthMqRxMode: u32 {
        const RSS_FLAG    = ffi::_RTE_ETH_MQ_RX_RSS_FLAG;
        const DCB_FLAG    = ffi::_RTE_ETH_MQ_RX_DCB_FLAG;
        const VMDQ_FLAG   = ffi::_RTE_ETH_MQ_RX_VMDQ_FLAG;
    }
}

bitflags! {
    pub struct DevTxOffload: u64 {
        const VLAN_INSERT       = ffi::_RTE_ETH_TX_OFFLOAD_VLAN_INSERT;
        const IPV4_CKSUM        = ffi::_RTE_ETH_TX_OFFLOAD_IPV4_CKSUM;
        const UDP_CKSUM         = ffi::_RTE_ETH_TX_OFFLOAD_UDP_CKSUM;
        const TCP_CKSUM         = ffi::_RTE_ETH_TX_OFFLOAD_TCP_CKSUM;
        const SCTP_CKSUM        = ffi::_RTE_ETH_TX_OFFLOAD_SCTP_CKSUM;
        const TCP_TSO           = ffi::_RTE_ETH_TX_OFFLOAD_TCP_TSO;
        const UDP_TSO           = ffi::_RTE_ETH_TX_OFFLOAD_UDP_TSO;
        const OUTER_IPV4_CKSUM  = ffi::_RTE_ETH_TX_OFFLOAD_OUTER_IPV4_CKSUM;
        const QINQ_INSERT       = ffi::_RTE_ETH_TX_OFFLOAD_QINQ_INSERT;
        const VXLAN_TNL_TSO     = ffi::_RTE_ETH_TX_OFFLOAD_VXLAN_TNL_TSO;
        const GRE_TNL_TSO       = ffi::_RTE_ETH_TX_OFFLOAD_GRE_TNL_TSO;
        const IPIP_TNL_TSO      = ffi::_RTE_ETH_TX_OFFLOAD_IPIP_TNL_TSO;
        const GENEVE_TNL_TSO    = ffi::_RTE_ETH_TX_OFFLOAD_GENEVE_TNL_TSO;
        const MACSEC_INSERT     = ffi::_RTE_ETH_TX_OFFLOAD_MACSEC_INSERT;
        const MT_LOCKFREE       = ffi::_RTE_ETH_TX_OFFLOAD_MT_LOCKFREE;
        const MULTI_SEGS        = ffi::_RTE_ETH_TX_OFFLOAD_MULTI_SEGS;
        const MBUF_FAST_FREE    = ffi::_RTE_ETH_TX_OFFLOAD_MBUF_FAST_FREE;
        const SECURITY          = ffi::_RTE_ETH_TX_OFFLOAD_SECURITY;
        const UDP_TNL_TSO       = ffi::_RTE_ETH_TX_OFFLOAD_UDP_TNL_TSO;
        const IP_TNL_TSO        = ffi::_RTE_ETH_TX_OFFLOAD_IP_TNL_TSO;
        const OUTER_UDP_CKSUM   = ffi::_RTE_ETH_TX_OFFLOAD_OUTER_UDP_CKSUM;
        const SEND_ON_TIMESTAMP = ffi::_RTE_ETH_TX_OFFLOAD_SEND_ON_TIMESTAMP;
    }
}

bitflags! {
    pub struct PktTxOffload: u64 {
        const OUTER_UDP_CKSUM    = ffi::RTE_MBUF_F_TX_OUTER_UDP_CKSUM;
        const UDP_SEG            = ffi::RTE_MBUF_F_TX_UDP_SEG;
        const SEC_OFFLOAD        = ffi::RTE_MBUF_F_TX_SEC_OFFLOAD;
        const MACSEC             = ffi::RTE_MBUF_F_TX_MACSEC;
        const TUNNEL_VXLAN       = ffi::RTE_MBUF_F_TX_TUNNEL_VXLAN;
        const TUNNEL_GRE         = ffi::RTE_MBUF_F_TX_TUNNEL_GRE;
        const TUNNEL_IPIP        = ffi::RTE_MBUF_F_TX_TUNNEL_IPIP;
        const TUNNEL_GENEVE      = ffi::RTE_MBUF_F_TX_TUNNEL_GENEVE;
        const TUNNEL_MPLSINUDP   = ffi::RTE_MBUF_F_TX_TUNNEL_MPLSINUDP;
        const TUNNEL_VXLAN_GPE   = ffi::RTE_MBUF_F_TX_TUNNEL_VXLAN_GPE;
        const TUNNEL_GTP         = ffi::RTE_MBUF_F_TX_TUNNEL_GTP;
        const TUNNEL_IP          = ffi::RTE_MBUF_F_TX_TUNNEL_IP;
        const TUNNEL_UDP         = ffi::RTE_MBUF_F_TX_TUNNEL_UDP;
        const TUNNEL_MASK        = ffi::RTE_MBUF_F_TX_TUNNEL_MASK;
        const QINQ               = ffi::RTE_MBUF_F_TX_QINQ;
        const TCP_SEG            = ffi::RTE_MBUF_F_TX_TCP_SEG;
        const IEEE1588_TMST      = ffi::RTE_MBUF_F_TX_IEEE1588_TMST;
        const TCP_CKSUM          = ffi::RTE_MBUF_F_TX_TCP_CKSUM;
        const SCTP_CKSUM         = ffi::RTE_MBUF_F_TX_SCTP_CKSUM;
        const UDP_CKSUM          = ffi::RTE_MBUF_F_TX_UDP_CKSUM;
        const L4_MASK            = ffi::RTE_MBUF_F_TX_L4_MASK;
        const IP_CKSUM           = ffi::RTE_MBUF_F_TX_IP_CKSUM;
        const IPV4               = ffi::RTE_MBUF_F_TX_IPV4;
        const IPV6               = ffi::RTE_MBUF_F_TX_IPV6;
        const VLAN               = ffi::RTE_MBUF_F_TX_VLAN;
        const OUTER_IP_CKSUM     = ffi::RTE_MBUF_F_TX_OUTER_IP_CKSUM;
        const OUTER_IPV4         = ffi::RTE_MBUF_F_TX_OUTER_IPV4;
        const OUTER_IPV6         = ffi::RTE_MBUF_F_TX_OUTER_IPV6;
        const OFFLOAD_MASK       = ffi::RTE_MBUF_F_TX_OFFLOAD_MASK;

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
        const IPV4                  = ffi::_RTE_ETH_RSS_IPV4 as u64;
        const FRAG_IPV4             = ffi::_RTE_ETH_RSS_FRAG_IPV4 as u64;
        const NONFRAG_IPV4_TCP      = ffi::_RTE_ETH_RSS_NONFRAG_IPV4_TCP as u64;
        const NONFRAG_IPV4_UDP      = ffi::_RTE_ETH_RSS_NONFRAG_IPV4_UDP as u64;
        const NONFRAG_IPV4_SCTP     = ffi::_RTE_ETH_RSS_NONFRAG_IPV4_SCTP as u64;
        const NONFRAG_IPV4_OTHER    = ffi::_RTE_ETH_RSS_NONFRAG_IPV4_OTHER as u64;
        const IPV6                  = ffi::_RTE_ETH_RSS_IPV6 as u64;
        const FRAG_IPV6             = ffi::_RTE_ETH_RSS_FRAG_IPV6 as u64;
        const NONFRAG_IPV6_TCP      = ffi::_RTE_ETH_RSS_NONFRAG_IPV6_TCP as u64;
        const NONFRAG_IPV6_UDP      = ffi::_RTE_ETH_RSS_NONFRAG_IPV6_UDP as u64;
        const NONFRAG_IPV6_SCTP     = ffi::_RTE_ETH_RSS_NONFRAG_IPV6_SCTP as u64;
        const NONFRAG_IPV6_OTHER    = ffi::_RTE_ETH_RSS_NONFRAG_IPV6_OTHER as u64;
        const L2_PAYLOAD            = ffi::_RTE_ETH_RSS_L2_PAYLOAD as u64;
        const IPV6_EX               = ffi::_RTE_ETH_RSS_IPV6_EX as u64;
        const IPV6_TCP_EX           = ffi::_RTE_ETH_RSS_IPV6_TCP_EX as u64;
        const IPV6_UDP_EX           = ffi::_RTE_ETH_RSS_IPV6_UDP_EX as u64;
        const PORT                  = ffi::_RTE_ETH_RSS_PORT as u64;
        const VXLAN                 = ffi::_RTE_ETH_RSS_VXLAN as u64;
        const GENEVE                = ffi::_RTE_ETH_RSS_GENEVE as u64;
        const NVGRE                 = ffi::_RTE_ETH_RSS_NVGRE as u64;
        const GTPU                  = ffi::_RTE_ETH_RSS_GTPU as u64;
        const ETH                   = ffi::_RTE_ETH_RSS_ETH as u64;
        const S_VLAN                = ffi::_RTE_ETH_RSS_S_VLAN as u64;
        const C_VLAN                = ffi::_RTE_ETH_RSS_C_VLAN as u64;
        const ESP                   = ffi::_RTE_ETH_RSS_ESP as u64;
        const AH                    = ffi::_RTE_ETH_RSS_AH as u64;
        const L2TPV3                = ffi::_RTE_ETH_RSS_L2TPV3 as u64;
        const PFCP                  = ffi::_RTE_ETH_RSS_PFCP as u64;
        const PPPOE                 = ffi::_RTE_ETH_RSS_PPPOE as u64;
        const ECPRI                 = ffi::_RTE_ETH_RSS_ECPRI as u64;
        const MPLS                  = ffi::_RTE_ETH_RSS_MPLS as u64;
    }
}

bitflags! {
    /// Device supported speeds bitmap flags
    pub struct EthLinkSpeed: u32 {
        /// Autonegotiate (all speeds)
        const AUTONEG = ffi::RTE_ETH_LINK_AUTONEG;
        /// Disable autoneg (fixed speed)
        const FIXED = ffi::RTE_ETH_LINK_FIXED;
    }
}

impl Default for EthLinkSpeed {
    fn default() -> Self {
        EthLinkSpeed::AUTONEG
    }
}
