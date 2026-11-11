syscall_error! {
    pub enum ReadError {
        ResourceNotOpened = -1,
        InvalidBufferSize = -2,
        InvalidBufferAddress = -3,
        OperationNotSupported = -4,
    }
}
