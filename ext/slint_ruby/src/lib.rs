mod brush;
mod errors;
mod image;
mod interpreter;
mod models;
mod timer;
mod value;

use magnus::{function, Error, Ruby};

fn run_event_loop() -> Result<(), Error> {
    slint_interpreter::run_event_loop().map_err(errors::platform_error)
}

fn quit_event_loop() -> Result<(), Error> {
    slint_interpreter::quit_event_loop().map_err(errors::event_loop_error)
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

    Ok(())
}
