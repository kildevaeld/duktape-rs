use super::sources::IO_JS;
use duktape::prelude::*;
use duktape::{
    self,
    error::{ErrorKind, Result},
};
use duktape_modules::require;
use std::any::Any;
use std::io::{self, BufRead, Read, Write};

pub struct WriterKey;

impl duktape::Key for WriterKey {
    type Value = Box<dyn Write>;
}

pub struct ReaderKey;

impl duktape::Key for ReaderKey {
    type Value = Box<dyn Read>;
}

pub struct BufReaderKey;

impl duktape::Key for BufReaderKey {
    type Value = Box<dyn BufRead>;
}

pub struct StdinReaderKey;

impl duktape::Key for StdinReaderKey {
    type Value = Box<io::Stdin>;
}

pub trait ReadWrite: Read + Write {}

pub struct ReadWriterKey;

impl duktape::Key for ReadWriterKey {
    type Value = Box<dyn ReadWrite>;
}

pub fn init_writer<'a>() -> duktape::class::Builder<'a> {
    let mut writer = duktape::class::build();
    writer
        .method(
            "write",
            (1, |ctx: &Context, this: &mut class::Instance| {
                let writer = match this.data_mut().get_mut::<WriterKey>() {
                    Some(w) => w,
                    None => return Ok(0),
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
            let writer = match this.data_mut().get_mut::<WriterKey>() {
                Some(w) => w,
                None => return Ok(0),
            };
            writer.flush().unwrap();
            ctx.push_this();
            Ok(1)
        });
    writer
}

fn get_reader<'a>(this: &'a mut class::Instance) -> Option<&'a mut Box<dyn Read>> {
    if this.data().contains::<ReaderKey>() {
        return this.data_mut().get_mut::<ReaderKey>();
    } else if this.data().contains::<BufReaderKey>() {
        return match this.data_mut().get_mut::<BufReaderKey>() {
            Some(m) => {
                let b_any = m as &mut Any;
                b_any.downcast_mut()
            }
            None => None,
        };
    } else if this.data().contains::<ReadWriterKey>() {
        return match this.data_mut().get_mut::<ReadWriterKey>() {
            Some(m) => {
                let b_any = m as &mut Any;
                b_any.downcast_mut()
            }
            None => None,
        };
    }
    None
}

fn get_buf_reader<'a>(this: &'a mut class::Instance) -> Option<&'a mut Box<dyn BufRead>> {
    if this.data().contains::<BufReaderKey>() {
        return this.data_mut().get_mut::<BufReaderKey>();
    }
    None
}

pub fn init_reader<'a>() -> duktape::class::Builder<'a> {
    let mut reader = duktape::class::build();
    reader
        .method(
            "read",
            (1, |ctx: &Context, this: &mut class::Instance| {
                let reader = match get_reader(this) {
                    Some(r) => r,
                    None => {
                        return Err(
                            ErrorKind::ReferenceError(format!("could not resovle reader")).into(),
                        )
                    }
                };

                let mut buffer = [0; 8192];
                let size;

                loop {
                    match reader.read(&mut buffer[..]) {
                        Err(e) => match e.kind() {
                            io::ErrorKind::Interrupted => {
                                continue;
                            }
                            _ => {
                                return Err(ErrorKind::ReferenceError(format!(
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
            }),
        )
        .method("readAll", |ctx: &Context, this: &mut class::Instance| {
            let reader = match get_reader(this) {
                Some(r) => r,
                None => {
                    return Err(
                        ErrorKind::ReferenceError(format!("could not resovle reader")).into(),
                    )
                }
            };

            let mut buffer = Vec::new();
            match reader.read_to_end(&mut buffer) {
                Err(e) => {
                    return Err(ErrorKind::Error(format!("error while reading: {}", e)).into())
                }
                Ok(_) => {}
            };

            ctx.push(buffer.as_slice())?;

            Ok(1)
        });
    reader
}

pub fn init_read_writer<'a>() -> duktape::class::Builder<'a> {
    let mut builder = class::build();

    builder
        .method(
            "read",
            (1, |ctx: &Context, this: &mut class::Instance| {
                let reader = match get_reader(this) {
                    Some(r) => r,
                    None => {
                        return Err(
                            ErrorKind::ReferenceError(format!("could not resovle reader")).into(),
                        )
                    }
                };

                let mut buffer = [0; 8192];
                let size;

                loop {
                    match reader.read(&mut buffer[..]) {
                        Err(e) => match e.kind() {
                            io::ErrorKind::Interrupted => {
                                continue;
                            }
                            _ => return Err(ErrorKind::ReferenceError(format!("{}", e)).into()),
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
            }),
        )
        .method("readAll", |ctx: &Context, this: &mut class::Instance| {
            let reader = match get_reader(this) {
                Some(r) => r,
                None => {
                    return Err(
                        ErrorKind::ReferenceError(format!("could not resovle reader")).into(),
                    )
                }
            };

            let mut buffer = Vec::new();
            match reader.read_to_end(&mut buffer) {
                Err(e) => {
                    return Err(ErrorKind::Error(format!("error while reading: {}", e)).into())
                }
                Ok(_) => {}
            };

            ctx.push(buffer.as_slice())?;

            Ok(1)
        });

    builder
}

pub fn init_buf_reader<'a>(reader: Function<'a>) -> duktape::class::Builder<'a> {
    let mut builder = class::build();

    builder
        .inherit(reader)
        .method("readLine", |ctx: &Context, this: &mut class::Instance| {
            let mut reader = match get_buf_reader(this) {
                Some(r) => r,
                None => {
                    return Err(
                        ErrorKind::ReferenceError(format!("could not resovle buf reader")).into(),
                    )
                }
            };

            //let mut reader = io::BufReader::new(reader);
            let mut output = String::new();
            reader.read_line(&mut output)?;
            ctx.push(output)?;

            Ok(1)
        });

    builder
}

pub fn init_io(ctx: &Context) -> Result<i32> {
    let exports = ctx.create::<Object>()?;

    exports.set("Writer", init_writer());
    exports.set("Reader", init_reader());

    let writer_ctor: Function = exports.get("Writer")?;
    let reader_ctor: Function = exports.get("Reader")?;

    exports.set("BufReader", init_buf_reader(reader_ctor));
    exports.set("ReadWriter", init_read_writer());

    let bufreader_ctor: Function = exports.get("BufReader")?;

    let mut stdout = duktape::class::build();
    stdout
        .constructor(|_ctx: &Context, this: &mut class::Instance| {
            this.data_mut().insert::<WriterKey>(Box::new(io::stdout()));
            Ok(0)
        })
        .inherit(writer_ctor.clone());

    let mut stderr = duktape::class::build();
    stderr
        .constructor(|_ctx: &Context, this: &mut class::Instance| {
            this.data_mut().insert::<WriterKey>(Box::new(io::stderr()));
            Ok(0)
        })
        .inherit(writer_ctor.clone());

    let mut stdin = duktape::class::build();
    stdin
        .constructor(|_ctx: &Context, this: &mut class::Instance| {
            this.data_mut()
                .insert::<StdinReaderKey>(Box::new(io::stdin()));
            Ok(0)
        })
        .inherit(bufreader_ctor)
        .method("readLine", |ctx: &Context, this: &mut class::Instance| {
            let reader = match this.data_mut().get_mut::<StdinReaderKey>() {
                Some(r) => r,
                None => {
                    return Err(
                        ErrorKind::ReferenceError(format!("could not resovle buf reader")).into(),
                    )
                }
            };

            let mut output = String::new();
            reader.read_line(&mut output)?;
            ctx.push(output.trim_end_matches("\n"))?;

            Ok(1)
        });

    exports
        .set("Stderr", stderr)
        .set("Stdout", stdout)
        .set("Stdin", stdin);

    exports.set("stdout", exports.construct("Stdout", ())?);
    exports.set("stderr", exports.construct("Stderr", ())?);
    exports.set("stdin", exports.construct("Stdin", ())?);

    let module: Object = ctx.get(-1)?;
    module.set("exports", exports);

    require::eval_module(ctx, IO_JS, &module).unwrap();

    module.get::<_, Ref>("exports")?.push();

    //ctx.push(module)?;

    Ok(1)
}
