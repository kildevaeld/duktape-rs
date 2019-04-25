use super::error::DukResult;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use typemap::Key;

struct DiscardWriter;

impl Write for DiscardWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct Pipes {
    stdin: Box<dyn Read>,
    stdout: Box<dyn Write>,
    stderr: Box<dyn Write>,
}

impl Default for Pipes {
    fn default() -> Pipes {
        Pipes {
            stdin: Box::new(io::empty()),
            stdout: Box::new(DiscardWriter),
            stderr: Box::new(DiscardWriter),
        }
    }
}

impl Pipes {
    pub fn from_env() -> Pipes {
        Pipes {
            stdin: Box::new(std::io::stdin()),
            stdout: Box::new(std::io::stdout()),
            stderr: Box::new(std::io::stderr()),
        }
    }

    pub fn stdin(&self) -> &Read {
        &self.stdin
    }

    pub fn stdout(&self) -> &Write {
        &self.stdout
    }

    pub fn stdout_mut(&mut self) -> &mut Write {
        &mut self.stdout
    }

    pub fn stderr(&self) -> &Write {
        &self.stderr
    }

    pub fn stderr_mut(&mut self) -> &mut Write {
        &mut self.stderr
    }
}

#[derive(Default)]
pub struct EnvironmentBuilder {
    cwd: PathBuf,
    pipes: Pipes,
    env: Option<Box<Fn() -> HashMap<String, String>>>,
    args: Option<Vec<String>>,
}

impl EnvironmentBuilder {
    pub fn form_env() -> DukResult<EnvironmentBuilder> {
        Ok(EnvironmentBuilder {
            cwd: std::env::current_dir()?,
            pipes: Pipes::from_env(),
            env: None,
            args: Some(std::env::args().collect()),
        })
    }

    pub fn build(self) -> Environment {
        Environment {
            cwd: self.cwd,
            pipes: self.pipes,
            env: self.env,
            args: self.args,
        }
    }
}

#[derive(Default)]
pub struct Environment {
    cwd: PathBuf,
    pipes: Pipes,
    env: Option<Box<Fn() -> HashMap<String, String>>>,
    args: Option<Vec<String>>,
}

impl Environment {
    pub fn build() -> DukResult<EnvironmentBuilder> {
        EnvironmentBuilder::form_env()
    }

    pub fn from_env() -> DukResult<Environment> {
        Ok(EnvironmentBuilder::form_env()?.build())
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn pipes(&self) -> &Pipes {
        &self.pipes
    }

    pub fn pipes_mut(&mut self) -> &mut Pipes {
        &mut self.pipes
    }

    pub fn env(&self) -> Option<HashMap<String, String>> {
        match &self.env {
            Some(s) => Some(s()),
            None => None,
        }
    }

    pub fn args(&self) -> Option<&Vec<String>> {
        match &self.args {
            Some(s) => Some(s),
            None => None,
        }
    }
}

impl Key for Environment {
    type Value = Environment;
}
