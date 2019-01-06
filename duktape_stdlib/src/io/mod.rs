use duktape::prelude::*;
use duktape_modules::Builder;
mod readers;
mod readwriters;
mod types;
mod writers;
mod linereader;

pub use self::types::*;
use super::sources::IO_JS;
use duktape_modules::{require, CJSContext};
use std::io;

pub fn register(_ctx: &Context, builder: &mut Builder) {
    builder.module("io", build_io);
}

fn build_io(ctx: &Context) -> DukResult<i32> {
    let (reader, writer, readwriter) = (
        readers::build_reader(ctx)?,
        writers::build_writer(ctx)?,
        readwriters::build_readwriter(ctx)?,
    );

    let linereader = linereader::build_linereader(ctx, reader.clone())?;

    let (read_builder, write_builder, readwrite_builder, linereader_builder) = (
        readers::build_read_reader_class(ctx, reader.clone())?,
        writers::build_write_writer_class(ctx, writer.clone())?,
        readwriters::build_readwriter_class(ctx, readwriter.clone())?,
        linereader::build_linereader_class(ctx, linereader.clone())?,
    );

    let exports: Object = ctx.create()?;

    exports
        .set(b"\xFFreader", read_builder.clone())
        .set(b"\xFFwriter", write_builder.clone())
        .set(b"\xFFreadwriter", readwrite_builder.clone())
        .set(b"\xFFlinereader", linereader_builder.clone());


    let mut stdin_builder = class::build();
    stdin_builder.constructor(|ctx:&Context, this: &mut class::Instance| {
        this.data_mut()
            .insert::<LineReaderKey>(IOLineReader::new(io::stdin()));
        Ok(0)
    }).inherit(linereader_builder);

    
    ctx.push(stdin_builder)?.construct(0)?;
    let stdin = ctx.getp::<Ref>()?;

    ctx.push(write_builder.clone())?.construct(0)?;
    class::get_instance(ctx, -1, |this| {
        this.data_mut()
            .insert::<WriterKey>(IOWriter::new(io::stdout()));
        Ok(())
    })?;

    let stdout = ctx.getp::<Ref>()?;

    exports
        .set("Reader", reader)
        .set("Writer", writer)
        .set("ReadWriter", readwriter)
        .set("LineReader", linereader)
        .set("stdin", stdin)
        .set("stdout", stdout);

    let module: Object = ctx.get(-1)?;
    module.set("exports", exports);

    require::eval_module(ctx, IO_JS, &module).unwrap();

    module.get::<_, Ref>("exports")?.push();


    Ok(1)
}

macro_rules! inherit_impl {
    ($name: ident, $key: expr) => {
        pub fn $name<'a>(ctx: &'a Context, mut builder: class::Builder<'a>) -> DukResult<class::Builder<'a>> {
            let module = ctx.require("io").unwrap();
            let parent = module.get::<_, Function>($key)?;
            builder.inherit(parent);
            Ok(builder)
        }
    };
}

inherit_impl!(inherit_reader, b"\xFFreader");
inherit_impl!(inherit_readwriter, b"\xFFreadwriter");
inherit_impl!(inherit_writer, b"\xFFwriter");
inherit_impl!(inherit_linereader, b"\xFFlinereader");


// pub fn inherit_reader<'a>(
//     ctx: &'a Context,
//     mut builder: class::Builder<'a>,
// ) -> DukResult<class::Builder<'a>> {
//     let module = ctx.require("io").unwrap();
//     let reader = module.get::<_, Function>(b"\xFFreader")?;
//     builder.inherit(reader);
//     Ok(builder)
// }

// pub fn inherit_readwriter<'a>(
//     ctx: &'a Context,
//     mut builder: class::Builder<'a>,
// ) -> DukResult<class::Builder<'a>> {
//     let module = ctx.require("io").unwrap();
//     let readwriter = module.get::<_, Function>(b"\xFFreadwriter")?;
//     builder.inherit(readwriter);
//     Ok(builder)
// }

// pub fn inherit_writer<'a>(
//     ctx: &'a Context,
//     mut builder: class::Builder<'a>,
// ) -> DukResult<class::Builder<'a>> {
//     let module = ctx.require("io").unwrap();
//     let writer = module.get::<_, Function>(b"\xFFwriter")?;
//     builder.inherit(writer);
//     Ok(builder)
// }

