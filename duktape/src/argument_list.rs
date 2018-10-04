use super::context::Context;
use super::encoding::Serialize;

pub trait ArgumentList {
    fn len(&self) -> i32;
    fn push(self, ctx: &Context);
}

impl<'a> ArgumentList for &'a str {
    fn len(&self) -> i32 {
        1
    }

    fn push(self, context: &Context) {
        let len = self.len();
        let data = self.as_ptr() as *const i8;
        unsafe {
            duktape_sys::duk_push_lstring(context.inner, data, len);
        };
    }
}

impl<T1: 'static + Serialize, T2: 'static + Serialize> ArgumentList for (T1, T2)
where
    &T1: Serialize,
    &T2: Serialize,
{
    fn len(&self) -> i32 {
        2
    }
    fn push(self, ctx: &Context) {
        ctx.push(self.0);
        ctx.push(self.1);
    }
}

impl<T1: 'static + Serialize, T2: 'static + Serialize, T3: 'static + Serialize> ArgumentList
    for (T1, T2, T3)
where
    &T1: Serialize,
    &T2: Serialize,
    &T3: Serialize,
{
    fn len(&self) -> i32 {
        3
    }
    fn push(self, ctx: &Context) {
        ctx.push(self.0);
        ctx.push(self.1);
        ctx.push(self.2);
    }
}
