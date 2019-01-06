use duktape::prelude::*;

use super::types::get_reader;
use std::io::{self, BufRead, Stdin, Stdout};

// impl Reader for Stdin {
//     fn read_line(&mut self, line: &mut String) -> io::Result<usize> {
//         self.lock().read_line(line)
//     }
// }

// impl Writer for Stdout {}

pub(crate) struct ReadFn;

impl class::Method for ReadFn {
    fn argc(&self) -> i32 {
        1
    }

    fn call(&self, ctx: &Context, this: &mut class::Instance) -> DukResult<i32> {
        let reader = get_reader(ctx, this)?;

        let mut buffer = if ctx.is(Type::Number, 0) {
            Vec::with_capacity(ctx.get_int(0)? as usize)
        } else {
            Vec::with_capacity(8192)
        };

        let size;

        loop {
            match reader.read(&mut buffer[..]) {
                Err(e) => match e.kind() {
                    io::ErrorKind::Interrupted => {
                        continue;
                    }
                    _ => {
                        return Err(DukErrorKind::ReferenceError(format!(
                            "could not resovle reader"
                        ))
                        .into())
                    }
                },
                Ok(s) => {
                    if s == 0 {
                        ctx.push_undefined();
                        return Ok(1);
                    } else {
                        size = s;
                        break;
                    }
                }
            }
        }

        ctx.push(&buffer[0..size])?;

        Ok(1)
    }
}

pub(crate) struct ReadAllFn;

impl class::Method for ReadAllFn {
    fn call(&self, ctx: &Context, this: &mut class::Instance) -> DukResult<i32> {
        let reader = get_reader(ctx, this)?;

        let mut buffer = Vec::new();
        match reader.read_to_end(&mut buffer) {
            Err(e) => duk_error!(format!("error while reading: {}", e)),
            Ok(_) => {}
        };

        ctx.push(buffer.as_slice())?;

        Ok(1)
    }
}

// pub(crate) struct ReadLineFn;

// impl class::Method for ReadLineFn {
//     fn call(&self, ctx: &Context, this: &mut class::Instance) -> DukResult<i32> {
//         let reader = get_reader(ctx, this)?;

//         let mut buffer = String::new();
//         match reader.read_line(&mut buffer) {
//             Err(e) => duk_error!(format!("error while reading: {}", e)),
//             Ok(_) => {}
//         };

//         ctx.push(buffer)?;

//         Ok(1)
//     }
// }

pub(crate) fn build_reader<'a>(ctx: &'a Context) -> DukResult<Function<'a>> {
    let mut reader = class::build();

    reader
        .name("Reader")
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

    let reader = ctx.push(reader)?.getp()?;

    Ok(reader)
}

pub(crate) fn build_read_reader_class<'a>(
    ctx: &'a Context,
    reader: Function<'a>,
) -> DukResult<Function<'a>> {
    let mut builder = class::build();

    builder
        .inherit(reader)
        .method("read", ReadFn {})
        //.method("readLine", ReadLineFn {})
        .method("readAll", ReadAllFn {});

    Ok(ctx.push(builder)?.getp()?)
}
