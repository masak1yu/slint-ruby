use magnus::{function, method, Error, Module, Object, RArray, RModule, Ruby, Value};
use send_wrapper::SendWrapper;

use i_slint_core::model::{Model, ModelRc, VecModel};

use crate::errors;
use crate::value;

// ---------------------------------------------------------------------------
// ListModel — wraps VecModel<slint_interpreter::Value>
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "Slint::ListModel")]
pub struct ListModel {
    inner: SendWrapper<std::rc::Rc<VecModel<slint_interpreter::Value>>>,
}

impl ListModel {
    fn new(initial: RArray) -> Result<Self, Error> {
        let mut items = Vec::new();
        for i in 0..initial.len() {
            let val: Value = initial.entry(i as isize)?;
            items.push(value::ruby_to_slint(val)?);
        }
        let vec_model = std::rc::Rc::new(VecModel::from(items));
        Ok(Self {
            inner: SendWrapper::new(vec_model),
        })
    }

    fn row_count(&self) -> usize {
        self.inner.row_count()
    }

    fn row_data(&self, row: usize) -> Result<Value, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        match self.inner.row_data(row) {
            Some(val) => Ok(value::slint_to_ruby(&ruby, &val)),
            None => Err(errors::to_range_error(
                format!("Row index {} out of range (count: {})", row, self.inner.row_count()),
            )),
        }
    }

    fn set_row_data(&self, row: usize, val: Value) -> Result<(), Error> {
        let slint_val = value::ruby_to_slint(val)?;
        self.inner.set_row_data(row, slint_val);
        Ok(())
    }

    fn push(&self, val: Value) -> Result<(), Error> {
        let slint_val = value::ruby_to_slint(val)?;
        self.inner.push(slint_val);
        Ok(())
    }

    fn remove(&self, index: usize) -> Result<(), Error> {
        if index >= self.inner.row_count() {
            return Err(errors::to_range_error(
                format!("Row index {} out of range (count: {})", index, self.inner.row_count()),
            ));
        }
        self.inner.remove(index);
        Ok(())
    }

    fn insert(&self, index: usize, val: Value) -> Result<(), Error> {
        let slint_val = value::ruby_to_slint(val)?;
        self.inner.insert(index, slint_val);
        Ok(())
    }

    fn clear(&self) {
        // Remove all items by clearing from end to start
        while self.inner.row_count() > 0 {
            self.inner.remove(self.inner.row_count() - 1);
        }
    }

    fn to_a(&self) -> Result<RArray, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let array = ruby.ary_new();
        for i in 0..self.inner.row_count() {
            if let Some(val) = self.inner.row_data(i) {
                array.push(value::slint_to_ruby(&ruby, &val))?;
            }
        }
        Ok(array)
    }

    fn length(&self) -> usize {
        self.inner.row_count()
    }
}

impl ListModel {
    pub fn as_model(&self) -> ModelRc<slint_interpreter::Value> {
        let rc: std::rc::Rc<VecModel<slint_interpreter::Value>> = (*self.inner).clone();
        ModelRc::from(rc)
    }
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

pub fn define(ruby: &Ruby, module: &RModule) -> Result<(), Error> {
    let object_class = ruby.class_object();

    let list_model_class = module.define_class("ListModel", object_class)?;
    list_model_class.define_singleton_method("new", function!(ListModel::new, 1))?;
    list_model_class.define_method("row_count", method!(ListModel::row_count, 0))?;
    list_model_class.define_method("row_data", method!(ListModel::row_data, 1))?;
    list_model_class.define_method("set_row_data", method!(ListModel::set_row_data, 2))?;
    list_model_class.define_method("push", method!(ListModel::push, 1))?;
    list_model_class.define_method("<<", method!(ListModel::push, 1))?;
    list_model_class.define_method("remove", method!(ListModel::remove, 1))?;
    list_model_class.define_method("insert", method!(ListModel::insert, 2))?;
    list_model_class.define_method("clear", method!(ListModel::clear, 0))?;
    list_model_class.define_method("to_a", method!(ListModel::to_a, 0))?;
    list_model_class.define_method("length", method!(ListModel::length, 0))?;
    list_model_class.define_method("size", method!(ListModel::length, 0))?;

    Ok(())
}
