use std::{
    ffi::CString,
    io::{self, BufRead, BufReader},
    mem,
    os::unix::{net::UnixStream, prelude::AsRawFd},
    thread,
};

use rte_error::ReturnValue as _;
use tracing::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Rte(#[from] rte_error::Error),
}

/// Set up unix stream for RTE logs (instead of stderr), spawn a thread for reading logs
/// and writing them through global logging mechanism.
fn init_log_reader() -> Result<(), Error> {
    let (tx, rx) = UnixStream::pair()?;

    unsafe {
        let mode = CString::new("w").unwrap();
        let fd = libc::fdopen(tx.as_raw_fd(), mode.as_ptr());
        ffi::rte_openlog_stream(fd as *mut _).rte_ok()?;
    }

    // prevent the fd rte uses for logging from being dropped, which would
    // cause it to be closed
    mem::forget(tx);

    thread::spawn(|| {
        let mut logs = BufReader::new(rx).lines();
        while let Some(Ok(log)) = logs.next() {
            info!(target: "ddosd::rte", "{log}");
        }
    });

    Ok(())
}

/// Initializes EAL by calling [`rte_eal_init`](https://doc.dpdk.org/api/rte__eal_8h.html#a5c3f4dddc25e38c5a186ecd8a69260e3),
/// passing in the provided command line arguments, and returning an
/// [`Iterator<Item = String>`](Iterator) of the arguments, skipping the ones
/// "digested" by EAL.
pub fn init<A, S>(args: A) -> Result<impl Iterator<Item = String>, Error>
where
    A: IntoIterator<Item = S>,
    S: Into<String>,
{
    init_log_reader()?;

    let args = args.into_iter().map(S::into).collect::<Vec<_>>();

    let args_read = {
        let mut args = argv::Args::new(args.clone());
        let mut arg_ptrs = args.as_ptrs();
        let mut argv = arg_ptrs.as_argv();

        unsafe { ffi::rte_eal_init(argv.argc(), argv.argv()) }.rte_ok()?
    };

    Ok(args.into_iter().skip(args_read as usize))
}
