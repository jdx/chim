use crate::config::Config;
use crate::fetchers::Fetcher;
use crate::util::path_to_str;
use async_trait::async_trait;
use color_eyre::eyre::{eyre, Result};
use color_eyre::{Section, SectionExt};
use std::path::Path;
use std::process::Command;

struct Ssh<'a> {
    #[allow(dead_code)]
    config: &'a Config,
}

pub fn new<'a>(config: &'a Config) -> Box<dyn Fetcher + 'a> {
    Box::new(Ssh { config })
}

#[async_trait]
impl<'a> Fetcher for Ssh<'a> {
    async fn fetch(&self, url: &str, output: &Path) -> Result<()> {
        let mut cmd = Command::new("scp");
        cmd.args(&[url, path_to_str(output).as_ref()]);
        debug!("{:?}", cmd);

        let output = cmd.output()?;

        match output.status.success() {
            true => Ok(()),
            false => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                Err(
                    eyre!("ssh exited with status {}", output.status.to_string())
                        .with_section(move || stdout.trim().to_string().header("Stdout"))
                        .with_section(move || stderr.trim().to_string().header("Stderr")),
                )
            }
        }
    }
}
