use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::fs::driver::DiskDriver;
use crate::fs::path::Path;
use crate::io::disk::driver::DISK_DRIVER;
use crate::program::executor::ProgramExecutor;

pub fn exec_handler(file_path: Option<&str>, args: &[&str]) -> i64 {
    let Some(file_path) = file_path else {
        return -1;
    };

    let file = DISK_DRIVER
        .try_get()
        .unwrap()
        .open(Path::new(file_path))
        .unwrap();

    let current_pid = Worker::current().current_thread_id().unwrap().pid;

    let process = ProgramExecutor::execute(file, args, current_pid);
    let pid = process.pid();
    Scheduler::lock().add_process(process);

    pid as i64
}
