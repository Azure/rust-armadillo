//! A (hopefully) safe way to wrap command line arguments (possibly those
//! returned by [env::args](std::env::args)) and provide a FFI-compatible way of
//! passing command line parameters to C libraries that use the standard
//! signature for receiving command line arguments, i.e.:
//!
//! `int main(int argc, char* argv[]);`
//!
//! # Example
//!
//! ```
//! extern "C" fn extern_main(argc: i32, argv: *mut *mut i8) {}
//!
//! use argv::Args;
//!
//! let mut args = Args::new(std::env::args());
//! let mut ptrs = args.as_ptrs();
//! let mut argv = ptrs.as_argv();
//!
//! unsafe {
//!     let (argc, mut argv) = (argv.argc(), argv.argv());
//!     extern_main(argc, argv);
//! }
//! ```
//!
//! # Notes
//!
//! This crate was built to facilitate calling DPDK's [`rte_eal_init`](http://doc.dpdk.org/api/rte__eal_8h.html#a5c3f4dddc25e38c5a186ecd8a69260e3).
//!
//! The implementation aims to be as safe as possible, while not necessarily as
//! performant as possible (it shouldn't be called more than once in an
//! executable's lifetime).
//!
//! Raw pointers are stored alongside a regular Rust reference to "tie" the
//! pointer to the lifetime's provenance. This allows the Rust borrow checker to
//! statically prevent possible use-after-free of the pointers and general
//! misuse.
//!
//! ```
//! # use argv::Args;
//! let mut args = Args::new(std::env::args());
//! let mut ptrs = args.as_ptrs();
//! let mut argv = ptrs.as_argv();
//! ```
//!
//! ```compile_fail
//! # use argv::Args;
//! # let mut args = Args::new(std::env::args());
//! # let mut ptrs = args.as_ptrs();
//! # let mut argv = ptrs.as_argv();
//! drop(ptrs);
//! unsafe { argv.argv(); } // Can't use argv because it is tied to ptrs' lifetime
//! ```
//!
//! ```compile_fail
//! # use argv::Args;
//! # let mut args = Args::new(std::env::args());
//! # let mut ptrs = args.as_ptrs();
//! # let mut argv = ptrs.as_argv();
//! drop(args);
//! ptrs.to_argv(); // Can't use ptrs because it is tied to args' lifetime
//! ```

use std::{ffi::CString, os::raw::c_char, ptr};

/// Create a clone of command line arguments, encoded into [`CString`]s.
pub struct Args(Vec<CString>);

impl Args {
    pub fn new(args: impl IntoIterator<Item = String>) -> Self {
        Self(args.into_iter().map(CString::new).collect::<Result<_, _>>().unwrap())
    }

    pub fn as_ptrs(&mut self) -> ArgPtrs {
        ArgPtrs::new(self)
    }
}

/// A list of pointers pointing to a list of [`CString`]s contained in an
/// [`Args`] struct.
///
/// This struct will ensure that there is a NUL pointer at the end of the list,
/// which programs are allowed to use to count the number of arguments passed
/// (in lieu of the `argc` param): <https://stackoverflow.com/a/18547129/1410290>
///
/// Created by calling [`Args::as_ptrs`].
pub struct ArgPtrs<'a> {
    args: &'a mut Args,
    ptrs: Vec<*const c_char>,
}

impl<'a> ArgPtrs<'a> {
    fn new(args: &'a mut Args) -> Self {
        let mut ptrs = args.0.iter_mut().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
        ptrs.push(ptr::null_mut());

        Self { args, ptrs }
    }

    /// See: [`Argv`]
    pub fn as_argv(&mut self) -> Argv<'a, '_> {
        Argv { ptrs: self }
    }
}

/// A wrapper around a mutable borrow of an [`ArgPtrs`] that can return a raw
/// pointer to the list of pointers contained in the [`ArgPtrs`].
pub struct Argv<'a, 'p> {
    ptrs: &'p mut ArgPtrs<'a>,
}

impl<'a, 'p> Argv<'a, 'p> {
    /// # Safety
    ///
    /// This function returns a raw pointer that is not tied via lifetimes to
    /// any provenance, the caller must ensure that this pointer is not used
    /// after the lifetime of `&mut self` has ended.
    pub unsafe fn argv(&mut self) -> *mut *mut i8 {
        self.ptrs.ptrs.as_mut_ptr() as *mut _
    }

    pub fn argc(&self) -> i32 {
        self.ptrs.args.0.len() as i32
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn test_argv() {
        const ARGS: [&str; 2] = ["hello", "world"];
        let mut args = Args::new(ARGS.map(str::to_string));
        let mut ptrs = args.as_ptrs();
        let mut argv = ptrs.as_argv();

        unsafe {
            let (argc, mut argv) = (argv.argc(), argv.argv());

            assert_eq!(argc, ARGS.len() as i32);

            let mut args = vec![];
            while !ptr::read(argv).is_null() {
                args.push(CStr::from_ptr(ptr::read(argv as *const _)).to_str().unwrap().to_string());
                argv = argv.add(1);
            }

            assert_eq!(args, ARGS);
        }
    }
}
