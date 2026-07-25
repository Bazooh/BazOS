use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use x86_64::VirtAddr;

pub fn mmap_handler(n_pages: u64) -> i64 {
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    let mut current_process = current_process.lock();
    let available_range = current_process.find_free_mem_region(n_pages);
    if unsafe { current_process.user_mmap(available_range) }.is_err() {
        return 0;
    }
    i64::from_ne_bytes(available_range.start.start_address().as_u64().to_ne_bytes())
}

pub fn munmap_handler(addr: VirtAddr, n_pages: u64) -> i64 {
    todo!()
}
