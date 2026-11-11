use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::fs::driver::DiskDriver;
use crate::io::disk::driver::DISK_DRIVER;
use crate::io::key_events::KeyEvents;
use alloc::sync::Arc;
use common::io_error::IOError;
use common::keyboard::KEY_EVENTS_PATH;
use common::opened_resource::Resource;
use common::path::Path;
use common::syscall_errors::open::OpenError;

pub fn open_handler(path: Result<&str, ()>) -> i64 {
    let Ok(path) = path else {
        return OpenError::InvalidMemoryAccess.into();
    };
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    let resource = match open_device(path) {
        Some(device) => device,
        None => {
            let Ok(path) = Path::new(path) else {
                return OpenError::InvalidArgument.into();
            };
            match DISK_DRIVER
                .try_get()
                .expect("Driver not initialized")
                .open(path)
            {
                Ok(resource) => Arc::new(resource),
                Err(IOError::DoesNotExist(..)) => return OpenError::DoesNotExist.into(),
                Err(_) => unreachable!(),
            }
        }
    };

    match current_process.lock().add_resource(resource) {
        None => OpenError::TooManyResourcesOpen.into(),
        Some(file_descriptor) => file_descriptor.as_u64() as i64,
    }
}

fn open_device(path: &str) -> Option<Arc<dyn Resource>> {
    match path {
        KEY_EVENTS_PATH => Some(Arc::new(KeyEvents::open())),
        _ => None,
    }
}
