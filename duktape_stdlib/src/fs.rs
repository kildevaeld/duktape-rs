use super::sources::FS;
use duktape::prelude::*;
use duktape::{
    self,
    error::{ErrorKind, Result},
};
use duktape_modules::require;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

struct FileKey;

impl duktape::Key for FileKey {
    type Value = File;
}

fn get_file_options(input: &str) -> OpenOptions {
    let mut o = OpenOptions::new();

    match input {
        "r" => o.read(true),
        "w" => o.write(true).create(true).truncate(true),
        "rw" | "wr" => o.write(true).read(true).truncate(true),
        "w+" => o.write(true).create(true),
        "wr+" | "rw+" => o.write(true).read(true).create(true),
        _ => o.read(true),
    };

    o
}

pub fn init_file<'a>() -> class::Builder<'a> {
    let mut file = class::build();

    file.constructor((2, |ctx: &Context, this: &mut class::Instance| {
        let path: String = ctx.get(0)?;
        let options: OpenOptions;
        if ctx.is(Type::String, 1) {
            options = get_file_options(ctx.get::<&str>(1)?);
        } else {
            let mut o = OpenOptions::new();
            o.read(true);
            options = o;
        }

        let file = options.open(path).unwrap();
        this.data_mut().insert::<FileKey>(file);

        Ok(0)
    }))
    .method(
        "read",
        (1, |ctx: &Context, this: &mut class::Instance| {
            let file = this.data_mut().get_mut::<FileKey>().unwrap();

            let mut cap = 256;
            if ctx.is(Type::Number, 0) {
                cap = ctx.get(0)?;
            }

            let mut buffer = Vec::with_capacity(cap);
            file.read(&mut buffer).unwrap();
            ctx.push(buffer.as_slice())?;

            Ok(1)
        }),
    )
    .method(
        "write",
        (1, |ctx: &Context, this: &mut class::Instance| {
            let writer = match this.data_mut().get_mut::<FileKey>() {
                Some(w) => w,
                None => return Err(ErrorKind::TypeError("file closed".to_owned()).into()),
            };

            if ctx.is(Type::Undefined, 0) {
                return Err(ErrorKind::TypeError("invalid type".to_owned()).into());
            }

            let r = ctx.get::<Ref>(0)?;
            write!(writer, "{}", r).unwrap();

            ctx.push_this();
            Ok(1)
        }),
    )
    .method("flush", |ctx: &Context, this: &mut class::Instance| {
        let writer = match this.data_mut().get_mut::<FileKey>() {
            Some(w) => w,
            None => return Ok(0),
        };
        writer.flush().unwrap();
        ctx.push_this();
        Ok(1)
    })
    .method("close", |ctx: &Context, this: &mut class::Instance| {
        let writer = match this.data_mut().get_mut::<FileKey>() {
            Some(w) => w,
            None => return Ok(0),
        };
        drop(writer);
        this.data_mut().remove::<FileKey>();
        ctx.push_this();
        Ok(1)
    });

    file
}

pub fn init_fs(ctx: &Context) -> Result<i32> {
    let exports = ctx.create::<Object>()?;

    exports.set("File", init_file());

    let module: Object = ctx.get(-1)?;
    module.set("exports", exports);

    require::eval_module(ctx, FS, &module).unwrap();

    module.get::<_, Ref>("exports")?.push();

    //ctx.push(exports)?;
    Ok(1)
}
