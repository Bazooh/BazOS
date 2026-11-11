use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::fs::driver::DiskDriver;
use crate::io::disk::driver::DISK_DRIVER;
use crate::program::executor::ProgramExecutor;
use common::path::Path;

pub fn exec_handler(file_path: Result<&str, ()>, args: Result<&[&str], ()>) -> i64 {
    let file_path = match file_path {
        Ok(path) => match Path::new(path) {
            Ok(path) => path,
            Err(()) => return -3,
        },
        Err(()) => return -1,
    };
    let args = match args {
        Ok(args) => args,
        Err(_) => return -2,
    };

    let file = DISK_DRIVER.try_get().unwrap().open(file_path).unwrap();

    let current_pid = Worker::current().current_thread_id().unwrap().pid;

    let process = match ProgramExecutor::execute(file, args, current_pid) {
        Ok(process) => process,
        Err(_) => return -4,
    };
    let pid = process.pid();
    Scheduler::lock().add_process(process);

    pid as i64
}
