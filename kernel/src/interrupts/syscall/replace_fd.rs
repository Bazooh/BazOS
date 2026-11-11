use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use common::fs::FileDescriptorIndex;
use common::syscall_errors::replace_fd::ReplaceFdError;

pub fn replace_fd_handler(
    old_fd_index: FileDescriptorIndex,
    new_fd_index: FileDescriptorIndex,
) -> i64 {
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    let mut current_process = current_process.lock();

    let Some(duplicated) = current_process.clone_resource(&new_fd_index) else {
        return ReplaceFdError::InvalidNewFileDescriptorIndex.into();
    };

    match current_process.replace_resource(old_fd_index, duplicated) {
        None => ReplaceFdError::InvalidOldFileDescriptorIndex.into(),
        Some(()) => 0,
    }
}
