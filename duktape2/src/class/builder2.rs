use super::super::from_context::*;
use super::super::function::Function;
use super::super::object::Object;
use super::super::reference::Reference;
use super::super::to_context::*;
use super::super::{
    context::{Context, Idx},
    error::{DukErrorCode, DukResult},
};
use super::method::{push_method, Instance, Method, CTOR_KEY, DATA_KEY};
use duktape_sys as duk;
use std::collections::HashMap;
use std::ffi::c_void;
use typemap::Key;

macro_rules! persist {
    ($name: ident, $expr: ty) => {
        struct $name;
        impl Key for $name {
            type Value = $expr;
        }
    };
}

persist!(Ctor, Box<dyn Method>);
persist!(Dtor, Box<dyn Method>);

pub enum Prototype {
    Method(Box<dyn Method>),
}

#[derive(Default)]
pub struct ClassBuilder<'a> {
    name: String,
    ctor: Option<Box<dyn Method>>,
    dtor: Option<Box<dyn Method>>,
    parent: Option<Function<'a>>,
    props: HashMap<String, Prototype>,
}

impl<'a> ClassBuilder<'a> {
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn set(mut self, name: &str, prop: Prototype) -> Self {
        self.props.insert(name.to_owned(), prop);
        self
    }

    pub fn method<T: 'static + Method>(mut self, name: &str, method: T) -> Self {
        let b: Box<dyn Method> = Box::new(method);
        self.props.insert(name.to_owned(), Prototype::Method(b));
        self
    }

    pub fn constructor<T: 'static + Method>(mut self, ctor: T) -> Self {
        let b: Box<dyn Method> = Box::new(ctor);
        self.ctor = Some(b);
        self
    }

    pub fn inherit(mut self, parent: Function<'a>) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn build(self, ctx: &Context) -> DukResult<()> {
        unsafe { push_class_builder(ctx, self) }
    }
}

unsafe extern "C" fn class_ctor(ctx: *mut duk::duk_context) -> duk::duk_ret_t {
    //debug!("class constructor");
    duk::duk_push_current_function(ctx);

    if duk::duk_has_prop_lstring(ctx, -1, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len()) == 1 {
        //debug!("found custom class constructor");
        duk::duk_get_prop_lstring(ctx, -1, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len());
        let ptr = duk::duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
        duk::duk_pop(ctx);

        let c = Context::with(ctx);
        let this = match c.push_this().getp::<Reference>() {
            Err(e) => {
                duk::duk_error_raw(
                    ctx,
                    duk::DUK_ERR_ERROR as i32,
                    "".as_ptr() as *const i8,
                    0,
                    format!("ctor call failed: {}", e).as_ptr() as *const i8,
                );
                return 0;
            }
            Ok(s) => s,
        };

        let ctor = Box::from_raw(ptr);
        let mut instance = Instance::new(this);
        match ctor.call(&c, &mut instance) {
            Ok(_) => {}
            Err(e) => {
                Box::into_raw(ctor);
                duk::duk_error_raw(
                    ctx,
                    duk::DUK_ERR_ERROR as i32,
                    "".as_ptr() as *const i8,
                    0,
                    format!("ctor call failed: {}", e).as_ptr() as *const i8,
                );
                return 0;
            }
        };

        // We wanna keep the ctor on the heap
        Box::into_raw(ctor);
    }

    duk::duk_push_this(ctx);
    //duk::duk_push_pointer(ctx, Box::into_raw(instance) as *mut c_void);
    //duk::duk_put_prop_lstring(ctx, -2, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
    duk::duk_push_c_function(ctx, Some(class_dtor), 1);
    duk::duk_set_finalizer(ctx, -2);

    return 0;
}

unsafe extern "C" fn class_dtor(ctx: *mut duk::duk_context) -> duk::duk_ret_t {
    //debug!("class dtor");
    if duk::duk_has_prop_lstring(ctx, 0, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len()) == 1 {
        //debug!("dropping instance data");
        duk::duk_get_prop_lstring(ctx, 0, DATA_KEY.as_ptr() as *const i8, DATA_KEY.len());
        let ptr = duk::duk_get_pointer(ctx, -1) as *mut Instance;
        Box::from_raw(ptr);
        duk::duk_pop(ctx);
    }
    0
}

unsafe extern "C" fn constructor_dtor(ctx: *mut duk::duk_context) -> duk::duk_ret_t {
    //debug!("constructor dtor");

    if duk::duk_has_prop_lstring(ctx, 0, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len()) == 1 {
        //debug!("dropping class constructor");
        duk::duk_get_prop_lstring(ctx, 0, CTOR_KEY.as_ptr() as *const i8, CTOR_KEY.len());
        let ptr = duk::duk_get_pointer(ctx, -1) as *mut Box<dyn Method>;
        Box::from_raw(ptr);
        duk::duk_pop(ctx);
    }

    return 0;
}

pub(crate) unsafe fn push_class_builder<'a>(
    ctx: &'a Context,
    builder: ClassBuilder<'a>,
) -> DukResult<()> {
    duk::duk_push_c_function(ctx.inner, Some(class_ctor), duk::DUK_VARARGS);

    if !builder.name.is_empty() {
        ctx.push_string("name").push_string(builder.name);

        duk::duk_def_prop(
            ctx.inner,
            -3,
            duk::DUK_DEFPROP_HAVE_VALUE | duk::DUK_DEFPROP_FORCE,
        );
    }

    if let Some(parent) = builder.parent {
        ctx.get_global_string("Object")
            .push_string("create")
            .push(parent)?
            .get_prop_string(-1, "prototype")
            .remove(-2)
            .call_prop(-3, 1)?
            .remove(-2);
        ctx.dup(1)?.put_prop_string(-3, "__super__");
    } else {
        ctx.push_object();
    }

    for (name, method) in builder.props {
        match method {
            Prototype::Method(m) => {
                push_method(ctx, m);
                ctx.put_prop_string(-2, &name);
            }
        }
    }

    ctx.put_prop_string(-2, "prototype");

    if let Some(ctor) = builder.ctor {
        //debug!("push class constructor");
        let b = Box::new(ctor);
        duk::duk_push_pointer(ctx.inner, Box::into_raw(b) as *mut c_void);
        duk::duk_put_prop_lstring(
            ctx.inner,
            -2,
            CTOR_KEY.as_ptr() as *const i8,
            CTOR_KEY.len(),
        );
    }

    duk::duk_push_c_function(ctx.inner, Some(constructor_dtor), 1);
    duk::duk_set_finalizer(ctx.inner, -2);

    Ok(())
}
