use std::{error, ffi::CStr, fmt, os::raw::c_int, ptr::NonNull};

/// Error returned from call to RTE library function.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error(pub i32);

impl error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = unsafe { CStr::from_ptr(ffi::rte_strerror(self.0)) };

        f.debug_struct("RteError").field("code", &self.0).field("description", &description.to_string_lossy()).finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Error number value, stored per-thread, which can be queried after
/// calls to certain functions to determine why those functions failed.
pub fn rte_error() -> Error {
    Error(unsafe { ffi::_rte_errno() })
}

/// Trait for checking the return value from a call through FFI to the RTE library.
pub trait ReturnValue {
    type Ok;
    fn rte_ok(self) -> Result<Self::Ok, Error>;
}

/// Returns `Ok` if the pointer is non-null, otherwise uses [`rte_error`]
/// to return the error.
impl<T> ReturnValue for *mut T {
    type Ok = NonNull<T>;

    fn rte_ok(self) -> Result<Self::Ok, Error> {
        NonNull::new(self).ok_or_else(rte_error)
    }
}

/// Returns `Ok` if the value is zero or positive.
impl ReturnValue for c_int {
    type Ok = Self;

    fn rte_ok(self) -> Result<Self::Ok, Error> {
        if !self.is_negative() {
            Ok(self)
        } else {
            Err(Error(-self))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;

    #[test]
    fn check_int_result() {
        let ret = 1.rte_ok().unwrap();
        assert_eq!(ret, 1);

        let ret = (-1).rte_ok().unwrap_err();
        assert_eq!(ret, Error(1));
    }

    #[test]
    fn check_ptr_result() {
        let mut alloc = Box::new(());
        let ret = (&mut *alloc as *mut ()).rte_ok().unwrap();
        assert_eq!(ret.as_ptr() as *const _, &*alloc as *const _);

        let ret = ptr::null_mut::<()>().rte_ok();
        assert!(ret.is_err());
    }
}
