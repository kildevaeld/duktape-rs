use super::argument_list::ArgumentList;
use super::context::{Constructable, Context, Idx};
use super::encoding::{Deserialize, Serialize};
use super::error::{ErrorKind, Result};
use super::reference::Reference;
use duktape_sys as duk;

pub struct Object<'a> {
    refer: Reference<'a>,
}

impl<'a> Object<'a> {
    pub(crate) fn new(refer: Reference<'a>) -> Object<'a> {
        Object { refer }
    }

    pub fn get<T: AsRef<[u8]>, V: Deserialize<'a>>(&self, prop: T) -> Result<V> {
        self.refer.push();
        let ret = self.refer.ctx.get_prop_string(-1, prop).get::<V>(-1)?;
        unsafe { duk::duk_pop_n(self.refer.ctx.inner, 2) };
        Ok(ret)
    }

    pub fn set<T: AsRef<[u8]>, V: Serialize>(&mut self, prop: T, value: V) -> &mut Self {
        self.refer.push();
        self.refer.ctx.push(value).put_prop_string(-2, prop).pop(1);
        self
    }

    pub fn del<T: AsRef<[u8]>>(&mut self, prop: T) -> &mut Self {
        self.refer.push();
        self.refer.ctx.del_prop_string(-1, prop.as_ref());
        unsafe {
            duk::duk_del_prop_string(
                self.refer.ctx.inner,
                -1,
                prop.as_ref().as_ptr() as *const i8,
            )
        };
        unsafe { duk::duk_pop(self.refer.ctx.inner) };
        self
    }

    pub fn as_ref(&self) -> &'a Reference {
        &self.refer
    }

    pub fn call<T: AsRef<str>, A: ArgumentList, R: Deserialize<'a>>(
        &self,
        fn_name: T,
        args: A,
    ) -> Result<R> {
        self.refer.push();
        let idx = self.refer.ctx.normalize_index(-1);
        self.refer.ctx.push(fn_name.as_ref());
        let len = args.len();
        args.push(self.refer.ctx);
        if let Err(e) = self.refer.ctx.call_prop(idx, len) {
            self.refer.ctx.pop(1);
            return Err(e);
        }
        self.refer.ctx.remove(-2);
        self.refer.ctx.getp()
    }

    pub fn construct<T: AsRef<str>, A: ArgumentList, R: Deserialize<'a>>(
        &self,
        fn_name: T,
        args: A,
    ) -> Result<R> {
        self.refer.push();
        if !self.refer.ctx.has_prop_string(-1, fn_name.as_ref()) {
            return Err(ErrorKind::TypeError("not a function".to_owned()).into());
        }

        let len = args.len();

        self.refer.ctx.get_prop_string(-1, fn_name.as_ref());
        args.push(self.refer.ctx);
        if let Err(e) = self.refer.ctx.construct(len) {
            self.refer.ctx.pop(2);
            return Err(e);
        }

        self.refer.ctx.remove(-2).getp()

        //self.refer.push();
        //let idx = self.refer.ctx.normalize_index(-1);
        //self.refer.ctx.push(fn_name.as_ref());
        // let len = args.len();
        // args.push(self.refer.ctx);
        // if let Err(e) = self.refer.ctx.call_prop(idx, len) {
        //     self.refer.ctx.pop(1);
        //     return Err(e);
        // }
        // self.refer.ctx.remove(-2);
        // self.refer.ctx.getp()
    }
}

impl<'a> Serialize for Object<'a> {
    fn to_context(self, _ctx: &Context) -> Result<()> {
        self.refer.push();
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Object<'a> {
    fn from_context(ctx: &'a Context, index: Idx) -> Result<Self> {
        let re = Reference::new(ctx, index);
        Ok(Object::new(re))
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
