use super::context::Context;
use super::encoding::Serialize;
use super::error::Result;
use duktape_sys::*;
use std::ffi::{c_void, CString};

static KEY: &'static [u8] = b"\xFFptr";

pub type CallableBoxed = Box<dyn Callable>;

/// A Callable is callable from js
pub trait Callable {
    /// Specify how many arguments the function accepts
    fn argc(&self) -> i32 {
        0
    }

    /// Call the fn with the context which the callable was registered
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
        Err(e) => {
            duk_error_raw(
                ctx,
                DUK_ERR_ERROR as i32,
                "".as_ptr() as *const i8,
                0,
                CString::new(format!("{}", e.0)).unwrap().as_ptr(),
            );
            -1
        }
        Ok(ret) => ret,
    };

    // It should not be dropped
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
    fn to_context(self, context: &Context) -> Result<()> {
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

/* Closure support */
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
