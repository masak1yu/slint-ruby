use magnus::Error;

pub fn to_runtime_error(e: impl std::fmt::Display) -> Error {
    Error::new(magnus::exception::runtime_error(), e.to_string())
}

pub fn to_arg_error(e: impl std::fmt::Display) -> Error {
    Error::new(magnus::exception::arg_error(), e.to_string())
}

pub fn platform_error(e: slint_interpreter::PlatformError) -> Error {
    to_runtime_error(e)
}

pub fn event_loop_error(e: slint_interpreter::EventLoopError) -> Error {
    to_runtime_error(e)
}
