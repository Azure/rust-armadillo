//! Generated FFI bindings for DPDK.
//!
//! # Statically linking a Rust binary to DPDK
//!
//! DPDK uses a bespoke plugin system that uses the
//! `__attribute__((constructor))`[^a] [GCC function attribute](https://gcc.gnu.org/onlinedocs/gcc-4.7.0/gcc/Function-Attributes.html)
//! for run-time registration of plugins. This attribute causes a function to be
//! placed in an ELF section that the Linux library loader calls when the
//! library is loaded (if it statically linked that means program start-up).
//!
//! Unless told otherwise, both `ld` and `lld`, the GCC and LLVM linkers
//! respectively, will eliminate any code that is not used from the resultant
//! binary, which in DPDK's case means that any plugins that are not statically
//! called from any client code paths not be present in the final binary, and
//! those plugins will not get registered.
//!
//! To make sure that the plugins are kept in the resultant binary,
//! normally[^b] one would use the `--whole-archive` flag to mark their
//! libraries to be included verbatim in the linked binary, unconditionally:
//!
//! ```bash
//! $ pkg-config --static --libs libdpdk
//! -L/usr/local/lib/x86_64-linux-gnu \
//! -Wl,--whole-archive \
//! -l:librte_common_cpt.a -l:librte_common_dpaax.a [...plugin libraries] \
//! -Wl,--no-whole-archive \
//! -Wl,--export-dynamic \
//! -lmlx5 -libverbs -latomic -lmlx4 [...other libraries]
//! ```
//!
//! Unfortunately, at the time of writing, the
//! [`whole-archive` modifier](https://doc.rust-lang.org/nightly/unstable-book/language-features/native-link-modifiers-whole-archive.html)
//! for libraries is [still being stabilized](https://github.com/rust-lang/rust/pull/93901).
//! For now, we use a slightly awkward hack to work around the missing feature on
//! stable, which can hopefully be removed after one of the next stable Rust
//! releases.
//!
//! ## `whole-archive` Hack
//!
//! To use `rte-sys`, every upstream binary user of this create a build script
//! that invokes the [`rte_build::bind()`](../rte_build/) function, and then
//! [`include!`] the generated file some where in its entrypoint file:
//!
//! ```toml
//! ; Cargo.toml
//! [bin]
//! name = "..."
//!
//! [build-dependencies]
//! rte-build = { path = "rte-build" }
//! ```
//!
//! ```no_run
//! // build.rs
//! use std::env;
//!
//! fn main() {
//!     rte_build::bind(env::var("OUT_DIR").unwrap());
//! }
//! ```
//!
//! ```no_run
//! // main.rs
//! include!(concat!(env!("OUT_DIR", "/whole_archive_hack.rs")));
//!
//! fn main() {
//!     // ...
//! }
//! ```
//!
//! [^a]: The mechanism is explained pretty well here:
//!     <https://stackoverflow.com/questions/2053029/how-exactly-does-attribute-constructor-work>
//!
//! [^b]: Solution explained for non-Rust use cases:
//!     <https://stackoverflow.com/questions/52293444/how-to-compile-a-dpdk-application-as-a-library>

#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg(target_os = "linux")]

pub mod dpdk_bindings {
    include!(concat!(env!("OUT_DIR"), "/dpdk_bindings.rs"));
}
