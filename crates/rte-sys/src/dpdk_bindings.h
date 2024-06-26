/// known issues:
// 1. https://github.com/rust-lang/rust/issues/54341

#include <rte_eal.h>
#include <rte_errno.h>
#include <rte_ethdev.h>
#include <rte_lcore.h>
#include <rte_malloc.h>

#include "consts.h"

// Used for testing to initialize lcore ids for all threads while running in parallel
void _rte_set_mock_lcore(uint32_t lcore_id);

// bindgen can't generate bindings for static functions defined in C
// header files. these shims are necessary to expose them to FFI.

unsigned _rte_lcore_id(void);

/**
 * Error number value, stored per-thread, which can be queried after
 * calls to certain functions to determine why those functions failed.
 */
int _rte_errno(void);

/**
 * Allocate a new mbuf from a mempool.
 */
char *_rte_pktmbuf_prepend(struct rte_mbuf *m, uint16_t len);

/**
 * Allocate a new mbuf from a mempool.
 */
struct rte_mbuf *_rte_pktmbuf_alloc(struct rte_mempool *mp);

/**
 * Free a packet mbuf back into its original mempool.
 */
void _rte_pktmbuf_free(struct rte_mbuf *m);

/**
 * Put several objects back in the mempool.
 */
void _rte_mempool_put_bulk(struct rte_mempool *mp, void *const *obj_table, unsigned int n);

/**
 * Retrieve a burst of input packets from a receive queue of an Ethernet
 * device. The retrieved packets are stored in *rte_mbuf* structures whose
 * pointers are supplied in the *rx_pkts* array.
 */
uint16_t _rte_eth_rx_burst(uint16_t port_id, uint16_t queue_id, struct rte_mbuf **rx_pkts, const uint16_t nb_pkts);

/**
 * Send a burst of output packets on a transmit queue of an Ethernet device.
 */
uint16_t _rte_eth_tx_burst(uint16_t port_id, uint16_t queue_id, struct rte_mbuf **tx_pkts, uint16_t nb_pkts);

/**
 * Get the application private size of mbufs stored in a pktmbuf_pool.
 */
uint16_t _rte_pktmbuf_priv_size(struct rte_mempool *mp);

/**
 * Get the data room size of mbufs stored in a pktmbuf_pool.
 */
uint16_t _rte_pktmbuf_data_room_size(struct rte_mempool *mp);
