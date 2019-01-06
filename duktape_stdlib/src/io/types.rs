use duktape::prelude::*;
use std::io::{self, Read, Write};

// pub trait Reader: Read {
//     fn read_line(&mut self, line: &mut String) -> io::Result<usize>;
// }

// pub trait Writer: Write {}

pub trait ReadWriter: Write + Read {}

// trait ReadWriter = Read + Write;

pub(crate) fn get_reader<'a>(
    _ctx: &'a Context,
    this: &'a mut class::Instance,
) -> DukResult<&'a mut Read> {
    let reader = if this.data().contains::<ReaderKey>() {
        match this.data_mut().get_mut::<ReaderKey>() {
            Some(m) => Some(m as &'a mut Read),
            None => None,
        }
    } else if this.data().contains::<ReadWriterKey>() {
        match this.data_mut().get_mut::<ReadWriterKey>() {
            Some(m) => Some(m as &'a mut Read),
            None => None,
        }
    } else {
        None
    };

    if reader.is_some() {
        return Ok(reader.unwrap());
    }

    duk_reference_error!("could not resolver reader")
}

pub(crate) fn get_writer<'a>(
    _ctx: &'a Context,
    this: &'a mut class::Instance,
) -> DukResult<&'a mut Write> {
    let writer = if this.data().contains::<WriterKey>() {
        match this.data_mut().get_mut::<WriterKey>() {
            Some(m) => Some(m as &'a mut Write),
            None => None,
        }
    } else if this.data().contains::<ReadWriterKey>() {
        match this.data_mut().get_mut::<ReadWriterKey>() {
            Some(m) => Some(m as &'a mut Write),
            None => None,
        }
    } else {
        None
    };

    if writer.is_some() {
        return Ok(writer.unwrap());
    }

    duk_reference_error!("could not resolver writer")
}

macro_rules! key_impl {
    ($name: ident, $trait: ident) => {
        pub struct $name;

        impl duktape::Key for $name {
            type Value = $trait;
        }
    };
}

key_impl!(ReaderKey, IOReader);
key_impl!(WriterKey, IOWriter);
key_impl!(ReadWriterKey, IOReadWriter);

pub struct IOReader {
    inner: Box<dyn Read>,
}

impl IOReader {
    pub fn new<T: Read + 'static>(reader: T) -> IOReader {
        return IOReader {
            inner: Box::new(reader),
        };
    }
}

// impl Reader for IOReader {
//     fn read_line(&mut self, line: &mut String) -> io::Result<usize> {
//         self.inner.read_line(line)
//     }
// }

impl Read for IOReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

pub struct IOReadWriter {
    inner: Box<dyn ReadWriter>,
}

impl IOReadWriter {
    pub fn new<T: ReadWriter + 'static>(reader: T) -> IOReadWriter {
        return IOReadWriter {
            inner: Box::new(reader),
        };
    }
}

// impl Reader for IOReadWriter {
//     fn read_line(&mut self, line: &mut String) -> io::Result<usize> {
//         self.inner.read_line(line)
//     }
// }

impl Read for IOReadWriter {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for IOReadWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

// impl Writer for IOReadWriter {}

pub struct IOWriter {
    inner: Box<dyn Write>,
}

impl IOWriter {
    pub fn new<T: Write + 'static>(reader: T) -> IOWriter {
        return IOWriter {
            inner: Box::new(reader),
        };
    }
}

impl Write for IOWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

// impl Writer for IOWriter {}
