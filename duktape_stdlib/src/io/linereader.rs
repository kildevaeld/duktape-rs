use super::readers::{ReadAllFn, ReadFn};
use super::writers::{FlushFn, WriteFn};
use duktape::prelude::*;
use super::types::get_line_reader;

pub(crate) struct ReadLineFn;

impl class::Method for ReadLineFn {
    fn call(&self, ctx: &Context, this: &mut class::Instance) -> DukResult<i32> {
        let reader = get_line_reader(ctx, this)?;

        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Err(e) => duk_error!(format!("error while reading: {}", e)),
            Ok(_) => {}
        };

        ctx.push(buffer.trim())?;

        Ok(1)
    }
}

pub(crate) fn build_linereader<'a>(ctx: &'a Context, parent: Function<'a>) -> DukResult<Function<'a>> {
    let mut readwriter = class::build();

    readwriter
        .name("LineReader")
        .inherit(parent)
        .method(
            "readLine",
            (1, |_ctx: &Context, _this: &mut class::Instance| {
                duk_error!("Don't use the Reader class directly")
            }),
        );

    let readwriter = ctx.push(readwriter)?.getp()?;

    Ok(readwriter)
}

pub(crate) fn build_linereader_class<'a>(
    ctx: &'a Context,
    linereader: Function<'a>,
) -> DukResult<Function<'a>> {
    let mut builder = class::build();

    builder
        .inherit(linereader)
        .method("read", ReadFn {})
        .method("readLine", ReadLineFn {})
        .method("readAll", ReadAllFn {});

    Ok(ctx.push(builder)?.getp()?)
}
