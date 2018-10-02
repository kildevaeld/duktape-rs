use super::callable::CallableBoxed;
use super::context::Context;
use super::encoding::Serialize;
use super::error::Result;
use duktape_sys as duk;
use std::collections::HashMap;
use typemap::{Key, TypeMap};

static DATA_KEY: &'static [u8] = b"\xFFdata_ptr";
static CTOR_KEY: &'static [u8] = b"\xFFctor_ptr";

pub type MethodBoxed = Box<dyn Method>;

pub struct Instance {}

/// A Callable is callable from js
pub trait Method {
    /// Specify how many arguments the function accepts
    fn argc(&self) -> i32 {
        0
    }

    /// Call the fn with the context which the callable was registered
    fn call(&self, ctx: &Context, instance: &mut Instance) -> Result<i32>;
}

pub type ConstructorBoxed = Box<dyn Constructor>;

pub trait Constructor {
    fn argc(&self) -> i32 {
        0
    }

    fn call(&self, ctx: &Context, map: &mut TypeMap) -> Result<i32>;
}

pub struct ClassBuilder {
    constructor: Option<ConstructorBoxed>,
    methods: HashMap<String, MethodBoxed>,
}

impl ClassBuilder {
    pub fn new() -> ClassBuilder {
        ClassBuilder {
            constructor: None,
            methods: HashMap::new(),
        }
    }

    pub fn method(&mut self, name: &str, callable: CallableBoxed) -> &mut Self {
        //self.methods[name.to_string()] = callable;
        self.methods.insert(name.to_string(), callable);
        self
    }

    pub fn ctor(&mut self, callable: CallableBoxed) -> &mut Self {
        self.constructor = Some(callable);
        self
    }
}

unsafe extern "C" fn class_ctor(ctx: *mut duk::duk_context) -> duk::duk_ret_t {
    duk::duk_push_current_function(ctx);

    let mut map = Box::new(TypeMap::new());

    if duk::duk_has_prop_string(ctx, DATA_KEY.as_ptr() as *const i8) == 1 {}

    unsafe {
        duk::duk_push_pointer(ctx.inner, Box::into_raw(map));
    }

    ctx.put_prop_string(-2, DATA_KEY);

    return 0;
}

impl Serialize for ClassBuilder {
    fn to_context(self, ctx: &Context) -> Result<()> {
        unsafe { duk::duk_push_c_function(ctx.inner, Some(class_ctor), 0) };

        if let Some(ctor) = self.constructor {
            let b = Box::new(ctor);
            ctx.push(Box::into_raw(b));
            ctx.put_prop_string(-2, CTOR_KEY);
        }

        ctx.push_object();

        for (name, method) in self.methods {
            ctx.push(method);
            ctx.put_prop_string(-2, name);
        }

        ctx.put_prop_string(-2, "prototype");

        Ok(())
    }
}
