syscall_error! {
    pub enum OpenError {
        AccessDenied = -1,
        InvalidPath = -2,
        InvalidArgument = -3,
        InvalidMemoryAccess = -4,
        AlreadyOpen = -5,
        DoesNotExist = -6,
        TooManyResourcesOpen = -7,
    }
}
