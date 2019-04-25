use super::super::{
    class, context::Context, error::DukResult, from_context::*, function::Function,
};

pub(crate) fn build_reader<'a>(ctx: &'a Context) -> DukResult<Function<'a>> {
    class::new()
        .name("Reader")
        .method(
            "read",
            (1, |_ctx: &Context, _this: &mut class::Instance| {
                duk_error!("Don't use the Reader class directly")
            }),
        )
        .method("readAll", |_ctx: &Context, _this: &mut class::Instance| {
            duk_error!("Don't use the Reader class directly")
        })
        .build(ctx)?;

    let reader = ctx.getp()?;

    Ok(reader)
}

pub(crate) fn build_writer<'a>(ctx: &'a Context) -> DukResult<Function<'a>> {
    class::new()
        .name("Writer")
        .method(
            "write",
            (1, |_ctx: &Context, _this: &mut class::Instance| {
                duk_error!("Don't use the Writer class directly")
            }),
        )
        .method("flush", |_ctx: &Context, _this: &mut class::Instance| {
            duk_error!("Don't use the Writer class directly")
        })
        .build(ctx);

    let writer = ctx.getp()?;

    Ok(writer)
}
