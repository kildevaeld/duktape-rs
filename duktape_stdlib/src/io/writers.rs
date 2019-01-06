use super::types::get_writer;
use duktape::prelude::*;

pub(crate) struct WriteFn;

impl class::Method for WriteFn {
    fn argc(&self) -> i32 {
        1
    }

    fn call(&self, ctx: &Context, this: &mut class::Instance) -> DukResult<i32> {
        let writer = get_writer(ctx, this)?;

        if ctx.is(Type::Undefined, 0) {
            duk_type_error!("invalid type");
        }

        let r = ctx.get::<Ref>(0)?;
        write!(writer, "{}", r).unwrap();

        ctx.push_this();
        Ok(1)
    }
}

pub(crate) struct FlushFn;

impl class::Method for FlushFn {
    fn argc(&self) -> i32 {
        0
    }

    fn call(&self, ctx: &Context, this: &mut class::Instance) -> DukResult<i32> {
        let writer = get_writer(ctx, this)?;
        writer.flush()?;
        ctx.push_this();
        Ok(1)
    }
}

pub(crate) fn build_writer<'a>(ctx: &'a Context) -> DukResult<Function<'a>> {
    let mut writer = class::build();

    writer
        .name("Writer")
        .method(
            "write",
            (1, |_ctx: &Context, _this: &mut class::Instance| {
                duk_error!("Don't use the Writer class directly")
            }),
        )
        .method("flush", |_ctx: &Context, _this: &mut class::Instance| {
            duk_error!("Don't use the Writer class directly")
        });

    let writer = ctx.push(writer)?.getp()?;

    Ok(writer)
}

pub(crate) fn build_write_writer_class<'a>(
    ctx: &'a Context,
    writer: Function<'a>,
) -> DukResult<Function<'a>> {
    let mut builder = class::build();

    builder
        .inherit(writer)
        .method("write", WriteFn {})
        .method("flush", FlushFn {});

    Ok(ctx.push(builder)?.getp()?)
}
