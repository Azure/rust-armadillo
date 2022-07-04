//! Launch tasks on other lcores
//!
use std::os::raw::{c_int, c_void};

use rte_error::ReturnValue as _;

use crate::{lcore, Result};

/// State of an lcore.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Wait = ffi::rte_lcore_state_t::WAIT,
    Running = ffi::rte_lcore_state_t::RUNNING,
    Finished = ffi::rte_lcore_state_t::FINISHED,
}

impl From<ffi::rte_lcore_state_t::Type> for State {
    fn from(s: ffi::rte_lcore_state_t::Type) -> Self {
        match s {
            ffi::rte_lcore_state_t::WAIT => Ok(State::Wait),
            ffi::rte_lcore_state_t::RUNNING => Ok(State::Running),
            ffi::rte_lcore_state_t::FINISHED => Ok(State::Finished),
            _ => Err(()),
        }
        .unwrap()
    }
}

// Definition of a remote launch function.
pub type LcoreFunc<T> = fn(T) -> i32;

struct LcoreContext<T> {
    callback: LcoreFunc<T>,
    arg: T,
}

unsafe extern "C" fn lcore_stub<T: Send + 'static>(arg: *mut c_void) -> c_int {
    let LcoreContext { callback, arg } = *Box::from_raw(arg as *mut LcoreContext<T>);

    callback(arg)
}

/// Launch a function on another lcore.
///
/// See docs for [`thread::spawn`](std::thread::spawn) for an explanation of the constraints on `T`.
pub fn remote_launch_with_arg<T: Send + 'static>(callback: LcoreFunc<T>, arg: T, worker_id: lcore::Id) -> Result<()> {
    let ctxt = Box::into_raw(Box::new(LcoreContext { callback, arg })) as *mut c_void;

    unsafe { ffi::rte_eal_remote_launch(Some(lcore_stub::<T>), ctxt, *worker_id) }.rte_ok()?;
    Ok(())
}

/// Launch a function on all lcores.
///
/// See docs for [`thread::spawn`](std::thread::spawn) for an explanation of the constraints on `T`.
pub fn mp_remote_launch_with_arg<T: Send + 'static>(callback: LcoreFunc<T>, arg: T, skip_master: bool) -> Result<()> {
    let ctxt = Box::into_raw(Box::new(LcoreContext { callback, arg })) as *mut c_void;
    let call_main = if skip_master { ffi::rte_rmt_call_main_t::SKIP_MAIN } else { ffi::rte_rmt_call_main_t::CALL_MAIN };

    unsafe { ffi::rte_eal_mp_remote_launch(Some(lcore_stub::<T>), ctxt, call_main) }.rte_ok()?;
    Ok(())
}

impl lcore::Id {
    /// Get the state of the lcore identified by lcore_id.
    pub fn state(self) -> State {
        unsafe { ffi::rte_eal_get_lcore_state(*self) }.into()
    }
}

/// Wait until all lcores finish their jobs.
///
/// To be executed on the MASTER lcore only.
/// Issue an rte_eal_wait_lcore() for every lcore.
/// The return values are ignored.
pub fn mp_wait_lcore() {
    unsafe { ffi::rte_eal_mp_wait_lcore() }
}
