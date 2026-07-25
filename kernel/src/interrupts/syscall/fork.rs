use crate::r#async::scheduling::scheduler::Scheduler;
use crate::r#async::scheduling::worker::Worker;
use crate::interrupts::idt::ExceptionStackFrame;
use x86_64::VirtAddr;

pub fn fork_handler(frame: &ExceptionStackFrame) -> i64 {
    let thread_id = Worker::current().current_thread_id().unwrap();

    let current_process = Scheduler::lock()
        .get_process(thread_id.pid)
        .expect("Could not get process from pid");

    let new_process = current_process
        .lock()
        .fork(thread_id, VirtAddr::from_ptr(frame));

    Scheduler::lock().add_process(new_process);

    0
}
