use magnus::value::ReprValue;
use magnus::{Error, IntoValue, Module, Object, RHash, Ruby, TryConvert, Value};

/// Convert a slint_interpreter::Value to a Ruby Value.
/// Phase 2: supports Void, Number, String, Bool only.
/// Other types fall back to nil with a warning.
pub fn slint_to_ruby(ruby: &Ruby, val: &slint_interpreter::Value) -> Value {
    use slint_interpreter::Value as SV;
    match val {
        SV::Void => ruby.qnil().as_value(),
        SV::Number(n) => n.into_value_with(ruby),
        SV::String(s) => s.as_str().into_value_with(ruby),
        SV::Bool(b) => b.into_value_with(ruby),
        SV::Brush(brush) => {
            crate::brush::Brush::from(brush.clone()).into_value_with(ruby)
        }
        SV::Image(img) => {
            crate::image::Image::from(img.clone()).into_value_with(ruby)
        }
        SV::Struct(s) => {
            let hash = ruby.hash_new();
            for (key, val) in s.iter() {
                let _ = hash.aset(
                    key.into_value_with(ruby),
                    slint_to_ruby(ruby, val),
                );
            }
            hash.as_value()
        }
        _ => {
            eprintln!(
                "slint-ruby: conversion from Slint to Ruby not yet implemented for {:?}",
                val
            );
            ruby.qnil().as_value()
        }
    }
}

/// Convert a Ruby Value to a slint_interpreter::Value.
/// Phase 2: supports nil, Float/Integer, String, Bool.
pub fn ruby_to_slint(val: Value) -> Result<slint_interpreter::Value, Error> {
    let ruby = unsafe { Ruby::get_unchecked() };
    use slint_interpreter::Value as SV;

    if val.is_nil() {
        return Ok(SV::Void);
    }

    // Check for actual true/false singletons (not truthy/falsy coercion).
    // Compare raw VALUE pointers since magnus::Value doesn't implement PartialEq.
    let raw: usize = unsafe { std::mem::transmute(val) };
    let qtrue: usize = unsafe { std::mem::transmute(ruby.qtrue().as_value()) };
    let qfalse: usize = unsafe { std::mem::transmute(ruby.qfalse().as_value()) };
    if raw == qtrue {
        return Ok(SV::Bool(true));
    }
    if raw == qfalse {
        return Ok(SV::Bool(false));
    }

    // Try f64 (covers both Integer and Float in Ruby)
    if let Ok(n) = f64::try_convert(val) {
        return Ok(SV::Number(n));
    }

    if let Ok(s) = String::try_convert(val) {
        return Ok(SV::String(s.into()));
    }

    // Check for wrapped Slint types
    if let Ok(model) = <&crate::models::ListModel>::try_convert(val) {
        return Ok(SV::Model(model.as_model()));
    }

    if let Ok(brush) = <&crate::brush::Brush>::try_convert(val) {
        return Ok(SV::Brush((*brush.inner).clone()));
    }

    if let Ok(color) = <&crate::brush::Color>::try_convert(val) {
        return Ok(SV::Brush(slint_interpreter::Brush::SolidColor(*color.inner)));
    }

    if let Ok(image) = <&crate::image::Image>::try_convert(val) {
        return Ok(SV::Image((*image.inner).clone()));
    }

    // Ruby Hash → Slint Struct
    if let Ok(hash) = <RHash>::try_convert(val) {
        let mut fields: Vec<(String, slint_interpreter::Value)> = Vec::new();
        hash.foreach(|key: String, value: Value| {
            fields.push((key, ruby_to_slint(value)?));
            Ok(magnus::r_hash::ForEach::Continue)
        })?;
        return Ok(SV::Struct(slint_interpreter::Struct::from_iter(fields)));
    }

    Err(Error::new(
        magnus::exception::type_error(),
        format!("Cannot convert Ruby value to Slint value: {:?}", val),
    ))
}
