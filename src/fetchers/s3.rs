use crate::config::Config;
use color_eyre::eyre::{eyre, Result};
use color_eyre::{Section, SectionExt};
use std::path::Path;
use std::process::Command;

pub fn fetch(config: &Config, url: &str, output: &Path) -> Result<()> {
    let mut cmd = Command::new("aws");
    cmd.args(["s3", "cp", url, output.to_str().unwrap()]);
    add_aws_args(config, &mut cmd);

    let output = cmd.output()?;

    match output.status.success() {
        true => Ok(()),
        false => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            Err(
                eyre!("aws exited with status {}", output.status.to_string())
                    .with_section(move || stdout.trim().to_string().header("Stdout"))
                    .with_section(move || stderr.trim().to_string().header("Stderr")),
            )
        }
    }
}

fn add_aws_args(config: &Config, cmd: &mut Command) {
    match &config.aws_profile {
        Some(aws_profile) => {
            cmd.env("AWS_PROFILE", aws_profile);
        }
        None => {}
    }
    match &config.aws_access_key_id {
        Some(aws_access_key_id) => {
            cmd.env("AWS_ACCESS_KEY_ID", aws_access_key_id);
        }
        None => {}
    }
    match &config.aws_secret_access_key {
        Some(aws_secret_access_key) => {
            cmd.env("AWS_SECRET_ACCESS_KEY", aws_secret_access_key);
        }
        None => {}
    }
    match &config.aws_access_token {
        Some(aws_access_token) => {
            cmd.env("AWS_ACCESS_TOKEN", aws_access_token);
        }
        None => {}
    }
    match &config.aws_region {
        Some(aws_region) => {
            cmd.env("AWS_DEFAULT_REGION", aws_region);
        }
        None => {}
    }
}
