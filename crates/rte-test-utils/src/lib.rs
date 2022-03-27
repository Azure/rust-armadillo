use std::thread;

pub fn init_test_eal() {
    let _ = rte_eal::init(["", "--no-huge", "-m", "1024", "--no-shconf"]).expect("Could not initialize EAL for tests");
}

// call after init to prevent timer collisions
pub fn mock_lcore() {
    fn parse(s: &str) -> Option<u32> {
        s.strip_prefix("ThreadId(")?.strip_suffix(')')?.parse().ok()
    }

    let thread_id_str = format!("{:?}", thread::current().id());
    let thread_id: u32 = parse(&thread_id_str).unwrap();

    set_mock_lcore(thread_id)
}

fn set_mock_lcore(lcore_id: u32) {
    unsafe { ffi::_rte_set_mock_lcore(lcore_id) };
}
