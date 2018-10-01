use super::callable::CallableBoxed;
use super::context::Context;
use super::encoding::Serialize;
use super::error::Result;
use duktape_sys as duk;
use std::collections::HashMap;

pub struct ClassBuilder {
    constructor: Option<CallableBoxed>,
    methods: HashMap<String, CallableBoxed>,
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
    // duk_push_current_function(ctx);
    // duk_get_prop_string(ctx, -1, KEY.as_ptr() as *const i8);
    // let mut c = Context::with(ctx);

    // let ptr = duk_get_pointer(ctx, -1) as *mut Box<dyn Callable>;
    // let pp = Box::from_raw(ptr);
    // duk_pop_2(ctx);
    // let ret = match pp.call(&mut c) {
    //     Err(e) => {
    //         duk_error_raw(
    //             ctx,
    //             DUK_ERR_ERROR as i32,
    //             "".as_ptr() as *const i8,
    //             0,
    //             CString::new(format!("{}", e.0)).unwrap().as_ptr(),
    //         );
    //         -1
    //     }
    //     Ok(ret) => ret,
    // };

    // // It should not be dropped
    // Box::into_raw(pp);

    return 0;
}

impl Serialize for ClassBuilder {
    fn to_context(self, ctx: &Context) -> Result<()> {
        if let Some(ctor) = self.constructor {
            ctx.push(ctor);
        } else {
            unsafe { duk::duk_push_c_function(ctx.inner, Some(class_ctor), 0) };
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
