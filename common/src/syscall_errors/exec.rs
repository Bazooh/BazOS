use crate::syscall_errors::read::ReadError;

syscall_error! {
    pub enum ExecError {
        CouldNotReadFile = -1,
        EmptyFile = -2,
        NotExecutable = -3,
    }
}

impl From<ReadError> for ExecError {
    fn from(_: ReadError) -> ExecError {
        ExecError::CouldNotReadFile
    }
}
