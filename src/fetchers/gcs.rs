use crate::config::Config;
use crate::fetchers::Fetcher;
use crate::util::path_to_str;
use async_trait::async_trait;
use color_eyre::eyre::{eyre, Result};
use color_eyre::{Section, SectionExt};
use std::path::Path;
use std::process::Command;

struct Gcs<'a> {
    #[allow(dead_code)]
    config: &'a Config,
}

pub fn new<'a>(config: &'a Config) -> Box<dyn Fetcher + 'a> {
    Box::new(Gcs { config })
}

#[async_trait]
impl<'a> Fetcher for Gcs<'a> {
    async fn fetch(&self, url: &str, output: &Path) -> Result<()> {
        let mut cmd = Command::new("gsutil");
        cmd.args(&["cp", url, path_to_str(output).as_ref()]);
        debug!("{:?}", cmd);

        let output = cmd.output()?;

        match output.status.success() {
            true => Ok(()),
            false => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                Err(
                    eyre!("gsutil exited with status {}", output.status.to_string())
                        .with_section(move || stdout.trim().to_string().header("Stdout"))
                        .with_section(move || stderr.trim().to_string().header("Stderr")),
                )
            }
        }
    }
}
