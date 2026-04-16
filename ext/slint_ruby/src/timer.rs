use magnus::value::ReprValue;
use magnus::{function, method, Error, Module, Object, RModule, Ruby, Value};
use send_wrapper::SendWrapper;

// ---------------------------------------------------------------------------
// Timer
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::Timer")]
pub struct Timer {
    inner: SendWrapper<i_slint_core::timers::Timer>,
}

impl Timer {
    fn new() -> Self {
        Self {
            inner: SendWrapper::new(Default::default()),
        }
    }

    fn start_repeated(&self, interval_secs: f64, callable: Value) -> Result<(), Error> {
        let duration = std::time::Duration::from_secs_f64(interval_secs);
        magnus::gc::register_mark_object(callable);
        let raw: usize = unsafe { std::mem::transmute(callable) };

        self.inner.start(
            i_slint_core::timers::TimerMode::Repeated,
            duration,
            move || {
                let callable: Value = unsafe { std::mem::transmute(raw) };
                if let Err(e) = callable.funcall::<_, _, Value>("call", ()) {
                    eprintln!("slint-ruby: timer callback error: {}", e);
                }
            },
        );
        Ok(())
    }

    fn start_single_shot(&self, interval_secs: f64, callable: Value) -> Result<(), Error> {
        let duration = std::time::Duration::from_secs_f64(interval_secs);
        magnus::gc::register_mark_object(callable);
        let raw: usize = unsafe { std::mem::transmute(callable) };

        self.inner.start(
            i_slint_core::timers::TimerMode::SingleShot,
            duration,
            move || {
                let callable: Value = unsafe { std::mem::transmute(raw) };
                if let Err(e) = callable.funcall::<_, _, Value>("call", ()) {
                    eprintln!("slint-ruby: timer callback error: {}", e);
                }
            },
        );
        Ok(())
    }

    fn stop(&self) {
        self.inner.stop();
    }

    fn restart(&self) {
        self.inner.restart();
    }

    fn is_running(&self) -> bool {
        self.inner.running()
    }

    fn interval_secs(&self) -> f64 {
        self.inner.interval().as_secs_f64()
    }
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

pub fn define(ruby: &Ruby, module: &RModule) -> Result<(), Error> {
    let object_class = ruby.class_object();

    let timer_class = module.define_class("Timer", object_class)?;
    timer_class.define_singleton_method("new", function!(Timer::new, 0))?;
    timer_class.define_method("start_repeated", method!(Timer::start_repeated, 2))?;
    timer_class.define_method("start_single_shot", method!(Timer::start_single_shot, 2))?;
    timer_class.define_method("stop", method!(Timer::stop, 0))?;
    timer_class.define_method("restart", method!(Timer::restart, 0))?;
    timer_class.define_method("running?", method!(Timer::is_running, 0))?;
    timer_class.define_method("interval", method!(Timer::interval_secs, 0))?;

    Ok(())
}
