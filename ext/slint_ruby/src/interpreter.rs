use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use magnus::value::ReprValue;
use magnus::{function, method, Error, IntoValue, Module, Object, RArray, RModule, Ruby, Value};
use send_wrapper::SendWrapper;
use slint_interpreter::ComponentHandle;

use crate::errors;
use crate::value;

/// Store a Ruby Value as a raw usize for use in 'static closures.
/// Safety: caller must ensure the Value is GC-rooted via register_mark_object.
fn value_to_raw(val: Value) -> usize {
    unsafe { std::mem::transmute(val) }
}

/// Reconstruct a Ruby Value from a raw usize.
/// Safety: the raw value must be a valid, GC-rooted Ruby VALUE.
unsafe fn value_from_raw(raw: usize) -> Value {
    std::mem::transmute(raw)
}

// All slint-interpreter types are !Send because they use Rc internally.
// Ruby's GVL guarantees single-threaded access, so wrapping in SendWrapper is safe.

// ---------------------------------------------------------------------------
// Compiler
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::Compiler")]
struct Compiler {
    inner: SendWrapper<RefCell<slint_interpreter::Compiler>>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            inner: SendWrapper::new(RefCell::new(slint_interpreter::Compiler::default())),
        }
    }

    fn build_from_path(&self, path: String) -> Result<CompilationResult, Error> {
        let result = {
            let compiler = self.inner.borrow();
            spin_on::spin_on(compiler.build_from_path(&PathBuf::from(&path)))
        };
        Ok(CompilationResult {
            inner: SendWrapper::new(result),
        })
    }

    fn build_from_source(&self, source_code: String, path: String) -> Result<CompilationResult, Error> {
        let result = {
            let compiler = self.inner.borrow();
            spin_on::spin_on(compiler.build_from_source(source_code, PathBuf::from(&path)))
        };
        Ok(CompilationResult {
            inner: SendWrapper::new(result),
        })
    }

    fn get_include_paths(&self) -> Vec<String> {
        self.inner
            .borrow()
            .include_paths()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    fn set_include_paths(&self, paths: Vec<String>) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .set_include_paths(paths.into_iter().map(PathBuf::from).collect());
        Ok(())
    }

    fn get_style(&self) -> Option<String> {
        self.inner.borrow().style().cloned()
    }

    fn set_style(&self, style: String) -> Result<(), Error> {
        self.inner.borrow_mut().set_style(style);
        Ok(())
    }

    fn set_library_paths(&self, libraries: HashMap<String, String>) -> Result<(), Error> {
        self.inner.borrow_mut().set_library_paths(
            libraries
                .into_iter()
                .map(|(k, v)| (k, PathBuf::from(v)))
                .collect(),
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Diagnostic
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::Diagnostic")]
struct Diagnostic {
    inner: SendWrapper<slint_interpreter::Diagnostic>,
}

impl Diagnostic {
    fn level(&self) -> String {
        match self.inner.level() {
            slint_interpreter::DiagnosticLevel::Error => "error".to_string(),
            slint_interpreter::DiagnosticLevel::Warning => "warning".to_string(),
            slint_interpreter::DiagnosticLevel::Note => "note".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn message(&self) -> String {
        self.inner.message().to_string()
    }

    fn line_number(&self) -> usize {
        self.inner.line_column().0
    }

    fn column_number(&self) -> usize {
        self.inner.line_column().1
    }

    fn source_file(&self) -> Option<String> {
        self.inner
            .source_file()
            .map(|p| p.to_string_lossy().to_string())
    }

    fn to_s(&self) -> String {
        self.inner.to_string()
    }
}

// ---------------------------------------------------------------------------
// CompilationResult
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::CompilationResult")]
struct CompilationResult {
    inner: SendWrapper<slint_interpreter::CompilationResult>,
}

impl CompilationResult {
    fn component_names(&self) -> Vec<String> {
        self.inner
            .component_names()
            .map(ToString::to_string)
            .collect()
    }

    fn component(&self, name: String) -> Option<ComponentDefinition> {
        self.inner.component(&name).map(|definition| ComponentDefinition {
            inner: SendWrapper::new(definition),
        })
    }

    fn diagnostics(&self) -> Result<RArray, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let array = ruby.ary_new();
        for d in self.inner.diagnostics() {
            array.push(Diagnostic {
                inner: SendWrapper::new(d.clone()),
            })?;
        }
        Ok(array)
    }

    fn has_errors(&self) -> bool {
        self.inner.has_errors()
    }
}

// ---------------------------------------------------------------------------
// ComponentDefinition
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::ComponentDefinition")]
struct ComponentDefinition {
    inner: SendWrapper<slint_interpreter::ComponentDefinition>,
}

impl ComponentDefinition {
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    fn create(&self) -> Result<ComponentInstance, Error> {
        let instance = self.inner.create().map_err(errors::platform_error)?;
        Ok(ComponentInstance {
            inner: SendWrapper::new(instance),
        })
    }

    fn properties(&self) -> Vec<(String, String)> {
        self.inner
            .properties()
            .map(|(name, value_type)| {
                let type_name = format!("{:?}", value_type);
                (name, type_name)
            })
            .collect()
    }

    fn callbacks(&self) -> Vec<String> {
        self.inner.callbacks().collect()
    }

    fn functions(&self) -> Vec<String> {
        self.inner.functions().collect()
    }

    fn globals(&self) -> Vec<String> {
        self.inner.globals().collect()
    }

    fn global_properties(&self, global_name: String) -> Option<Vec<(String, String)>> {
        self.inner.global_properties(&global_name).map(|props| {
            props
                .map(|(name, value_type)| {
                    let type_name = format!("{:?}", value_type);
                    (name, type_name)
                })
                .collect()
        })
    }

    fn global_callbacks(&self, global_name: String) -> Option<Vec<String>> {
        self.inner
            .global_callbacks(&global_name)
            .map(|cbs| cbs.collect())
    }

    fn global_functions(&self, global_name: String) -> Option<Vec<String>> {
        self.inner
            .global_functions(&global_name)
            .map(|fns| fns.collect())
    }
}

// ---------------------------------------------------------------------------
// ComponentInstance
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::ComponentInstance")]
struct ComponentInstance {
    inner: SendWrapper<slint_interpreter::ComponentInstance>,
}

impl ComponentInstance {
    fn show(&self) -> Result<(), Error> {
        self.inner.show().map_err(errors::platform_error)
    }

    fn hide(&self) -> Result<(), Error> {
        self.inner.hide().map_err(errors::platform_error)
    }

    fn run(&self) -> Result<(), Error> {
        self.inner.show().map_err(errors::platform_error)?;
        slint_interpreter::run_event_loop().map_err(errors::platform_error)?;
        self.inner.hide().map_err(errors::platform_error)
    }

    fn get_property(&self, name: String) -> Result<Value, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let val = self
            .inner
            .get_property(&name)
            .map_err(errors::to_runtime_error)?;
        Ok(value::slint_to_ruby(&ruby, &val))
    }

    fn set_property(&self, name: String, val: Value) -> Result<(), Error> {
        let slint_val = value::ruby_to_slint(val)?;
        self.inner
            .set_property(&name, slint_val)
            .map_err(errors::to_runtime_error)
    }

    fn get_global_property(&self, global_name: String, prop_name: String) -> Result<Value, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let val = self
            .inner
            .get_global_property(&global_name, &prop_name)
            .map_err(errors::to_runtime_error)?;
        Ok(value::slint_to_ruby(&ruby, &val))
    }

    fn set_global_property(
        &self,
        global_name: String,
        prop_name: String,
        val: Value,
    ) -> Result<(), Error> {
        let slint_val = value::ruby_to_slint(val)?;
        self.inner
            .set_global_property(&global_name, &prop_name, slint_val)
            .map_err(errors::to_runtime_error)
    }

    fn set_callback(&self, name: String, callable: Value) -> Result<(), Error> {
        // GC-root the callable so it won't be collected while the closure lives
        unsafe { magnus::gc::register_mark_object(callable) };
        let raw = value_to_raw(callable);

        self.inner
            .set_callback(&name, move |args| {
                let ruby = unsafe { Ruby::get_unchecked() };
                let callable = unsafe { value_from_raw(raw) };

                let ruby_args: Vec<Value> = args
                    .iter()
                    .map(|a| value::slint_to_ruby(&ruby, a))
                    .collect();

                match callable.funcall::<_, _, Value>("call", ruby_args.as_slice()) {
                    Ok(result) => value::ruby_to_slint(result).unwrap_or(slint_interpreter::Value::Void),
                    Err(e) => {
                        eprintln!("slint-ruby: callback error: {}", e);
                        slint_interpreter::Value::Void
                    }
                }
            })
            .map_err(errors::to_runtime_error)
    }

    fn set_global_callback(
        &self,
        global_name: String,
        callback_name: String,
        callable: Value,
    ) -> Result<(), Error> {
        unsafe { magnus::gc::register_mark_object(callable) };
        let raw = value_to_raw(callable);

        self.inner
            .set_global_callback(&global_name, &callback_name, move |args| {
                let ruby = unsafe { Ruby::get_unchecked() };
                let callable = unsafe { value_from_raw(raw) };

                let ruby_args: Vec<Value> = args
                    .iter()
                    .map(|a| value::slint_to_ruby(&ruby, a))
                    .collect();

                match callable.funcall::<_, _, Value>("call", ruby_args.as_slice()) {
                    Ok(result) => value::ruby_to_slint(result).unwrap_or(slint_interpreter::Value::Void),
                    Err(e) => {
                        eprintln!("slint-ruby: global callback error: {}", e);
                        slint_interpreter::Value::Void
                    }
                }
            })
            .map_err(errors::to_runtime_error)
    }

    fn invoke(&self, name: String, args: RArray) -> Result<Value, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let mut slint_args = Vec::new();
        for i in 0..args.len() {
            let val: Value = args.entry(i as isize)?;
            slint_args.push(value::ruby_to_slint(val)?);
        }
        let result = self
            .inner
            .invoke(&name, &slint_args)
            .map_err(errors::to_runtime_error)?;
        Ok(value::slint_to_ruby(&ruby, &result))
    }

    fn invoke_global(
        &self,
        global_name: String,
        callable_name: String,
        args: RArray,
    ) -> Result<Value, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let mut slint_args = Vec::new();
        for i in 0..args.len() {
            let val: Value = args.entry(i as isize)?;
            slint_args.push(value::ruby_to_slint(val)?);
        }
        let result = self
            .inner
            .invoke_global(&global_name, &callable_name, &slint_args)
            .map_err(errors::to_runtime_error)?;
        Ok(value::slint_to_ruby(&ruby, &result))
    }
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

pub fn define(ruby: &Ruby, module: &RModule) -> Result<(), Error> {
    let object_class = ruby.class_object();

    // Compiler
    let compiler_class = module.define_class("Compiler", object_class)?;
    compiler_class.define_singleton_method("new", function!(Compiler::new, 0))?;
    compiler_class.define_method("build_from_path", method!(Compiler::build_from_path, 1))?;
    compiler_class.define_method("build_from_source", method!(Compiler::build_from_source, 2))?;
    compiler_class.define_method("include_paths", method!(Compiler::get_include_paths, 0))?;
    compiler_class.define_method("include_paths=", method!(Compiler::set_include_paths, 1))?;
    compiler_class.define_method("style", method!(Compiler::get_style, 0))?;
    compiler_class.define_method("style=", method!(Compiler::set_style, 1))?;
    compiler_class.define_method("library_paths=", method!(Compiler::set_library_paths, 1))?;

    // Diagnostic
    let diag_class = module.define_class("Diagnostic", object_class)?;
    diag_class.define_method("level", method!(Diagnostic::level, 0))?;
    diag_class.define_method("message", method!(Diagnostic::message, 0))?;
    diag_class.define_method("line_number", method!(Diagnostic::line_number, 0))?;
    diag_class.define_method("column_number", method!(Diagnostic::column_number, 0))?;
    diag_class.define_method("source_file", method!(Diagnostic::source_file, 0))?;
    diag_class.define_method("to_s", method!(Diagnostic::to_s, 0))?;

    // CompilationResult
    let result_class = module.define_class("CompilationResult", object_class)?;
    result_class.define_method("component_names", method!(CompilationResult::component_names, 0))?;
    result_class.define_method("component", method!(CompilationResult::component, 1))?;
    result_class.define_method("diagnostics", method!(CompilationResult::diagnostics, 0))?;
    result_class.define_method("has_errors?", method!(CompilationResult::has_errors, 0))?;

    // ComponentDefinition
    let def_class = module.define_class("ComponentDefinition", object_class)?;
    def_class.define_method("name", method!(ComponentDefinition::name, 0))?;
    def_class.define_method("create", method!(ComponentDefinition::create, 0))?;
    def_class.define_method("properties", method!(ComponentDefinition::properties, 0))?;
    def_class.define_method("callbacks", method!(ComponentDefinition::callbacks, 0))?;
    def_class.define_method("functions", method!(ComponentDefinition::functions, 0))?;
    def_class.define_method("globals", method!(ComponentDefinition::globals, 0))?;
    def_class.define_method("global_properties", method!(ComponentDefinition::global_properties, 1))?;
    def_class.define_method("global_callbacks", method!(ComponentDefinition::global_callbacks, 1))?;
    def_class.define_method("global_functions", method!(ComponentDefinition::global_functions, 1))?;

    // ComponentInstance
    let inst_class = module.define_class("ComponentInstance", object_class)?;
    inst_class.define_method("show", method!(ComponentInstance::show, 0))?;
    inst_class.define_method("hide", method!(ComponentInstance::hide, 0))?;
    inst_class.define_method("run", method!(ComponentInstance::run, 0))?;
    inst_class.define_method("get_property", method!(ComponentInstance::get_property, 1))?;
    inst_class.define_method("set_property", method!(ComponentInstance::set_property, 2))?;
    inst_class.define_method("get_global_property", method!(ComponentInstance::get_global_property, 2))?;
    inst_class.define_method("set_global_property", method!(ComponentInstance::set_global_property, 3))?;
    inst_class.define_method("set_callback", method!(ComponentInstance::set_callback, 2))?;
    inst_class.define_method("set_global_callback", method!(ComponentInstance::set_global_callback, 3))?;
    inst_class.define_method("invoke", method!(ComponentInstance::invoke, 2))?;
    inst_class.define_method("invoke_global", method!(ComponentInstance::invoke_global, 3))?;

    Ok(())
}
