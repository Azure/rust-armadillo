use std::{fmt, fs::File, io::Write, path::Path};

/// Prune all unused device drivers from DPDK, cuts binaries size and build time by half.
const IGNORED_STATIC_LIBS: &[&str] = &[
    "rte_baseband_acc100",
    "rte_baseband_fpga_5gnr_fec",
    "rte_baseband_fpga_lte_fec",
    "rte_baseband_null",
    "rte_baseband_turbo_sw",
    "rte_bus_dpaa",
    "rte_bus_fslmc",
    "rte_bus_ifpga",
    "rte_common_cpt",
    "rte_common_dpaax",
    "rte_common_iavf",
    "rte_common_octeontx",
    "rte_common_octeontx2",
    "rte_common_sfc_efx",
    "rte_compress_octeontx",
    "rte_crypto_bcmfs",
    "rte_crypto_caam_jr",
    "rte_crypto_ccp",
    "rte_crypto_dpaa_sec",
    "rte_crypto_dpaa2_sec",
    "rte_crypto_nitrox",
    "rte_crypto_octeontx",
    "rte_crypto_octeontx2",
    "rte_event_dpaa",
    "rte_event_dpaa2",
    "rte_event_dsw",
    "rte_event_octeontx",
    "rte_event_octeontx2",
    "rte_mempool_dpaa",
    "rte_mempool_dpaa2",
    "rte_mempool_octeontx",
    "rte_mempool_octeontx2",
    "rte_net_ark",
    "rte_net_atlantic",
    "rte_net_avp",
    "rte_net_axgbe",
    "rte_net_bnx2x",
    "rte_net_bnxt",
    "rte_net_bond",
    "rte_net_cxgbe",
    "rte_net_dpaa",
    "rte_net_dpaa2",
    "rte_net_ena",
    "rte_net_enetc",
    "rte_net_enic",
    "rte_net_failsafe",
    "rte_net_fm10k",
    "rte_net_hinic",
    "rte_net_hns3",
    "rte_net_i40e",
    "rte_net_iavf",
    "rte_net_ice",
    "rte_net_igc",
    "rte_net_liquidio",
    "rte_net_memif",
    "rte_net_nfp",
    "rte_net_octeontx",
    "rte_net_octeontx2",
    "rte_net_pfe",
    "rte_net_qede",
    "rte_net_sfc",
    "rte_net_thunderx",
    "rte_net_txgbe",
    "rte_raw_dpaa2_cmdif",
    "rte_raw_dpaa2_qdma",
    "rte_raw_octeontx2_dma",
    "rte_raw_octeontx2_ep",
    "rte_regex_octeontx2",
];

enum LinkType {
    Static,
    Dynamic,
}

struct LibLink<'l> {
    name: &'l str,
    link_type: LinkType,
}

impl<'l> LibLink<'l> {
    fn is_static(&self) -> bool {
        matches!(self.link_type, LinkType::Static)
    }
}

impl<'l> fmt::Display for LibLink<'l> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cargo:rustc-link-lib={}{}",
            if self.is_static() {
                // FIXME: "static:+whole-archive,-bundle="
                "static="
            } else {
                ""
            },
            self.name
        )?;
        Ok(())
    }
}

pub fn bind(out_dir: &Path) {
    let pkg = pkg_config::Config::new()
        .exactly_version("21.08.0")
        .statik(true)
        .cargo_metadata(false)
        .probe("libdpdk")
        .unwrap();

    for path in pkg.link_paths {
        println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    }

    // pkg-config returns a list of libs, where static libs are specified as
    // ":librte_mempool_ring.a" and dynamic ones like "rte_mempool", so we'll use that
    // to parse them into two lists
    let (mut static_libs, dyn_libs) = pkg
        .libs
        .iter()
        .map(|lib| {
            lib.strip_prefix(":lib")
                .and_then(|lib| lib.strip_suffix(".a"))
                .map(|lib| LibLink { name: lib, link_type: LinkType::Static })
                .unwrap_or(LibLink { name: lib, link_type: LinkType::Dynamic })
        })
        .partition::<Vec<_>, _>(LibLink::is_static);

    static_libs.retain(|LibLink { name, .. }| !IGNORED_STATIC_LIBS.contains(name));

    for link in static_libs.iter().chain(&dyn_libs) {
        println!("{link}");
    }

    // TODO: https://msazure.visualstudio.com/One/_workitems/edit/13763345
    // once the `whole-archive` feature is stabilized: https://github.com/rust-lang/rust/pull/93901
    // we can:
    // 1. remove the next block of code that generates "whole_archive_hack.rs"
    //    (and the optional arg this function takes)
    // 2. in the `Display` impl for `LibLink`, replace "static=" with "static:+whole-archive,-bundle="
    // 3. remove all usages of this function apart from in the `rte-sys` crate's build script
    let mut out_file = File::create(out_dir.join("whole_archive_hack.rs")).unwrap();
    for LibLink { name, .. } in static_libs {
        writeln!(out_file, r#"#[link(name = "{name}", kind = "static")]"#).unwrap();
    }
    writeln!(out_file, "{}", r#"extern "C" {}"#).unwrap();
}
