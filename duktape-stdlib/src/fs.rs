use duktape;
use duktape::prelude::*;
use std::convert;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Read, Stdin, Write};

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
    println!("read {}", input);
    o
}

pub fn init_file<'a>() -> duktape::class::Builder<'a> {
    let mut file = duktape::class::build();

    file.constructor(2, |ctx, this| {
        let path: String = ctx.get(0)?;
        let options: OpenOptions;
        if ctx.is(Type::String, 1) {
            options = get_file_options(&ctx.get::<String>(1)?);
        } else {
            let mut o = OpenOptions::new();
            o.read(true);
            options = o;
        }
        println!("{:?}", options);
        let file = options.open(path).unwrap();
        this.data_mut().insert::<FileKey>(file);

        Ok(0)
    })
    .method("read", 1, |ctx, this| {
        let file = this.data_mut().get_mut::<FileKey>().unwrap();

        let mut buffer = Vec::with_capacity(256);
        file.read(&mut buffer).unwrap();
        ctx.push(buffer.as_slice());

        Ok(1)
    })
    .method("write", 1, |ctx, this| {
        let writer = match this.data_mut().get_mut::<FileKey>() {
            Some(w) => w,
            None => return Err(ErrorKind::TypeError("file closed".to_owned()).into()),
        };

        if ctx.is(Type::Undefined, 0) {
            return Err(ErrorKind::TypeError("invalid type".to_owned()).into());
        }

        let r = ctx.get::<Reference>(0)?;
        write!(writer, "{}", r);

        ctx.push_this();
        Ok(1)
    })
    .method("flush", 0, |ctx, this| {
        let writer = match this.data_mut().get_mut::<FileKey>() {
            Some(w) => w,
            None => return Ok(0),
        };
        writer.flush();
        ctx.push_this();
        Ok(1)
    })
    .method("close", 0, |ctx, this| {
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

pub fn init_fs(ctx: &mut Context) -> Result<i32> {
    let module = ctx.create::<Object>()?;

    module.set("File", init_file());

    module
        .set("mkdir", duktape::cb(1, Box::new(|ctx| Ok(0))))
        .set("mkdirAll", duktape::cb(1, Box::new(|ctx| Ok(0))));

    ctx.push(module);
    Ok(1)
}