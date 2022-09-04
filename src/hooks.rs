use crate::config::Config;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use std::process::{Command, Stdio};

pub struct Hooks<'a> {
    config: &'a Config,
}

impl<'a> Hooks<'a> {
    pub fn new(config: &Config) -> Hooks {
        Hooks { config }
    }

    pub fn pre_fetch(&self) -> Result<String> {
        self.exec_hook("pre_fetch", &self.config.pre_fetch)
    }
    pub fn pre_extract(&self) -> Result<String> {
        self.exec_hook("pre_extract", &self.config.pre_extract)
    }

    pub fn pre_execute(&self) -> Result<String> {
        // TODO: check config.pre_execute_interval
        self.exec_hook("pre_execute", &self.config.pre_execute)
    }

    pub fn post_execute(&self) -> Result<String> {
        self.exec_hook("post_execute", &self.config.post_execute)
    }

    fn exec_hook(&self, hook: &str, script: &Option<String>) -> Result<String> {
        match &script {
            Some(script) => {
                debug!("running {hook} hook: {}", script);
                let output = Command::new("sh")
                    .args(&["-c", script])
                    .env("CHIM_URL", &self.config.url)
                    .env("CHIM_BIN_PATH", &self.config.bin_path)
                    .stdin(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .wrap_err_with(|| format!("error executing {hook}: {}", script))?;

                let status = output.status;
                match status.success() {
                    true => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
                    false => Err(eyre!("{hook} failed with {status}")),
                }
            }
            None => {
                trace!("no {hook} hook specified");
                Ok(String::new())
            }
        }
    }
}
