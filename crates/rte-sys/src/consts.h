
/** Workaround for https://github.com/rust-lang/rust-bindgen/issues/753 **/

const uint64_t _RTE_ETH_RX_OFFLOAD_VLAN_STRIP           = RTE_ETH_RX_OFFLOAD_VLAN_STRIP;
const uint64_t _RTE_ETH_RX_OFFLOAD_IPV4_CKSUM           = RTE_ETH_RX_OFFLOAD_IPV4_CKSUM;
const uint64_t _RTE_ETH_RX_OFFLOAD_UDP_CKSUM            = RTE_ETH_RX_OFFLOAD_UDP_CKSUM;
const uint64_t _RTE_ETH_RX_OFFLOAD_TCP_CKSUM            = RTE_ETH_RX_OFFLOAD_TCP_CKSUM;
const uint64_t _RTE_ETH_RX_OFFLOAD_TCP_LRO              = RTE_ETH_RX_OFFLOAD_TCP_LRO;
const uint64_t _RTE_ETH_RX_OFFLOAD_QINQ_STRIP           = RTE_ETH_RX_OFFLOAD_QINQ_STRIP;
const uint64_t _RTE_ETH_RX_OFFLOAD_OUTER_IPV4_CKSUM     = RTE_ETH_RX_OFFLOAD_OUTER_IPV4_CKSUM;
const uint64_t _RTE_ETH_RX_OFFLOAD_MACSEC_STRIP         = RTE_ETH_RX_OFFLOAD_MACSEC_STRIP;
const uint64_t _RTE_ETH_RX_OFFLOAD_VLAN_FILTER          = RTE_ETH_RX_OFFLOAD_VLAN_FILTER;
const uint64_t _RTE_ETH_RX_OFFLOAD_VLAN_EXTEND          = RTE_ETH_RX_OFFLOAD_VLAN_EXTEND;
const uint64_t _RTE_ETH_RX_OFFLOAD_SCATTER              = RTE_ETH_RX_OFFLOAD_SCATTER;
const uint64_t _RTE_ETH_RX_OFFLOAD_TIMESTAMP            = RTE_ETH_RX_OFFLOAD_TIMESTAMP;
const uint64_t _RTE_ETH_RX_OFFLOAD_SECURITY             = RTE_ETH_RX_OFFLOAD_SECURITY;
const uint64_t _RTE_ETH_RX_OFFLOAD_KEEP_CRC             = RTE_ETH_RX_OFFLOAD_KEEP_CRC;
const uint64_t _RTE_ETH_RX_OFFLOAD_SCTP_CKSUM           = RTE_ETH_RX_OFFLOAD_SCTP_CKSUM;
const uint64_t _RTE_ETH_RX_OFFLOAD_OUTER_UDP_CKSUM      = RTE_ETH_RX_OFFLOAD_OUTER_UDP_CKSUM;
const uint64_t _RTE_ETH_RX_OFFLOAD_RSS_HASH             = RTE_ETH_RX_OFFLOAD_RSS_HASH;

const uint64_t _RTE_ETH_TX_OFFLOAD_VLAN_INSERT          = RTE_ETH_TX_OFFLOAD_VLAN_INSERT;
const uint64_t _RTE_ETH_TX_OFFLOAD_IPV4_CKSUM           = RTE_ETH_TX_OFFLOAD_IPV4_CKSUM;
const uint64_t _RTE_ETH_TX_OFFLOAD_UDP_CKSUM            = RTE_ETH_TX_OFFLOAD_UDP_CKSUM;
const uint64_t _RTE_ETH_TX_OFFLOAD_TCP_CKSUM            = RTE_ETH_TX_OFFLOAD_TCP_CKSUM;
const uint64_t _RTE_ETH_TX_OFFLOAD_SCTP_CKSUM           = RTE_ETH_TX_OFFLOAD_SCTP_CKSUM;
const uint64_t _RTE_ETH_TX_OFFLOAD_TCP_TSO              = RTE_ETH_TX_OFFLOAD_TCP_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_UDP_TSO              = RTE_ETH_TX_OFFLOAD_UDP_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_OUTER_IPV4_CKSUM     = RTE_ETH_TX_OFFLOAD_OUTER_IPV4_CKSUM;
const uint64_t _RTE_ETH_TX_OFFLOAD_QINQ_INSERT          = RTE_ETH_TX_OFFLOAD_QINQ_INSERT;
const uint64_t _RTE_ETH_TX_OFFLOAD_VXLAN_TNL_TSO        = RTE_ETH_TX_OFFLOAD_VXLAN_TNL_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_GRE_TNL_TSO          = RTE_ETH_TX_OFFLOAD_GRE_TNL_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_IPIP_TNL_TSO         = RTE_ETH_TX_OFFLOAD_IPIP_TNL_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_GENEVE_TNL_TSO       = RTE_ETH_TX_OFFLOAD_GENEVE_TNL_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_MACSEC_INSERT        = RTE_ETH_TX_OFFLOAD_MACSEC_INSERT;
const uint64_t _RTE_ETH_TX_OFFLOAD_MT_LOCKFREE          = RTE_ETH_TX_OFFLOAD_MT_LOCKFREE;
const uint64_t _RTE_ETH_TX_OFFLOAD_MULTI_SEGS           = RTE_ETH_TX_OFFLOAD_MULTI_SEGS;
const uint64_t _RTE_ETH_TX_OFFLOAD_MBUF_FAST_FREE       = RTE_ETH_TX_OFFLOAD_MBUF_FAST_FREE;
const uint64_t _RTE_ETH_TX_OFFLOAD_SECURITY             = RTE_ETH_TX_OFFLOAD_SECURITY;
const uint64_t _RTE_ETH_TX_OFFLOAD_UDP_TNL_TSO          = RTE_ETH_TX_OFFLOAD_UDP_TNL_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_IP_TNL_TSO           = RTE_ETH_TX_OFFLOAD_IP_TNL_TSO;
const uint64_t _RTE_ETH_TX_OFFLOAD_OUTER_UDP_CKSUM      = RTE_ETH_TX_OFFLOAD_OUTER_UDP_CKSUM;
const uint64_t _RTE_ETH_TX_OFFLOAD_SEND_ON_TIMESTAMP    = RTE_ETH_TX_OFFLOAD_SEND_ON_TIMESTAMP;

const uint32_t _RTE_ETH_MQ_RX_RSS_FLAG   = RTE_ETH_MQ_RX_RSS_FLAG;
const uint32_t _RTE_ETH_MQ_RX_DCB_FLAG   = RTE_ETH_MQ_RX_DCB_FLAG;
const uint32_t _RTE_ETH_MQ_RX_VMDQ_FLAG  = RTE_ETH_MQ_RX_VMDQ_FLAG;

const uint32_t _RTE_ETH_RSS_IPV4 =                  RTE_ETH_RSS_IPV4;
const uint32_t _RTE_ETH_RSS_FRAG_IPV4 =             RTE_ETH_RSS_FRAG_IPV4;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV4_TCP =      RTE_ETH_RSS_NONFRAG_IPV4_TCP;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV4_UDP =      RTE_ETH_RSS_NONFRAG_IPV4_UDP;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV4_SCTP =     RTE_ETH_RSS_NONFRAG_IPV4_SCTP;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV4_OTHER =    RTE_ETH_RSS_NONFRAG_IPV4_OTHER;
const uint32_t _RTE_ETH_RSS_IPV6 =                  RTE_ETH_RSS_IPV6;
const uint32_t _RTE_ETH_RSS_FRAG_IPV6 =             RTE_ETH_RSS_FRAG_IPV6;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV6_TCP =      RTE_ETH_RSS_NONFRAG_IPV6_TCP;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV6_UDP =      RTE_ETH_RSS_NONFRAG_IPV6_UDP;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV6_SCTP =     RTE_ETH_RSS_NONFRAG_IPV6_SCTP;
const uint32_t _RTE_ETH_RSS_NONFRAG_IPV6_OTHER =    RTE_ETH_RSS_NONFRAG_IPV6_OTHER;
const uint32_t _RTE_ETH_RSS_L2_PAYLOAD =            RTE_ETH_RSS_L2_PAYLOAD;
const uint32_t _RTE_ETH_RSS_IPV6_EX =               RTE_ETH_RSS_IPV6_EX;
const uint32_t _RTE_ETH_RSS_IPV6_TCP_EX =           RTE_ETH_RSS_IPV6_TCP_EX;
const uint32_t _RTE_ETH_RSS_IPV6_UDP_EX =           RTE_ETH_RSS_IPV6_UDP_EX;
const uint32_t _RTE_ETH_RSS_PORT =                  RTE_ETH_RSS_PORT;
const uint32_t _RTE_ETH_RSS_VXLAN =                 RTE_ETH_RSS_VXLAN;
const uint32_t _RTE_ETH_RSS_GENEVE =                RTE_ETH_RSS_GENEVE;
const uint32_t _RTE_ETH_RSS_NVGRE =                 RTE_ETH_RSS_NVGRE;
const uint32_t _RTE_ETH_RSS_GTPU =                  RTE_ETH_RSS_GTPU;
const uint32_t _RTE_ETH_RSS_ETH =                   RTE_ETH_RSS_ETH;
const uint32_t _RTE_ETH_RSS_S_VLAN =                RTE_ETH_RSS_S_VLAN;
const uint32_t _RTE_ETH_RSS_C_VLAN =                RTE_ETH_RSS_C_VLAN;
const uint32_t _RTE_ETH_RSS_ESP =                   RTE_ETH_RSS_ESP;
const uint32_t _RTE_ETH_RSS_AH =                    RTE_ETH_RSS_AH;
const uint32_t _RTE_ETH_RSS_L2TPV3 =                RTE_ETH_RSS_L2TPV3;
const uint32_t _RTE_ETH_RSS_PFCP =                  RTE_ETH_RSS_PFCP;
const uint32_t _RTE_ETH_RSS_PPPOE =                 RTE_ETH_RSS_PPPOE;
const uint32_t _RTE_ETH_RSS_ECPRI =                 RTE_ETH_RSS_ECPRI;
const uint32_t _RTE_ETH_RSS_MPLS =                  RTE_ETH_RSS_MPLS;