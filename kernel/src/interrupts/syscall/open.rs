use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::fs::driver::DiskDriver;
use crate::io::disk::driver::DISK_DRIVER;
use alloc::boxed::Box;
use common::io_error::IOError;
use common::path::Path;
use common::syscall_errors::open::OpenError;

pub fn open_handler(path: Result<&str, ()>) -> i64 {
    let path = match path {
        Ok(path) => match Path::new(path) {
            Ok(path) => path,
            Err(()) => return OpenError::InvalidArgument.into(),
        },
        Err(()) => return OpenError::InvalidMemoryAccess.into(),
    };
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    let resource = match DISK_DRIVER
        .try_get()
        .expect("Driver not initialized")
        .open(path)
    {
        Ok(resource) => Box::new(resource),
        Err(IOError::DoesNotExist(..)) => return OpenError::DoesNotExist.into(),
        Err(_) => unreachable!(),
    };

    match current_process.lock().add_resource(resource) {
        None => OpenError::TooManyResourcesOpen.into(),
        Some(file_descriptor) => file_descriptor.as_u64() as i64,
    }
}
