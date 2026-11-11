use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use common::fs::FileDescriptorIndex;
use common::syscall_errors::duplicate_fd::DuplicateFdError;

pub fn duplicate_fd_handler(fd_index: FileDescriptorIndex) -> i64 {
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    let mut current_process = current_process.lock();

    let Some(duplicated) = current_process.clone_resource(&fd_index) else {
        return DuplicateFdError::UnknownFileDescriptorIndex.into();
    };

    match current_process.add_resource(duplicated) {
        None => DuplicateFdError::TooManyResourcesOpen.into(),
        Some(fd_index) => fd_index.as_u64() as i64,
    }
}
