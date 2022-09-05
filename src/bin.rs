#[cfg(unix)]
use color_eyre::eyre::eyre;
use color_eyre::eyre::{Context, Result};
use itertools::Itertools;
use std::ffi::OsStr;
use std::process::{exit, Command};

pub struct Bin<T>
where
    T: AsRef<OsStr>,
{
    program: T,
    args: Vec<T>,
    execvp: bool,
}

pub fn new<T>(program: T) -> Bin<T>
where
    T: AsRef<OsStr>,
{
    Bin {
        program,
        args: Vec::new(),
        execvp: false,
    }
}

impl<T> Bin<T>
where
    T: AsRef<OsStr>,
{
    pub fn args<I>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.args = args.into_iter().collect();

        self
    }

    pub fn execvp(mut self, execvp: bool) -> Self {
        self.execvp = execvp;

        self
    }

    pub fn exec(self) -> Result<()> {
        match self.execvp {
            true => self.do_execvp(),
            false => self.do_subprocess(),
        }
        .with_context(|| {
            format!(
                "Error executing {} with args: {}",
                display_os_str(&self.program),
                display_args(&self.args),
            )
        })
    }

    #[cfg(unix)]
    #[cfg(not(tarpaulin_include))]
    fn do_execvp(&self) -> Result<()> {
        debug!(
            "execvp: {} {}",
            display_os_str(&self.program),
            display_args(&self.args)
        );
        let err = exec::Command::new(&self.program).args(&self.args).exec();

        // always errors if it gets here
        Err(eyre!("execvp failed: {}", err))
    }

    #[cfg(not(unix))]
    fn do_execvp(&self) -> Result<()> {
        self.do_subprocess()
    }

    fn do_subprocess(&self) -> Result<()> {
        debug!(
            "subprocess: {} {}",
            display_os_str(&self.program),
            display_args(&self.args)
        );

        let status = Command::new(&self.program).args(&self.args).status()?;

        debug!("subprocess exited with {status}");

        if status.code() != Some(0) {
            exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}

fn display_args<I>(args: &[I]) -> String
where
    I: AsRef<OsStr>,
{
    args.iter().map(display_os_str).join(" ")
}

fn display_os_str<T>(os_str: T) -> String
where
    T: AsRef<OsStr>,
{
    return os_str.as_ref().to_string_lossy().to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec() {
        let result = new("test").args(["1"]).exec();
        assert!(result.is_ok());
    }

    #[test]
    fn test_exec_invalid_bin() {
        let result = new("invalid_bin").exec();
        assert!(result.is_err());
    }
}
