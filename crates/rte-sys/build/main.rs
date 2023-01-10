mod linker;

use std::{env, path::PathBuf};

const GENERATED_FILE: &str = "dpdk_bindings.rs";

fn bind() {
    cc::Build::new()
        .file("src/stub.c")
        .flag("-mssse3")
        .compile("rte_stub");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let generated_path = out_dir.join(GENERATED_FILE);

    bindgen::Builder::default()
        .header("src/dpdk_bindings.h")
        .generate_comments(true)
        .generate_inline_functions(true)
        // treat as opaque as per issue w/ combining align/packed:
        // https://github.com/rust-lang/rust-bindgen/issues/1538
        .opaque_type(r"rte_arp_ipv4|rte_arp_hdr")
        // and this struct per this issue:
        // https://github.com/rust-lang/rust-bindgen/issues/2179
        .opaque_type("rte_l2tpv2_combined_msg_hdr")
        .allowlist_type(r"(rte|eth|DDOS)_.*")
        .allowlist_function(r"(_rte|rte|eth)_.*")
        .allowlist_var(r"(_?RTE|EXT|DEV|ETH|MEMPOOL|PKT|LCORE|rte)_.*")
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .clang_arg("-finline-functions")
        .rustfmt_bindings(true)
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&generated_path)
        .expect("Couldn't write bindings!");

    linker::link_dpdk();
}

fn main() {
    bind();

    println!("cargo:rerun-if-changed=src/");
}
