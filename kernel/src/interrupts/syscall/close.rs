use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use common::fs::FileDescriptorIndex;

pub fn close_handler(fd_index: FileDescriptorIndex) -> i64 {
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    match current_process.lock().remove_resource(&fd_index) {
        Some(_) => 0,
        None => -1,
    }
}
