use super::context::Context;
use super::encoding::Serialize;
use super::error::Result;
use duktape_sys::*;
use std::ffi::c_void;
use std::mem;

static KEY: &'static [u8] = b"\xFFptr";

pub trait Callable {
    fn argc(&self) -> i32 {
        0
    }
    fn call(&self, ctx: &mut Context) -> Result<i32>;
}

unsafe extern "C" fn call(ctx: *mut duk_context) -> duk_ret_t {
    duk_push_current_function(ctx);
    duk_get_prop_string(ctx, -1, KEY.as_ptr() as *const i8);
    let mut c = Context::with(ctx);

    let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Callable>;
    let pp = Box::from_raw(ptr);
    duk_pop_2(ctx);
    let ret = match pp.call(&mut c) {
        Err(_) => 0,
        Ok(ret) => ret,
    };

    // It just not be dropped
    Box::into_raw(pp);

    return ret;
}

unsafe extern "C" fn dtor(ctx: *mut duk_context) -> duk_ret_t {
    duk_get_prop_string(ctx, -1, KEY.as_ptr() as *const i8);
    let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Callable>;
    duk_pop(ctx);
    duk_del_prop_string(ctx, -1, KEY.as_ptr() as *const i8);
    let pp = Box::from_raw(ptr);
    drop(pp);
    return 0;
}

impl Serialize for Box<dyn Callable> {
    fn push(self, context: &mut Context) -> Result<()> {
        unsafe {
            duk_push_c_function(context.inner, Some(call), self.argc());
            let m = Box::new(self);
            duk_push_pointer(context.inner, Box::into_raw(m) as *mut c_void);
            duk_put_prop_string(context.inner, -2, KEY.as_ptr() as *const i8);
            duk_push_c_function(context.inner, Some(dtor), 1);
            duk_set_finalizer(context.inner, -2);
        }
        Ok(())
    }
}

struct Wrapped {
    cb: Box<dyn Fn(&mut Context) -> Result<i32>>,
    a: i32,
}

impl Callable for Wrapped {
    fn argc(&self) -> i32 {
        self.a
    }
    fn call(&self, ctx: &mut Context) -> Result<i32> {
        (self.cb)(ctx)
    }
}

pub fn cb(argc: i32, cb: Box<dyn Fn(&mut Context) -> Result<i32>>) -> Box<dyn Callable> {
    Box::new(Wrapped { cb: cb, a: argc })
}
