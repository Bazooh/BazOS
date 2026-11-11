#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OpenError {
    AccessDenied,
    InvalidPath,
    InvalidArgument,
    InvalidMemoryAccess,
    AlreadyOpen,
    DoesNotExist,
    TooManyResourcesOpen,
}

impl From<OpenError> for i64 {
    fn from(error: OpenError) -> Self {
        match error {
            OpenError::AccessDenied => -1,
            OpenError::InvalidPath => -2,
            OpenError::InvalidArgument => -3,
            OpenError::InvalidMemoryAccess => -4,
            OpenError::AlreadyOpen => -5,
            OpenError::DoesNotExist => -6,
            OpenError::TooManyResourcesOpen => -7,
        }
    }
}

impl TryFrom<i64> for OpenError {
    type Error = ();
    fn try_from(num: i64) -> Result<OpenError, Self::Error> {
        match &num {
            -1 => Ok(OpenError::AccessDenied),
            -2 => Ok(OpenError::InvalidPath),
            -3 => Ok(OpenError::InvalidArgument),
            -4 => Ok(OpenError::InvalidMemoryAccess),
            -5 => Ok(OpenError::AlreadyOpen),
            -6 => Ok(OpenError::DoesNotExist),
            -7 => Ok(OpenError::TooManyResourcesOpen),
            _ => Err(()),
        }
    }
}
