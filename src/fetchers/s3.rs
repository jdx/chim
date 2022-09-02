use crate::config::Config;
use crate::fetchers::Fetcher;
use crate::util::path_to_str;
use async_trait::async_trait;
use color_eyre::eyre::{eyre, Result};
use color_eyre::{Section, SectionExt};
use std::path::Path;
use std::process::Command;

struct S3<'a> {
    config: &'a Config,
}

pub fn new<'a>(config: &'a Config) -> Box<dyn Fetcher + 'a> {
    Box::new(S3 { config })
}

#[async_trait]
impl<'a> Fetcher for S3<'a> {
    async fn fetch(&self, url: &str, output: &Path) -> Result<()> {
        let mut cmd = Command::new("aws");
        cmd.args(&["s3", "cp", url, path_to_str(output).as_ref()]);
        self.add_aws_args(&mut cmd);

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
}

impl S3<'_> {
    fn add_aws_args(&self, cmd: &mut Command) {
        match &self.config.aws_profile {
            Some(aws_profile) => {
                cmd.env("AWS_PROFILE", aws_profile);
            }
            None => {}
        }
        match &self.config.aws_access_key_id {
            Some(aws_access_key_id) => {
                cmd.env("AWS_ACCESS_KEY_ID", aws_access_key_id);
            }
            None => {}
        }
        match &self.config.aws_secret_access_key {
            Some(aws_secret_access_key) => {
                cmd.env("AWS_SECRET_ACCESS_KEY", aws_secret_access_key);
            }
            None => {}
        }
        match &self.config.aws_access_token {
            Some(aws_access_token) => {
                cmd.env("AWS_ACCESS_TOKEN", aws_access_token);
            }
            None => {}
        }
        match &self.config.aws_region {
            Some(aws_region) => {
                cmd.env("AWS_DEFAULT_REGION", aws_region);
            }
            None => {}
        }
    }
}
