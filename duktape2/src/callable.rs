use super::context::{CallRet, Context, DUK_VARARGS};
use super::error::DukResult;
use super::to_context::ToDuktape;
use duktape_sys::*;
use std::ffi::{c_void, CString};

static KEY: &'static [u8] = b"\xFFptr";

/// A Callable is callable from js
pub trait Callable {
    /// Specify how many arguments the function accepts
    fn argc(&self) -> i32 {
        DUK_VARARGS
    }

    /// Call the fn with the context which the callable was registered
    fn call(&self, ctx: &Context) -> DukResult<CallRet>;
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
                CString::new(format!("{}", e)).unwrap().as_ptr(),
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

pub(crate) unsafe fn push_callable(context: &Context, callable: Box<dyn Callable>) {
    duk_push_c_function(context.inner, Some(call), callable.argc());
    let m = Box::new(callable);
    duk_push_pointer(context.inner, Box::into_raw(m) as *mut c_void);
    duk_put_prop_string(context.inner, -2, KEY.as_ptr() as *const i8);
    duk_push_c_function(context.inner, Some(dtor), 1);
    duk_set_finalizer(context.inner, -2);
}

impl<T: Fn(&Context) -> DukResult<CallRet>> Callable for (i32, T) {
    fn argc(&self) -> i32 {
        self.0
    }

    fn call(&self, ctx: &Context) -> DukResult<CallRet> {
        self.1(ctx)
    }
}

impl<T: Fn(&Context) -> DukResult<CallRet>> Callable for T {
    fn argc(&self) -> i32 {
        0
    }

    fn call(&self, ctx: &Context) -> DukResult<CallRet> {
        self(ctx)
    }
}

impl Callable for Box<dyn Callable> {
    fn argc(&self) -> i32 {
        self.as_ref().argc()
    }

    fn call(&self, ctx: &Context) -> DukResult<CallRet> {
        self.as_ref().call(ctx)
    }
}

// impl<T: Callable> ToDuktape for T {}

// impl<T: 'static + Fn(&Context) -> DukResult<CallRet>> ToDuktape for T {
//     fn to_context(self, ctx: &Context) -> DukResult<()> {
//         let boxed: Box<dyn Callable> = Box::new(self);
//         unsafe { push_callable(ctx, boxed) };
//         Ok(())
//     }
// }

// impl<T: 'static + Fn(&Context) -> DukResult<CallRet>> ToDuktape for (i32, T) {
//     fn to_context(self, ctx: &Context) -> DukResult<()> {
//         let boxed: Box<dyn Callable> = Box::new(self);
//         unsafe { push_callable(ctx, boxed) };
//         Ok(())
//     }
// }
