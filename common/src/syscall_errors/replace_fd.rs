syscall_error! {
    pub enum ReplaceFdError {
        InvalidOldFileDescriptorIndex = -1,
        InvalidNewFileDescriptorIndex = -2,
    }
}
