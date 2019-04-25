use super::super::context::Context;
use super::super::error::DukResult;
use super::super::to_context::ToDuktape;

macro_rules! push_or_pop {
    ($dims:expr, $ctx: ident, $popc: expr) => {
        match $dims.to_context($ctx) {
            Ok(_) => {}
            Err(e) => {
                $ctx.pop($popc);
                return Err(e);
            }
        };
    };
}

pub trait ArgumentList {
    fn len(&self) -> i32;
    fn push_args(self, ctx: &Context) -> DukResult<()>;
}

impl<'a> ArgumentList for &'a str {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> DukResult<()> {
        ctx.push_string(self);
        Ok(())
    }
}

impl<'a> ArgumentList for &'a [u8] {
    fn len(&self) -> i32 {
        1
    }

    fn push_args(self, ctx: &Context) -> DukResult<()> {
        ctx.push_bytes(self);
        Ok(())
    }
}

impl ArgumentList for () {
    fn len(&self) -> i32 {
        0
    }
    fn push_args(self, _ctx: &Context) -> DukResult<()> {
        Ok(())
    }
}

impl<'a, T1: 'a + ToDuktape, T2: 'a + ToDuktape> ArgumentList for (T1, T2)
where
    &'a T1: ToDuktape,
    &'a T2: ToDuktape,
{
    fn len(&self) -> i32 {
        2
    }
    fn push_args(self, ctx: &Context) -> DukResult<()> {
        push_or_pop!(self.0, ctx, 0);
        push_or_pop!(self.1, ctx, 1);
        Ok(())
    }
}

impl<'a, T1: 'a + ToDuktape, T2: 'a + ToDuktape, T3: 'a + ToDuktape> ArgumentList for (T1, T2, T3)
where
    &'a T1: ToDuktape,
    &'a T2: ToDuktape,
    &'a T3: ToDuktape,
{
    fn len(&self) -> i32 {
        3
    }
    fn push_args(self, ctx: &Context) -> DukResult<()> {
        push_or_pop!(self.0, ctx, 0);
        push_or_pop!(self.1, ctx, 1);
        push_or_pop!(self.2, ctx, 2);
        Ok(())
    }
}

impl<'a, T1: 'a + ToDuktape, T2: 'a + ToDuktape, T3: 'a + ToDuktape, T4: 'a + ToDuktape>
    ArgumentList for (T1, T2, T3, T4)
where
    &'a T1: ToDuktape,
    &'a T2: ToDuktape,
    &'a T3: ToDuktape,
    &'a T4: ToDuktape,
{
    fn len(&self) -> i32 {
        4
    }
    fn push_args(self, ctx: &Context) -> DukResult<()> {
        push_or_pop!(self.0, ctx, 0);
        push_or_pop!(self.1, ctx, 1);
        push_or_pop!(self.2, ctx, 2);
        push_or_pop!(self.3, ctx, 3);
        Ok(())
    }
}

impl<
        'a,
        T1: 'a + ToDuktape,
        T2: 'a + ToDuktape,
        T3: 'a + ToDuktape,
        T4: 'a + ToDuktape,
        T5: 'a + ToDuktape,
    > ArgumentList for (T1, T2, T3, T4, T5)
where
    &'a T1: ToDuktape,
    &'a T2: ToDuktape,
    &'a T3: ToDuktape,
    &'a T4: ToDuktape,
    &'a T5: ToDuktape,
{
    fn len(&self) -> i32 {
        5
    }
    fn push_args(self, ctx: &Context) -> DukResult<()> {
        push_or_pop!(self.0, ctx, 0);
        push_or_pop!(self.1, ctx, 1);
        push_or_pop!(self.2, ctx, 2);
        push_or_pop!(self.3, ctx, 3);
        push_or_pop!(self.4, ctx, 4);
        Ok(())
    }
}
