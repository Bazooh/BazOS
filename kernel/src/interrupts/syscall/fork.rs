use crate::r#async::scheduler::Scheduler;

pub fn fork_handler() -> isize {
    let current_thread = unsafe {
        Scheduler::current_thread()
            .as_mut()
            .expect("Current thread not found")
    };

    current_thread.fork();
    0
}
