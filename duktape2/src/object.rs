use super::argument_list::ArgumentList;
use super::array::Array;
use super::context::{Constructable, Context, Idx, Type};
use super::error::DukResult;
use super::from_context::*;
use super::function::Function;
use super::property::{Property, PropertyBuilder};
use super::reference::{JSValue, Reference};
use super::to_context::*;
use std::fmt;

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
        let r = Reference::new(self.ctx(), -1);
        self.ctx().pop(1);
        return Property {
            _ref: r,
            prop: prop,
        };
    }

    // fn call<T: AsRef<str>, A: ArgumentList, R: FromDuktape<'a>>(
    //     &self,
    //     fn_name: T,
    //     args: A,
    // ) -> DukResult<R> {
    //     self.push();
    //     let idx = self.ctx().normalize_index(-1);
    //     self.ctx().push_string(fn_name.as_ref());
    //     let len = args.len();
    //     args.push_args(self.ctx())?;
    //     if let Err(e) = self.ctx().call_prop(idx, len) {
    //         self.ctx().pop(1);
    //         return Err(e);
    //     }
    //     self.ctx().remove(-2);
    //     let ret = R::from_context(self.ctx(), -1);
    //     self.ctx().pop(1);
    //     ret
    // }

    // /// Construct a property on the object
    // fn construct<T: AsRef<str>, A: ArgumentList>(
    //     &self,
    //     fn_name: T,
    //     args: A,
    // ) -> DukResult<Object<'a>> {
    //     self.push();
    //     if !self.ctx().has_prop_string(-1, fn_name.as_ref()) {
    //         duk_type_error!("not a function");
    //     }

    //     let len = args.len();

    //     self.ctx().get_prop_string(-1, fn_name.as_ref());
    //     args.push_args(self.ctx())?;
    //     if let Err(e) = self.ctx().construct(len) {
    //         self.ctx().pop(2);
    //         return Err(e);
    //     }

    //     self.ctx().remove(-2);
    //     let o = Object::from_context(self.ctx(), -1);
    //     self.ctx().pop(1);
    //     o
    // }

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

    fn define_property(&self, definition: PropertyBuilder) -> DukResult<()> {
        self.push();
        duk_ok_or_pop!(definition.build(self.ctx(), -1), self.ctx(), 1);
        self.ctx().pop(1);
        Ok(())
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
    fn to_context(&self, _ctx: &Context) -> DukResult<()> {
        self.push();
        Ok(())
    }
}

// impl<'a> ToDuktape for &'a Object<'a> {
//     fn to_context(self, _ctx: &Context) -> DukResult<()> {
//         self.push();
//         Ok(())
//     }
// }

impl<'a> FromDuktape<'a> for Object<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> DukResult<Self> {
        let re = Reference::new(ctx, index);
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
        duk_type_error!("could not interpret object as array");
    }
}
