use std::{
    env, fs,
    path::{Path, PathBuf},
};

const GENERATED_DIR: &str = "../build/generated";
const GENERATED_FILE: &str = "dpdk_bindings.rs";

fn bind() {
    cc::Build::new().file("src/stub.c").flag("-march=broadwell").compile("rte_stub");

    fs::create_dir_all(GENERATED_DIR).unwrap();

    let generated_path = Path::new(GENERATED_DIR).join(GENERATED_FILE);

    bindgen::Builder::default()
        .header("src/dpdk_bindings.h")
        .generate_comments(true)
        .generate_inline_functions(true)
        // treat as opaque as per issue w/ combining align/packed:
        // https://github.com/rust-lang/rust-bindgen/issues/1538
        .opaque_type(r"rte_arp_ipv4|rte_arp_hdr")
        .allowlist_type(r"(rte|eth|DDOS)_.*")
        .allowlist_function(r"(_rte|rte|eth)_.*")
        .allowlist_var(r"(RTE|DEV|ETH|MEMPOOL|PKT|LCORE|rte)_.*")
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .clang_arg("-finline-functions")
        .clang_arg("-march=broadwell")
        .rustfmt_bindings(true)
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&generated_path)
        .expect("Couldn't write bindings!");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy(&generated_path, out_dir.join(GENERATED_FILE)).unwrap();

    rte_build::bind(&out_dir);
}

fn main() {
    bind();

    // re-run build.rs upon changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
}
