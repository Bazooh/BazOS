use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use common::fs::FileDescriptorIndex;
use common::syscall_errors::read::ReadError;

pub fn read_handler(file_descriptor: FileDescriptorIndex, buffer: Result<&mut [u8], ()>) -> i64 {
    let buffer = match buffer {
        Ok(buffer) => buffer,
        Err(()) => return ReadError::InvalidBufferAddress.into(),
    };
    let Ok(buffer) = buffer.try_into() else {
        return ReadError::InvalidBufferSize.into();
    };

    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    match current_process.lock().lock_resource(&file_descriptor) {
        None => ReadError::ResourceNotOpened.into(),
        Some(mut resource) => match resource.read_into(buffer) {
            Err(error) => error.into(),
            Ok(size) => size as i64,
        },
    }
}
