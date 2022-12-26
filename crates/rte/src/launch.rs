//! Based on DPDK's `rte_launch.h` API: <https://doc.dpdk.org/api-21.08/rte__launch_8h.html>

use std::{
    os::raw::{c_int, c_void},
    panic::{catch_unwind, AssertUnwindSafe},
    process,
};

use rte_error::ReturnValue as _;

use crate::{lcore, Result};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Wait = ffi::rte_lcore_state_t::WAIT,
    Running = ffi::rte_lcore_state_t::RUNNING,
}

impl From<ffi::rte_lcore_state_t::Type> for State {
    fn from(s: ffi::rte_lcore_state_t::Type) -> Self {
        match s {
            ffi::rte_lcore_state_t::WAIT => Ok(State::Wait),
            ffi::rte_lcore_state_t::RUNNING => Ok(State::Running),
            _ => Err(()),
        }
        .unwrap()
    }
}

type Entrypoint<T> = fn(T) -> i32;

struct ExecutionContext<T> {
    entrypoint: Entrypoint<T>,
    arg: T,
}

unsafe extern "C" fn lcore_stub<T: Send + 'static>(arg: *mut c_void) -> c_int {
    let ExecutionContext { entrypoint, arg } = *Box::from_raw(arg as *mut ExecutionContext<T>);

    // any panics that occurred inside `entrypoint` should NOT be unwound back into EAL,
    // this is unsafe and causes the rust panic mechanism to fail with a SIGABRT
    let res = catch_unwind(AssertUnwindSafe(|| entrypoint(arg)));

    match res {
        Ok(status) => status,
        Err(_) => {
            // unlike regular OS threads which don't crash the entire process on panics,
            // we'd like to bring down the whole process if one of the lcore workers has crashed.
            // at this point, the panic's backtrace has already been written to stderr by the
            // global rust panic hook
            process::abort()
        }
    }
}

impl lcore::Id {
    /// **NOTE:** should be executed on main lcore only. Will `panic` otherwise, if debug assertions are enabled.
    ///
    /// See docs for [`thread::spawn`](std::thread::spawn) for an explanation of the constraints on `T`.
    #[inline]
    pub fn launch<T: Send + 'static>(self, entrypoint: Entrypoint<T>, arg: T) -> Result<()> {
        debug_assert!(lcore::current().is_main());
        // Safety: memory is released in `lcore_stub` (success) or in the `Err` match arm (failure)
        let ctxt = Box::into_raw(Box::new(ExecutionContext { entrypoint, arg })) as *mut c_void;
        match unsafe { ffi::rte_eal_remote_launch(Some(lcore_stub::<T>), ctxt, self.get()) }.rte_ok() {
            Ok(_) => Ok(()),
            Err(err) => {
                let _ = unsafe { Box::from_raw(ctxt) };
                Err(err)
            }
        }
    }

    /// **NOTE:** should be executed on main lcore only. Will `panic` otherwise, if debug assertions are enabled.
    #[inline]
    pub fn state(self) -> State {
        debug_assert!(lcore::current().is_main());
        unsafe { ffi::rte_eal_get_lcore_state(self.get()) }.into()
    }
}

/// **NOTE:** should be executed on main lcore only. Will `panic` otherwise, if debug assertions are enabled.
#[inline]
pub fn join_lcores() {
    debug_assert!(lcore::current().is_main());
    unsafe { ffi::rte_eal_mp_wait_lcore() }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use rte_test_macros::rte_test;
    use util_macros::millis;

    use super::*;

    fn work(sleep_ms: u64) -> i32 {
        thread::sleep(millis!(sleep_ms));
        0
    }

    #[ignore = "There's no guarantee that the UT will run in the main thread.
    This means `debug_assert!`s verifying that functions run in the main thread might fail, which indeed happens occasionally in CI.
    Can be fixed by changing the way RTE EAL is used in UT.
    Tracked by: <https://msazure.visualstudio.com/One/_workitems/edit/15312324>"]
    #[rte_test]
    fn test_sanity() {
        let workers = lcore::Id::iter_enabled(true).take(3).collect::<Vec<_>>();
        for worker_id in &workers {
            assert!(worker_id.launch(work, 300).is_ok());
        }
        join_lcores();
        assert!(workers.iter().all(|worker| worker.state() == State::Wait));
    }
}
