use magnus::{Error, Ruby};

pub fn to_runtime_error(e: impl std::fmt::Display) -> Error {
    let ruby = unsafe { Ruby::get_unchecked() };
    Error::new(ruby.exception_runtime_error(), e.to_string())
}

pub fn to_range_error(e: impl std::fmt::Display) -> Error {
    let ruby = unsafe { Ruby::get_unchecked() };
    Error::new(ruby.exception_range_error(), e.to_string())
}

pub fn to_type_error(e: impl std::fmt::Display) -> Error {
    let ruby = unsafe { Ruby::get_unchecked() };
    Error::new(ruby.exception_type_error(), e.to_string())
}

pub fn platform_error(e: slint_interpreter::PlatformError) -> Error {
    to_runtime_error(e)
}

pub fn event_loop_error(e: slint_interpreter::EventLoopError) -> Error {
    to_runtime_error(e)
}
