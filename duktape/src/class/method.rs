use super::super::{error::Result, Context, Serialize};
use duktape_sys::*;
use std::ffi::{c_void, CString};
use typemap::TypeMap;

pub struct Instance {
    types: TypeMap,
}

impl Instance {
    pub fn new() -> Instance {
        Instance {
            types: TypeMap::new(),
        }
    }

    pub fn data(&self) -> &TypeMap {
        &self.types
    }

    pub fn data_mut(&mut self) -> &mut TypeMap {
        &mut self.types
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        println!("{}", "drop instance");
        drop(&mut self.types);
    }
}

static KEY: &'static [u8] = b"\xFFmethod_ptr";
pub static DATA_KEY: &'static [u8] = b"\xFFdata_ptr";
pub static CTOR_KEY: &'static [u8] = b"ctor_ptr";

pub trait Method {
    fn call(&self, ctx: &mut Context, instance: &mut Instance) -> Result<i32>;
}

pub struct Wrapped<T: Fn(&mut Context, &mut Instance) -> Result<i32>>(pub T);

impl<T: Fn(&mut Context, &mut Instance) -> Result<i32>> Method for Wrapped<T> {
    fn call(&self, ctx: &mut Context, instance: &mut Instance) -> Result<i32> {
        self.0(ctx, instance)
    }
}

impl Serialize for Box<dyn Method> {
    fn to_context(self, ctx: &Context) -> Result<()> {
        unsafe {
            duk_push_c_function(ctx.ptr(), Some(call), 1);
            let m = Box::new(self);
            duk_push_pointer(ctx.ptr(), Box::into_raw(m) as *mut c_void);
            duk_put_prop_lstring(ctx.ptr(), -2, KEY.as_ptr() as *const i8, KEY.len());
            duk_push_c_function(ctx.ptr(), Some(dtor), 1);
            duk_set_finalizer(ctx.ptr(), -2);
        }
        Ok(())
    }
}

unsafe extern "C" fn call(ctx: *mut duk_context) -> duk_ret_t {
    duk_push_current_function(ctx);

    // Get Function ptr
    duk_get_prop_lstring(ctx, -1, KEY.as_ptr() as *const i8, KEY.len());
    let mut c = Context::with(ctx);
    let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
    let method = Box::from_raw(ptr);
    duk_pop_2(ctx);

    duk_push_this(ctx);
    if duk_has_prop_lstring(ctx, -1, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len()) != 1 {
        // Keep it
        Box::into_raw(method);
        duk_error_raw(
            ctx,
            DUK_ERR_ERROR as i32,
            "".as_ptr() as *const i8,
            0,
            "could find data ptr".as_ptr() as *const i8,
        );
        return 0;
    }

    duk_get_prop_lstring(ctx, -1, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
    let ptr = duk_get_pointer(ctx, -1) as *mut Instance;
    let mut pp = Box::from_raw(ptr);

    let ret = match method.call(&mut c, &mut pp) {
        Err(e) => {
            // Keep it
            Box::into_raw(method);
            Box::into_raw(pp);
            duk_error_raw(
                ctx,
                DUK_ERR_ERROR as i32,
                "".as_ptr() as *const i8,
                0,
                CString::new(format!("{}", e.0)).unwrap().as_ptr(),
            );
            return 0;
        }
        Ok(ret) => ret,
    };

    // Keep it
    Box::into_raw(method);
    Box::into_raw(pp);

    return ret;
}

unsafe extern "C" fn dtor(ctx: *mut duk_context) -> duk_ret_t {
    debug!("method ctor");
    duk_get_prop_lstring(ctx, -1, KEY.as_ptr() as *const i8, KEY.len());
    let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
    duk_pop(ctx);
    duk_del_prop_lstring(ctx, -1, KEY.as_ptr() as *const i8, KEY.len());
    let pp = Box::from_raw(ptr);
    drop(pp);
    return 0;
}
