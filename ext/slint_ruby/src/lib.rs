mod brush;
mod errors;
mod image;
mod interpreter;
mod models;
mod timer;
mod value;

use magnus::value::ReprValue;
use magnus::{function, Error, Ruby, Value};

fn run_event_loop() -> Result<(), Error> {
    slint_interpreter::run_event_loop().map_err(errors::platform_error)
}

fn quit_event_loop() -> Result<(), Error> {
    slint_interpreter::quit_event_loop().map_err(errors::event_loop_error)
}

fn invoke_from_event_loop(callable: Value) -> Result<(), Error> {
    magnus::gc::register_mark_object(callable);
    let raw: usize = unsafe { std::mem::transmute(callable) };

    slint_interpreter::invoke_from_event_loop(move || {
        let callable: Value = unsafe { std::mem::transmute(raw) };
        if let Err(e) = callable.funcall::<_, _, Value>("call", ()) {
            eprintln!("slint-ruby: invoke_from_event_loop error: {}", e);
        }
    })
    .map_err(errors::event_loop_error)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Slint")?;

    interpreter::define(ruby, &module)?;
    models::define(ruby, &module)?;
    brush::define(ruby, &module)?;
    image::define(ruby, &module)?;
    timer::define(ruby, &module)?;

    module.define_module_function("run_event_loop", function!(run_event_loop, 0))?;
    module.define_module_function("quit_event_loop", function!(quit_event_loop, 0))?;
    module.define_module_function("invoke_from_event_loop", function!(invoke_from_event_loop, 1))?;

    Ok(())
}
