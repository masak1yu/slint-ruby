use magnus::value::ReprValue;
use magnus::{function, method, Error, Module, Object, RModule, Ruby, Value};
use send_wrapper::SendWrapper;

use crate::errors;

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::Color")]
pub struct Color {
    pub inner: SendWrapper<slint_interpreter::Color>,
}

impl From<slint_interpreter::Color> for Color {
    fn from(c: slint_interpreter::Color) -> Self {
        Self { inner: SendWrapper::new(c) }
    }
}

impl Color {
    fn new_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            inner: SendWrapper::new(slint_interpreter::Color::from_argb_u8(alpha, red, green, blue)),
        }
    }

    fn new_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::new_rgba(red, green, blue, 255)
    }

    fn red(&self) -> u8 {
        self.inner.red()
    }

    fn green(&self) -> u8 {
        self.inner.green()
    }

    fn blue(&self) -> u8 {
        self.inner.blue()
    }

    fn alpha(&self) -> u8 {
        self.inner.alpha()
    }

    fn brighter(&self, factor: f32) -> Self {
        Self {
            inner: SendWrapper::new(self.inner.brighter(factor)),
        }
    }

    fn darker(&self, factor: f32) -> Self {
        Self {
            inner: SendWrapper::new(self.inner.darker(factor)),
        }
    }

    fn to_s(&self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            self.inner.red(),
            self.inner.green(),
            self.inner.blue(),
            self.inner.alpha()
        )
    }
}

// ---------------------------------------------------------------------------
// Brush
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::Brush")]
pub struct Brush {
    pub inner: SendWrapper<slint_interpreter::Brush>,
}

impl From<slint_interpreter::Brush> for Brush {
    fn from(b: slint_interpreter::Brush) -> Self {
        Self { inner: SendWrapper::new(b) }
    }
}

impl Brush {
    fn from_color(color: &Color) -> Self {
        Self {
            inner: SendWrapper::new(slint_interpreter::Brush::SolidColor(*color.inner)),
        }
    }

    fn color(&self) -> Color {
        Color {
            inner: SendWrapper::new(self.inner.color()),
        }
    }

    fn brighter(&self, factor: f32) -> Self {
        Self {
            inner: SendWrapper::new(self.inner.brighter(factor)),
        }
    }

    fn darker(&self, factor: f32) -> Self {
        Self {
            inner: SendWrapper::new(self.inner.darker(factor)),
        }
    }

    fn to_s(&self) -> String {
        let c = self.inner.color();
        format!(
            "Brush(rgba({}, {}, {}, {}))",
            c.red(),
            c.green(),
            c.blue(),
            c.alpha()
        )
    }
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

pub fn define(ruby: &Ruby, module: &RModule) -> Result<(), Error> {
    let object_class = ruby.class_object();

    // Color
    let color_class = module.define_class("Color", object_class)?;
    color_class.define_singleton_method("new", function!(Color::new_rgb, 3))?;
    color_class.define_singleton_method("rgba", function!(Color::new_rgba, 4))?;
    color_class.define_method("red", method!(Color::red, 0))?;
    color_class.define_method("green", method!(Color::green, 0))?;
    color_class.define_method("blue", method!(Color::blue, 0))?;
    color_class.define_method("alpha", method!(Color::alpha, 0))?;
    color_class.define_method("brighter", method!(Color::brighter, 1))?;
    color_class.define_method("darker", method!(Color::darker, 1))?;
    color_class.define_method("to_s", method!(Color::to_s, 0))?;

    // Brush
    let brush_class = module.define_class("Brush", object_class)?;
    brush_class.define_singleton_method("from_color", function!(Brush::from_color, 1))?;
    brush_class.define_method("color", method!(Brush::color, 0))?;
    brush_class.define_method("brighter", method!(Brush::brighter, 1))?;
    brush_class.define_method("darker", method!(Brush::darker, 1))?;
    brush_class.define_method("to_s", method!(Brush::to_s, 0))?;

    Ok(())
}
