use super::super::error::DukResult;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct Pipes {
    stdin: Box<dyn Read>,
    stdout: Box<dyn Write>,
    stderr: Box<dyn Write>,
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
}

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
