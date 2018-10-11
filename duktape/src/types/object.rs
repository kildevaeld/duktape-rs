use super::super::context::{Constructable, Context, Idx, Type};
use super::super::error::{ErrorKind, Result};
use super::reference::Ref;
use super::{ArgumentList, Array, FromDuktape, Function, ToDuktape};
use std::convert::From;
use std::fmt;
use std::iter;

pub struct Object<'a> {
    refer: Ref<'a>,
}

impl<'a> Object<'a> {
    pub(crate) fn new(refer: Ref<'a>) -> Object<'a> {
        Object { refer }
    }

    /// Get property
    pub fn get<T: AsRef<[u8]>, V: FromDuktape<'a>>(&self, prop: T) -> Result<V> {
        self.refer.push();
        self.refer.ctx.get_prop_string(-1, prop);
        let ret = V::from_context(self.refer.ctx, -1)?;
        self.refer.ctx.pop(2);
        Ok(ret)
    }

    /// Set property
    pub fn set<T: AsRef<[u8]>, V: ToDuktape>(&self, prop: T, value: V) -> &Self {
        self.refer.push();
        value.to_context(self.refer.ctx).unwrap();
        self.refer.ctx.put_prop_string(-2, prop);
        self.refer.ctx.pop(1);
        self
    }

    /// Check if object has property
    pub fn has<T: AsRef<[u8]>>(&self, prop: T) -> bool {
        self.refer.push();
        let ret = self.refer.ctx.has_prop_string(-1, prop);
        self.refer.ctx.pop(1);
        ret
    }

    /// Delete property
    pub fn del<T: AsRef<[u8]>>(&mut self, prop: T) -> &mut Self {
        self.refer.push();
        self.refer.ctx.del_prop_string(-1, prop.as_ref());
        self.refer.ctx.pop(1);
        self
    }

    /// Cast object to a reference
    pub fn as_ref(&self) -> &'a Ref {
        &self.refer
    }

    /// Call a method on the object
    pub fn call<T: AsRef<str>, A: ArgumentList, R: FromDuktape<'a>>(
        &self,
        fn_name: T,
        args: A,
    ) -> Result<R> {
        self.refer.push();
        let idx = self.refer.ctx.normalize_index(-1);
        self.refer.ctx.push_string(fn_name.as_ref());
        let len = args.len();
        args.push_args(self.refer.ctx)?;
        if let Err(e) = self.refer.ctx.call_prop(idx, len) {
            self.refer.ctx.pop(1);
            return Err(e);
        }
        self.refer.ctx.remove(-2);
        let ret = R::from_context(self.refer.ctx, -1);
        self.refer.ctx.pop(1);
        ret
    }

    /// Construct a property on the object
    pub fn construct<T: AsRef<str>, A: ArgumentList>(&self, fn_name: T, args: A) -> Result<Object> {
        self.refer.push();
        if !self.refer.ctx.has_prop_string(-1, fn_name.as_ref()) {
            return Err(ErrorKind::TypeError("not a function".to_owned()).into());
        }

        let len = args.len();

        self.refer.ctx.get_prop_string(-1, fn_name.as_ref());
        args.push_args(self.refer.ctx)?;
        if let Err(e) = self.refer.ctx.construct(len) {
            self.refer.ctx.pop(2);
            return Err(e);
        }

        self.refer.ctx.remove(-2);
        let o = Object::from_context(self.refer.ctx, -1);
        self.refer.ctx.pop(1);
        o
    }

    /// Return keys
    pub fn keys(&'a self) -> Array<'a> {
        let o = self
            .refer
            .ctx
            .get_global_string("Object")
            .getp::<Object>()
            .unwrap();

        o.call::<_, _, Array>("keys", self).unwrap()
    }

    /// Get a object iterator
    pub fn iter(&'a self) -> impl iter::Iterator<Item = (&'a str, Ref<'a>)> {
        ObjectIterator::new(self, self.keys())
    }
}

impl<'a> ToDuktape for Object<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> ToDuktape for &'a Object<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> FromDuktape<'a> for Object<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> Result<Self> {
        let re = Ref::new(ctx, index);
        Ok(Object::new(re))
    }
}

impl<'a> Clone for Object<'a> {
    fn clone(&self) -> Self {
        Object::new(self.refer.clone())
    }
}

impl<'a> Constructable<'a> for Object<'a> {
    fn construct(duk: &'a Context) -> Result<Self> {
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

impl<'a> From<Function<'a>> for Object<'a> {
    fn from(func: Function<'a>) -> Self {
        Object::new(func.refer.clone())
    }
}

impl<'a> From<Object<'a>> for Result<Function<'a>> {
    fn from(func: Object<'a>) -> Self {
        if func.as_ref().is(Type::Function) {
            return Ok(Function::new(func.refer.clone()));
        }
        Err(ErrorKind::TypeError("could not interpret object as function".to_owned()).into())
    }
}

impl<'a> From<Array<'a>> for Object<'a> {
    fn from(array: Array<'a>) -> Self {
        Object::new(array.refer.clone())
    }
}

impl<'a> From<Object<'a>> for Result<Array<'a>> {
    fn from(obj: Object<'a>) -> Self {
        if obj.as_ref().is(Type::Array) {
            return Ok(Array::new(obj.refer.clone()));
        }
        Err(ErrorKind::TypeError("could not interpret object as array".to_owned()).into())
    }
}

impl<'a> ArgumentList for Object<'a> {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> Result<()> {
        self.to_context(ctx)
    }
}

impl<'a> ArgumentList for &'a Object<'a> {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> Result<()> {
        self.to_context(ctx)
    }
}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.refer.ctx.get_global_string("JSON");
        let j: Object = Object::from_context(self.refer.ctx, -1).unwrap();
        self.refer.ctx.pop(1);
        let clone = self.clone();
        let json = j
            .call::<_, _, String>("stringify", (clone, (), "  "))
            .unwrap();
        write!(f, "{}", json)
    }
}

struct ObjectIterator<'a> {
    object: &'a Object<'a>,
    keys: Array<'a>,
    index: u32,
}

impl<'a> ObjectIterator<'a> {
    pub(crate) fn new(object: &'a Object<'a>, keys: Array<'a>) -> ObjectIterator<'a> {
        ObjectIterator {
            object,
            keys,
            index: 0,
        }
    }
}

impl<'a> iter::Iterator for ObjectIterator<'a> {
    type Item = (&'a str, Ref<'a>);

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

#[cfg(test)]
pub mod tests {

    use super::super::super::context::{Context, Type};
    use super::Object;

    #[test]
    fn create_object() {
        let duk = Context::new().unwrap();
        let o: Object = duk.create().unwrap();
        assert_eq!(o.as_ref().get_type(), Type::Object);
    }

    #[test]
    fn object_keys() {
        let duk = Context::new().unwrap();
        let o: Object = duk.create().unwrap();
        o.set("prop1", 1).set("prop2", 2);
        let keys = o.keys();

        assert_eq!(2, keys.len());
        assert_eq!(keys.get::<&str>(0).unwrap(), "prop1");
        assert_eq!(keys.get::<&str>(1).unwrap(), "prop2");
    }

    #[test]
    fn object_iterator() {
        let duk = Context::new().unwrap();
        let o: Object = duk.create().unwrap();
        o.set("prop1", "rap").set("prop2", 2);

        let mut iter = o.iter();

        let ret = iter.next().unwrap();
        assert_eq!(ret.0, "prop1");
        assert_eq!(ret.1.get::<&str>().unwrap(), "rap");

        let ret = iter.next().unwrap();
        assert_eq!(ret.0, "prop2");
        assert_eq!(ret.1.get::<i32>().unwrap(), 2);
    }

}
