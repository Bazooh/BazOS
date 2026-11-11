use crate::syscall_errors::read::ReadError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExecError {
    CouldNotReadFile,
    EmptyFile,
    NotExecutable,
}

impl From<ReadError> for ExecError {
    fn from(_: ReadError) -> ExecError {
        ExecError::CouldNotReadFile
    }
}

impl From<ExecError> for i64 {
    fn from(error: ExecError) -> Self {
        match error {
            ExecError::CouldNotReadFile => -1,
            ExecError::EmptyFile => -2,
            ExecError::NotExecutable => -3,
        }
    }
}

impl TryFrom<i64> for ExecError {
    type Error = ();
    fn try_from(num: i64) -> Result<ExecError, Self::Error> {
        match &num {
            -1 => Ok(ExecError::CouldNotReadFile),
            -2 => Ok(ExecError::EmptyFile),
            -3 => Ok(ExecError::NotExecutable),
            _ => Err(()),
        }
    }
}
