use std::{marker::PhantomData, os::raw::c_uint};

use libc::c_void;
use rte_error::ReturnValue as _;

use crate::{lcore, Result};

enum TimerType {
    Single,
    Periodical,
}

impl TimerType {
    #[inline]
    fn raw(&self) -> c_uint {
        match self {
            TimerType::Single => ffi::rte_timer_type::SINGLE,
            TimerType::Periodical => ffi::rte_timer_type::PERIODICAL,
        }
    }
}

type RawTimer = ffi::rte_timer;
type RawTimerPtr = *mut ffi::rte_timer;

type TimerCallbackWrapper = unsafe extern "C" fn(RawTimerPtr, *mut c_void);

type TimerCallback<T> = fn(&T);
struct TimerCbContext<'a, T> {
    callback: TimerCallback<T>,
    arg: &'a T,
}

unsafe extern "C" fn timer_stub<T>(_timer: RawTimerPtr, arg: *mut c_void) {
    let ctxt = Box::leak(Box::from_raw(arg as *mut TimerCbContext<T>));
    (ctxt.callback)(ctxt.arg)
}

type TimerCallbackMut<S> = fn(&mut S);
struct TimerCbContextMut<'a, S> {
    callback: TimerCallbackMut<S>,
    arg_mut: &'a mut S,
}

unsafe extern "C" fn timer_stub_mut<T>(_timer: RawTimerPtr, arg: *mut c_void) {
    let ctxt = Box::leak(Box::from_raw(arg as *mut TimerCbContextMut<T>));
    (ctxt.callback)(ctxt.arg_mut)
}

pub struct Timer<T> {
    raw: Box<RawTimer>,
    stub: TimerCallbackWrapper,
    ctxt_ptr: *mut c_void,
    p: PhantomData<T>, // for type
}

impl<T> Timer<T> {
    #[inline]
    pub fn new<'a>(callback: TimerCallback<T>, arg: &'a T) -> Self {
        let ctxt = Box::into_raw(Box::new(TimerCbContext::<'a, T> { callback, arg })) as *mut c_void;
        let mut timer =
            Timer { raw: Box::new(RawTimer::default()), ctxt_ptr: ctxt, stub: timer_stub::<T>, p: PhantomData };
        timer.init();
        timer
    }

    #[inline]
    pub fn new_mut<'a>(callback: TimerCallbackMut<T>, arg_mut: &'a mut T) -> Self {
        let ctxt = Box::into_raw(Box::new(TimerCbContextMut::<'a, T> { callback, arg_mut })) as *mut c_void;
        let mut timer =
            Timer { raw: Box::new(RawTimer::default()), ctxt_ptr: ctxt, stub: timer_stub_mut::<T>, p: PhantomData };
        timer.init();
        timer
    }

    #[inline]
    fn as_raw(&mut self) -> RawTimerPtr {
        &mut *self.raw
    }

    #[inline]
    fn init(&mut self) {
        unsafe { ffi::rte_timer_init(self.as_raw()) };
    }

    #[inline]
    fn inner_reset(&mut self, ticks: u64, timer_type: TimerType) -> Result<()> {
        unsafe {
            ffi::rte_timer_reset(
                self.as_raw(),
                ticks,
                timer_type.raw(),
                *lcore::current(),
                Some(self.stub),
                self.ctxt_ptr,
            )
        }
        .rte_ok()?;
        Ok(())
    }

    #[inline]
    pub fn reset(&mut self, ticks: u64) -> Result<()> {
        self.inner_reset(ticks, TimerType::Single)
    }

    #[inline]
    pub fn reset_periodical(&mut self, ticks: u64) -> Result<()> {
        self.inner_reset(ticks, TimerType::Periodical)
    }

    #[inline]
    fn stop(&mut self) {
        assert_eq!(unsafe { ffi::rte_timer_stop(self.as_raw()) }, 0)
    }
}

impl<T> Drop for Timer<T> {
    fn drop(&mut self) {
        self.stop();
        // taking ownership of the context to let it drop and free the internal allocation
        // it won't be used in the callback anymore since we stopped the timer
        unsafe {
            let _ctxt = Box::from_raw(self.ctxt_ptr);
        }
    }
}

#[inline]
pub fn periodical_mut_timer<T>(ticks: u64, callback: TimerCallbackMut<T>, arg_mut: &mut T) -> Result<Timer<T>> {
    let mut timer = Timer::new_mut(callback, arg_mut);
    timer.reset_periodical(ticks).map(|_| timer)
}

pub fn manage() -> Result<()> {
    unsafe { ffi::rte_timer_manage() }.rte_ok()?;
    Ok(())
}

pub fn subsystem_init() -> Result<()> {
    unsafe { ffi::rte_timer_subsystem_init() }.rte_ok()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::test_utils::rte_test;

    struct DummyArg {
        x: u64,
        y: u64,
        calls_count: RefCell<usize>,
    }

    fn dummy_callback(arg: &DummyArg) {
        assert_eq!(arg.x, 5);
        assert_eq!(arg.y, 10);
        *arg.calls_count.borrow_mut() += 1;
    }

    fn dummy_mut_callback(arg_mut: &mut DummyArg) {
        assert_eq!(arg_mut.x, 5);
        assert_eq!(arg_mut.y, 10);
        *arg_mut.calls_count.borrow_mut() += 1;
        arg_mut.x = 7;
    }

    #[rte_test(mock_lcore)]
    fn single_reset() {
        let arg = DummyArg { x: 5, y: 10, calls_count: RefCell::new(0) };
        let mut timer = Timer::new(dummy_callback, &arg);
        timer.reset(1).unwrap();
        manage().unwrap();
        assert_eq!(*arg.calls_count.borrow(), 1);
    }

    #[rte_test(mock_lcore)]
    fn single_reset_mut() {
        let mut arg = DummyArg { x: 5, y: 10, calls_count: RefCell::new(0) };
        let mut timer = Timer::new_mut(dummy_mut_callback, &mut arg);
        timer.reset(1).unwrap();
        manage().unwrap();
        assert_eq!(*arg.calls_count.borrow(), 1);
        assert_eq!(arg.x, 7);
    }

    #[rte_test(mock_lcore)]
    fn periodical() {
        let arg = DummyArg { x: 5, y: 10, calls_count: RefCell::new(0) };
        let mut timer = Timer::new(dummy_callback, &arg);
        timer.reset_periodical(1).unwrap();
        manage().unwrap();
        assert_eq!(*arg.calls_count.borrow(), 1);
        manage().unwrap();
        assert_eq!(*arg.calls_count.borrow(), 2);
    }

    #[rte_test(mock_lcore)]
    fn periodical_mut() {
        let mut arg = DummyArg { x: 5, y: 10, calls_count: RefCell::new(0) };
        let _timer = periodical_mut_timer(1, dummy_mut_callback, &mut arg);
        manage().unwrap();
        assert_eq!(*arg.calls_count.borrow(), 1);
        assert_eq!(arg.x, 7);
        arg.x = 5; // so next callback's assert work
        manage().unwrap();
        assert_eq!(*arg.calls_count.borrow(), 2);
        assert_eq!(arg.x, 7);
    }
}
