use magnus::value::ReprValue;
use magnus::{function, method, Error, Module, Object, RModule, Ruby, Value};
use send_wrapper::SendWrapper;

use crate::errors;

// ---------------------------------------------------------------------------
// Image
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::Image")]
pub struct Image {
    pub inner: SendWrapper<slint_interpreter::Image>,
}

impl From<slint_interpreter::Image> for Image {
    fn from(img: slint_interpreter::Image) -> Self {
        Self { inner: SendWrapper::new(img) }
    }
}

impl Image {
    fn load_from_path(path: String) -> Result<Self, Error> {
        let img = slint_interpreter::Image::load_from_path(std::path::Path::new(&path))
            .map_err(errors::to_runtime_error)?;
        Ok(Self {
            inner: SendWrapper::new(img),
        })
    }

    fn width(&self) -> u32 {
        self.inner.size().width
    }

    fn height(&self) -> u32 {
        self.inner.size().height
    }

    fn path(&self) -> Option<String> {
        self.inner.path().map(|p| p.to_string_lossy().to_string())
    }

    fn to_s(&self) -> String {
        let size = self.inner.size();
        format!("Image({}x{})", size.width, size.height)
    }
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

pub fn define(ruby: &Ruby, module: &RModule) -> Result<(), Error> {
    let object_class = ruby.class_object();

    let image_class = module.define_class("Image", object_class)?;
    image_class.define_singleton_method("load_from_path", function!(Image::load_from_path, 1))?;
    image_class.define_method("width", method!(Image::width, 0))?;
    image_class.define_method("height", method!(Image::height, 0))?;
    image_class.define_method("path", method!(Image::path, 0))?;
    image_class.define_method("to_s", method!(Image::to_s, 0))?;

    Ok(())
}
