use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use common::fs::FileDescriptor;

pub fn write_handler(file_descriptor: FileDescriptor, buffer: Result<&[u8], ()>) -> i64 {
    let buffer = match buffer {
        Ok(buffer) => buffer,
        Err(_) => return -4,
    };
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    match current_process.lock().get_resource_mut(&file_descriptor) {
        None => -1,
        Some(resource) => {
            if buffer.len() > i64::MAX as usize {
                return -3;
            }

            match resource.write(buffer) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
    }
}
