use crate::archive;
use crate::checksum;
use crate::chim_file;
use crate::config;
use crate::config::Fetcher;
use crate::hooks::Hooks;
use crate::util::path_to_str;
use crate::{bin, fetchers};
use color_eyre::eyre::{eyre, Result, WrapErr};
use color_eyre::Section;
use std::path::Path;

pub async fn run(args: Vec<String>) -> Result<()> {
    let filename = args[1].to_string();
    let config = App::get_config(args).with_section(|| format!("Chim: {filename}"))?;
    debug!("config: {:#?}", config);

    let app = App::new(&config)?;
    if !app.exists() {
        app.install().await?;
    }
    app.exec()
}

struct App<'a> {
    config: &'a config::Config,
    hooks: Hooks<'a>,
}

impl<'a> App<'a> {
    fn new(config: &'a config::Config) -> Result<App<'a>> {
        Ok(App {
            config,
            hooks: Hooks::new(config),
        })
    }

    fn get_config(args: Vec<String>) -> Result<config::Config> {
        let chim_filename = &args[1].to_string();
        let chim_file = chim_file::from_file(chim_filename)
            .wrap_err_with(|| format!("error parsing chim {chim_filename}"))?;

        config::from_chim_file(chim_file, args)
            .wrap_err_with(|| format!("error building config from {chim_filename}"))
    }

    async fn install(&self) -> Result<()> {
        match self.config.fetcher {
            Fetcher::Local => {}
            _ => {
                let tmpdir = tempfile::tempdir()?;
                let archive = tmpdir.path().join("archive");
                self.fetch(&archive).await?;
                self.validate(&archive)?;
                self.extract(&archive)?;
            }
        }

        Ok(())
    }

    fn validate(&self, filename: &Path) -> Result<()> {
        let checksum = &self.config.checksum;
        match checksum {
            Some(checksum) => {
                debug!("validating checksum for {}", path_to_str(filename));
                checksum::validate(filename, checksum)?;
                debug!("checksum is valid");
                Ok(())
            }
            None if self.config.paranoid => Err(eyre!("checksum is required in paranoid mode")),
            None => {
                info!("no checksum specified for {}", path_to_str(filename));
                Ok(())
            }
        }
    }

    async fn fetch(&self, output: &Path) -> Result<()> {
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

    fn extract(&self, filename: &Path) -> Result<()> {
        let dest = &self.config.cache_path;
        debug!("extracting archive {} to {}", path_to_str(filename), dest);
        self.hooks.pre_extract()?;
        archive::extract(filename, dest, &self.config.bin_path, &self.config.archive)?;

        Ok(())
    }

    fn exists(&self) -> bool {
        let bin = &self.config.bin_path;

        Path::exists(Path::new(bin))
    }

    fn exec(&self) -> Result<()> {
        let bin = &self.config.bin_path;
        let args = &self.config.args;
        let execvp = self.config.execvp;

        debug!("executing platform binary {}", bin);
        self.hooks.pre_execute()?;
        bin::exec(bin, args, execvp)?;
        self.hooks.post_execute()?;

        Ok(())
    }
}
