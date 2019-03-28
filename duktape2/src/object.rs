use super::argument_list::ArgumentList;
use super::array::{Array, JSArray};
use super::context::{Constructable, Context, Idx, Type};
use super::error::DukResult;
use super::from_context::*;
use super::function::Function;
use super::privates::{get_data, init_data};
use super::property::{Property, PropertyBuilder};
use super::reference::{JSValue, Reference};
use super::to_context::*;
use std::fmt;
use typemap::TypeMap;
use std::iter;

pub trait JSObject<'a>: JSValue<'a> {
    fn get<T: AsRef<[u8]>, V: FromDuktape<'a>>(&self, prop: T) -> DukResult<V> {
        self.push();
        self.ctx().get_prop_string(-1, prop);
        let ret = self.ctx().get::<V>(-1)?;
        self.ctx().pop(2);
        Ok(ret)
    }

    /// Set property
    fn set<T: AsRef<[u8]>, V: ToDuktape>(&self, prop: T, value: V) -> DukResult<&Self> {
        self.push();
        self.ctx().push(value)?.put_prop_string(-2, prop).pop(1);
        Ok(self)
    }

    /// Check if object has property
    fn has<T: AsRef<[u8]>>(&self, prop: T) -> bool {
        self.push();
        let ret = self.ctx().has_prop_string(-1, prop);
        self.ctx().pop(1);
        ret
    }

    /// Delete property
    fn del<T: AsRef<[u8]>>(&mut self, prop: T) -> &mut Self {
        self.push();
        self.ctx().del_prop_string(-1, prop.as_ref());
        self.ctx().pop(1);
        self
    }

    fn prop(&self, prop: &'a str) -> Property<'a> {
        self.push();
        let r = Reference::new(self.ctx(), -1).expect("could get property");
        self.ctx().pop(1);
        return Property {
            _ref: r,
            prop: prop,
        };
    }

    /// Return keys
    fn keys(&'a self) -> Array<'a> {
        self.ctx()
            .get_global_string("Object")
            .getp::<Object>()
            .unwrap()
            .prop("keys")
            .call::<_, Array>(self.to::<Reference>().unwrap())
            .unwrap()
    }

    fn iter(&'a self) -> ObjectIterator<'a, Self> {
        ObjectIterator::new(self, self.keys())
    }

    fn define_property(&self, definition: PropertyBuilder) -> DukResult<()> {
        self.push();
        duk_ok_or_pop!(definition.build(self.ctx(), -1), self.ctx(), 1);
        self.ctx().pop(1);
        Ok(())
    }

    fn data(&self) -> &'a TypeMap {
        unsafe {
            init_data(self.ctx().inner, -1);
            let data = get_data(self.ctx().inner, -1);
            &*data
        }
    }

    fn data_mut(&self) -> &'a mut TypeMap {
        unsafe {
            init_data(self.ctx().inner, -1);
            let data = get_data(self.ctx().inner, -1);
            &mut *data
        }
    }
}

#[derive(Clone)]
pub struct Object<'a> {
    _ref: Reference<'a>,
}

impl<'a> Object<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Object<'a> {
        Object { _ref: refer }
    }
}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx().get_global_string("JSON");
        let j: Object = Object::from_context(self.ctx(), -1).unwrap();
        self.ctx().pop(1);
        let clone = self.clone();
        let json = j
            .prop("stringify")
            .call::<_, &str>((clone, (), "  "))
            .unwrap();
        write!(f, "{}", json)
    }
}

impl<'a> JSValue<'a> for Object<'a> {
    fn push(&self) -> &Self {
        self._ref.push();
        self
    }

    fn ctx(&self) -> &'a Context {
        self._ref.ctx()
    }
}

impl<'a> JSObject<'a> for Object<'a> {}

impl<'a> ToDuktape for Object<'a> {
    fn to_context(self, _ctx: &Context) -> DukResult<()> {
        self.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Object<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> DukResult<Self> {
        let re = Reference::new(ctx, index)?;
        Ok(Object::new(re))
    }
}

impl<'a> Constructable<'a> for Object<'a> {
    fn construct(duk: &'a Context) -> DukResult<Self> {
        duk.push_object();
        let o = match Object::from_context(duk, -1) {
            Ok(o) => o,
            Err(e) => {
                duk.pop(1);
                return Err(e);
            }
        };
        duk.pop(1);
        Ok(o)
    }
}

impl<'a> ArgumentList for Object<'a> {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> DukResult<()> {
        self.to_context(ctx)
    }
}

impl<'a> From<Function<'a>> for Object<'a> {
    fn from(func: Function<'a>) -> Self {
        Object::new(func._ref.clone())
    }
}

impl<'a> From<Object<'a>> for DukResult<Function<'a>> {
    fn from(func: Object<'a>) -> Self {
        if func.is(Type::Function) {
            return Ok(Function::new(func._ref.clone()));
        }
        duk_type_error!("could not interpret object as function")
    }
}

impl<'a> From<Array<'a>> for Object<'a> {
    fn from(array: Array<'a>) -> Self {
        Object::new(array._ref.clone())
    }
}

impl<'a> From<Object<'a>> for DukResult<Array<'a>> {
    fn from(obj: Object<'a>) -> Self {
        if obj.is(Type::Array) {
            return Ok(Array::new(obj._ref.clone()));
        }
        duk_type_error!("could not interpret object as array")
    }
}


pub struct ObjectIterator<'a, O: JSObject<'a>> {
    object: &'a O,
    keys: Array<'a>,
    index: u32,
}

impl<'a, O: JSObject<'a>> ObjectIterator<'a, O> {
    pub(crate) fn new(object: &'a O, keys: Array<'a>) -> ObjectIterator<'a, O> {
        ObjectIterator {
            object,
            keys,
            index: 0,
        }
    }
}

impl<'a, O: JSObject<'a>> iter::Iterator for ObjectIterator<'a, O> {
    type Item = (&'a str, Reference<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.keys.len() as u32 {
            return None;
        }

        let key: &str = match self.keys.get(self.index) {
            Ok(m) => m,
            Err(_) => return None,
        };

        let value = match self.object.get(key) {
            Ok(m) => m,
            Err(_) => return None,
        };

        self.index += 1;

        Some((key, value))
    }
}