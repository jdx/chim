use crate::archive;
use crate::checksum;
use crate::config::Config;
use crate::hooks::Hooks;
use crate::{bin, fetchers};
use color_eyre::eyre::{eyre, Result, WrapErr};
use color_eyre::Section;
use std::path::Path;

pub struct App<'a> {
    config: &'a Config,
    hooks: Hooks<'a>,
}

impl<'a> App<'a> {
    pub fn new(config: &'a Config) -> Result<App<'a>> {
        Ok(App {
            config,
            hooks: Hooks::new(config),
        })
    }

    pub fn validate(&self, filename: &Path) -> Result<()> {
        let checksum = &self.config.checksum;
        match checksum {
            Some(checksum) => {
                debug!("validating checksum for {:?}", filename);
                checksum::validate(filename, checksum)
                    .with_section(|| format!("URL: {}", self.config.url))
                    .with_suggestion(|| {
                        format!(
                            "ensure that checksum is valid in chim {}",
                            self.config.chim_path.display(),
                        )
                    })?;
                debug!("checksum is valid");
                Ok(())
            }
            None if self.config.paranoid => Err(eyre!("checksum is required in paranoid mode")),
            None => {
                info!("no checksum specified for {:?}", filename);
                Ok(())
            }
        }
    }

    pub async fn fetch(&self, output: &Path) -> Result<()> {
        let url = match self.hooks.pre_fetch()? {
            url if url.is_empty() => self.config.url.clone(),
            url => url,
        };
        debug!("fetching {}", url);

        fetchers::new(self.config)
            .fetch(&url, output)
            .await
            .wrap_err_with(|| format!("error fetching {}", url))?;

        Ok(())
    }

    pub fn extract(&self, filename: &Path) -> Result<()> {
        let dest = &self.config.cache_path;
        debug!("extracting archive {:?} to {:?}", filename, dest);
        self.hooks.pre_extract()?;
        archive::extract(filename, dest, &self.config.bin_path, &self.config.archive)?;

        Ok(())
    }

    pub fn exec(&self, args: Vec<String>) -> Result<()> {
        let bin = &self.config.bin_path;
        let execvp = self.config.execvp;

        debug!("executing platform binary {:?}", bin);
        self.hooks.pre_execute()?;
        bin::new(bin.as_os_str())
            .args(args.iter().skip(2).map(|s| s.as_ref()))
            .execvp(execvp)
            .exec()?;
        self.hooks.post_execute()?;

        Ok(())
    }
}
