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

impl ArgumentList for () {
    fn len(&self) -> i32 {
        0
    }
    fn push(self, ctx: &Context) {}
}

impl<'a, T1: 'a + Serialize, T2: 'a + Serialize> ArgumentList for (T1, T2)
where
    &'a T1: Serialize,
    &'a T2: Serialize,
{
    fn len(&self) -> i32 {
        2
    }
    fn push(self, ctx: &Context) {
        ctx.push(self.0);
        ctx.push(self.1);
    }
}

impl<'a, T1: 'a + Serialize, T2: 'a + Serialize, T3: 'a + Serialize> ArgumentList for (T1, T2, T3)
where
    &'a T1: Serialize,
    &'a T2: Serialize,
    &'a T3: Serialize,
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

impl<'a, T1: 'a + Serialize, T2: 'a + Serialize, T3: 'a + Serialize, T4: 'a + Serialize>
    ArgumentList for (T1, T2, T3, T4)
where
    &'a T1: Serialize,
    &'a T2: Serialize,
    &'a T3: Serialize,
    &'a T4: Serialize,
{
    fn len(&self) -> i32 {
        4
    }
    fn push(self, ctx: &Context) {
        ctx.push(self.0);
        ctx.push(self.1);
        ctx.push(self.2);
        ctx.push(self.3);
    }
}

impl<
        'a,
        T1: 'a + Serialize,
        T2: 'a + Serialize,
        T3: 'a + Serialize,
        T4: 'a + Serialize,
        T5: 'a + Serialize,
    > ArgumentList for (T1, T2, T3, T4, T5)
where
    &'a T1: Serialize,
    &'a T2: Serialize,
    &'a T3: Serialize,
    &'a T4: Serialize,
    &'a T5: Serialize,
{
    fn len(&self) -> i32 {
        5
    }
    fn push(self, ctx: &Context) {
        ctx.push(self.0);
        ctx.push(self.1);
        ctx.push(self.2);
        ctx.push(self.3);
        ctx.push(self.4);
    }
}
