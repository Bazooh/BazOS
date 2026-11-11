syscall_error! {
    pub enum DuplicateFdError {
        TooManyResourcesOpen = -1,
        UnknownFileDescriptorIndex = -2,
    }
}
