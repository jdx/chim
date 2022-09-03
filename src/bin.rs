#[cfg(unix)]
use color_eyre::eyre::{eyre};
use color_eyre::eyre::{Result, Context};
use std::process::{exit, Command};

pub fn exec(filename: &str, args: &[String], execvp: bool) -> Result<()> {
    let exec_args: Vec<String> = args.iter().skip(2).map(|s| s.to_string()).collect();

    if execvp {
        do_execvp(filename, &exec_args)?;
    } else {
        do_subprocess(filename, &exec_args)?;
    }

    Ok(())
}

#[cfg(unix)]
fn do_execvp(filename: &str, exec_args: &[String]) -> Result<()> {
    debug!("execvp: {} {}", filename, exec_args.join(" "));
    let err = exec::Command::new(filename).args(exec_args).exec();

    // always errors if it gets here
    Err(eyre!("Error executing {}: {}", filename, err))
    // .suggestion("Try running with $CHIM_EXECVP=0"))
}

#[cfg(not(unix))]
fn do_execvp(filename: &str, exec_args: &[String]) -> Result<()> {
    do_subprocess(filename, exec_args)
}

fn do_subprocess(filename: &str, exec_args: &[String]) -> Result<()> {
    debug!("subprocess: {} {}", filename, exec_args.join(" "));
    let status = Command::new(filename).args(exec_args).status().with_context(|| format!(
        "Error executing {} with args: {}",
        filename,
        exec_args.join(" ")
    ))?;

    debug!("subprocess exited with {status}");

    if status.code() != Some(0) {
        exit(status.code().unwrap_or(1));
    }

    Ok(())
}
