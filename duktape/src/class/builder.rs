use super::super::{error::Result, Context, Serialize};
use super::method::{Instance, Method, Wrapped, CTOR_KEY, DATA_KEY};
use duktape_sys as duk;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::Fn;

pub enum Prototype {
    Method(Box<dyn Method>),
}

#[derive(Default)]
pub struct Builder {
    ctor: Option<Box<dyn Method>>,
    methods: HashMap<String, Prototype>,
}

impl Builder {
    pub fn set(&mut self, name: &str, prop: Prototype) -> &mut Self {
        self.methods.insert(name.to_owned(), prop);
        self
    }

    pub fn method<T: 'static>(&mut self, name: &str, method: T) -> &mut Self
    where
        T: Fn(&mut Context, &mut Instance) -> Result<i32>,
    {
        let wrapped = Wrapped(method);
        let b: Box<dyn Method> = Box::new(wrapped);
        self.methods.insert(name.to_owned(), Prototype::Method(b));
        self
    }

    pub fn constructor<T: 'static>(&mut self, ctor: T) -> &mut Self
    where
        T: Fn(&mut Context, &mut Instance) -> Result<i32>,
    {
        let wrapped = Wrapped(ctor);
        let b: Box<dyn Method> = Box::new(wrapped);
        self.ctor = Some(b);
        self
    }
}

impl Serialize for Builder {
    fn to_context(self, ctx: &Context) -> Result<()> {
        debug!("pushing class");
        unsafe {
            duk::duk_push_c_function(ctx.ptr(), Some(class_ctor), 1);
        };

        ctx.push_object();

        for (name, method) in self.methods {
            match method {
                Prototype::Method(m) => {
                    ctx.push(m);
                    ctx.put_prop_string(-2, &name);
                }
            }
        }

        ctx.put_prop_string(-2, "prototype");

        if let Some(ctor) = self.ctor {
            debug!("push class constructor");
            let b = Box::new(ctor);
            unsafe { duk::duk_push_pointer(ctx.inner, Box::into_raw(b) as *mut c_void) };
            unsafe {
                duk::duk_put_prop_lstring(
                    ctx.inner,
                    -2,
                    CTOR_KEY.as_ptr() as *const i8,
                    CTOR_KEY.len(),
                )
            };
        }

        unsafe {
            duk::duk_push_c_function(ctx.inner, Some(constructor_dtor), 1);
            duk::duk_set_finalizer(ctx.inner, -2);
        }

        Ok(())
    }
}

unsafe extern "C" fn class_ctor(ctx: *mut duk::duk_context) -> duk::duk_ret_t {
    debug!("class constructor");
    duk::duk_push_current_function(ctx);

    let mut instance = Box::new(Instance::new());
    // duk::duk_dump_context_stdout(ctx);
    if duk::duk_has_prop_lstring(ctx, -1, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len()) == 1 {
        debug!("found custom class constructor");
        duk::duk_get_prop_lstring(ctx, -1, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len());
        let ptr = duk::duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
        duk::duk_pop(ctx);

        let ctor = Box::from_raw(ptr);
        let mut c = Context::with(ctx);
        match ctor.call(&mut c, &mut instance) {
            Ok(_) => {}
            Err(_) => {
                Box::into_raw(ctor);
                duk::duk_error_raw(
                    ctx,
                    duk::DUK_ERR_ERROR as i32,
                    "".as_ptr() as *const i8,
                    0,
                    "could find data ptr".as_ptr() as *const i8,
                );
                return 0;
            }
        };

        // We wanna keep the ctor on the heap
        Box::into_raw(ctor);
    }

    duk::duk_push_this(ctx);
    duk::duk_push_pointer(ctx, Box::into_raw(instance) as *mut c_void);
    duk::duk_put_prop_lstring(ctx, -2, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
    duk::duk_push_c_function(ctx, Some(class_dtor), 1);
    duk::duk_set_finalizer(ctx, -2);

    return 0;
}

unsafe extern "C" fn constructor_dtor(ctx: *mut duk::duk_context) -> duk::duk_ret_t {
    debug!("constructor dtor");

    if duk::duk_has_prop_lstring(ctx, 0, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len()) == 1 {
        debug!("dropping class constructor");
        duk::duk_get_prop_lstring(ctx, 0, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len());
        let ptr = duk::duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
        Box::from_raw(ptr);
        duk::duk_pop(ctx);
    }

    return 0;
}

unsafe extern "C" fn class_dtor(ctx: *mut duk::duk_context) -> duk::duk_ret_t {
    debug!("class dtor");
    if duk::duk_has_prop_lstring(ctx, 0, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len()) == 1 {
        debug!("dropping instance data");
        duk::duk_get_prop_lstring(ctx, 0, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
        let ptr = duk::duk_get_pointer(ctx, -1) as *mut Instance;
        Box::from_raw(ptr);
        duk::duk_pop(ctx);
    }
    0
}
