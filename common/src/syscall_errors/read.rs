#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ReadError {
    ResourceNotOpened,
    InvalidBufferSize,
    InvalidBufferAddress,
}

impl From<ReadError> for i64 {
    fn from(error: ReadError) -> Self {
        match error {
            ReadError::ResourceNotOpened => -1,
            ReadError::InvalidBufferSize => -2,
            ReadError::InvalidBufferAddress => -3,
        }
    }
}

impl TryFrom<i64> for ReadError {
    type Error = ();
    fn try_from(num: i64) -> Result<ReadError, Self::Error> {
        match &num {
            -1 => Ok(ReadError::ResourceNotOpened),
            -2 => Ok(ReadError::InvalidBufferSize),
            -3 => Ok(ReadError::InvalidBufferAddress),
            _ => Err(()),
        }
    }
}
