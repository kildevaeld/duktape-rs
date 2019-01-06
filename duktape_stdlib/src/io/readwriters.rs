use super::readers::{ReadAllFn, ReadFn};
use super::writers::{FlushFn, WriteFn};
use duktape::prelude::*;

pub(crate) fn build_readwriter<'a>(ctx: &'a Context) -> DukResult<Function<'a>> {
    let mut readwriter = class::build();

    readwriter
        .name("ReadWriter")
        .method(
            "write",
            (1, |_ctx: &Context, _this: &mut class::Instance| {
                duk_error!("Don't use the Writer class directly")
            }),
        )
        .method("flush", |_ctx: &Context, _this: &mut class::Instance| {
            duk_error!("Don't use the Writer class directly")
        })
        .method(
            "read",
            (1, |_ctx: &Context, _this: &mut class::Instance| {
                duk_error!("Don't use the Reader class directly")
            }),
        )
        .method(
            "readAll",
            (1, |_ctx: &Context, _this: &mut class::Instance| {
                duk_error!("Don't use the Reader class directly")
            }),
        );

    let readwriter = ctx.push(readwriter)?.getp()?;

    Ok(readwriter)
}

pub(crate) fn build_readwriter_class<'a>(
    ctx: &'a Context,
    readwriter: Function<'a>,
) -> DukResult<Function<'a>> {
    let mut builder = class::build();

    builder
        .inherit(readwriter)
        .method("write", WriteFn {})
        .method("flush", FlushFn {})
        .method("read", ReadFn {})
        //.method("readLine", ReadLineFn {})
        .method("readAll", ReadAllFn {});

    Ok(ctx.push(builder)?.getp()?)
}
