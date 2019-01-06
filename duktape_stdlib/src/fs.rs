use super::io as dukio;
use super::sources::FS;
use duktape::prelude::*;
use duktape::{self, error::Result};
use duktape_modules::require;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead};

// impl dukio::Reader for File {
//     fn read_line(&mut self, line: &mut String) -> io::Result<usize> {
//         let mut reader = io::BufReader::new(self);
//         reader.read_line(line)
//     }
// }

// impl dukio::Writer for File {}

//impl dukio::ReadWriter for File {}

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

pub fn init_file<'a>(ctx: &'a Context) -> DukResult<class::Builder<'a>> {
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

        this.data_mut()
            .insert::<dukio::ReadWriterKey>(dukio::IOReadWriter::new(file));

        Ok(0)
    }))
    .method("close", |ctx: &Context, this: &mut class::Instance| {
        let writer = match this.data_mut().get_mut::<dukio::ReadWriterKey>() {
            Some(w) => w,
            None => return Ok(0),
        };
        drop(writer);
        this.data_mut().remove::<dukio::ReadWriterKey>();
        ctx.push_this();
        Ok(1)
    });

    file = dukio::inherit_readwriter(ctx, file)?;

    Ok(file)
}

fn mkdir(ctx: &Context) -> Result<i32> {
    let path = ctx.get::<&str>(0)?;
    fs::create_dir(path)?;
    Ok(0)
}

fn mkdir_all(ctx: &Context) -> Result<i32> {
    let path = ctx.get::<&str>(0)?;
    fs::create_dir_all(path)?;
    Ok(0)
}

fn rmdir(ctx: &Context) -> Result<i32> {
    let path = ctx.get::<&str>(0)?;
    fs::remove_dir(path)?;
    Ok(0)
}

fn rmdir_all(ctx: &Context) -> Result<i32> {
    let path = ctx.get::<&str>(0)?;
    fs::remove_dir_all(path)?;
    Ok(0)
}

fn rmfile(ctx: &Context) -> Result<i32> {
    let path = ctx.get::<&str>(0)?;
    fs::remove_file(path)?;
    Ok(0)
}

fn readdir(ctx: &Context) -> Result<i32> {
    let path = ctx.get::<&str>(0)?;
    let files = fs::read_dir(path)?
        .filter_map(|file| {
            if file.is_err() {
                return None;
            }

            let file = file.unwrap().path();
            let path = file.to_str().unwrap().to_string();
            Some(path)
        })
        .collect::<Vec<_>>();

    ctx.push(files)?;

    Ok(1)
}

pub fn init_fs(ctx: &Context) -> Result<i32> {
    let exports = ctx.create::<Object>()?;

    exports
        .set("File", init_file(ctx)?)
        .set("mkdir", (1, mkdir))
        .set("mkdirAll", (1, mkdir_all))
        .set("rmdir", (1, rmdir))
        .set("rmdirAll", (1, rmdir_all))
        .set("unlink", (1, rmfile))
        .set("readdir", (1, readdir));

    let module: Object = ctx.get(-1)?;
    module.set("exports", exports);

    // Enchance the export with some js goodies
    require::eval_module(ctx, FS, &module).unwrap();

    module.get::<_, Ref>("exports")?.push();

    Ok(1)
}
