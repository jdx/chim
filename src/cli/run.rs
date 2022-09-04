use crate::app::App;
use crate::config::{Config, Fetcher};
use color_eyre::eyre::Result;
use color_eyre::Section;
use std::env::consts::{ARCH, OS};
use std::path::Path;

pub async fn run(args: Vec<String>) -> Result<()> {
    let filename = Path::new(&args[1]);
    let config = Config::from_chim_file(filename, OS, ARCH)
        .with_section(|| format!("Chim: {}", filename.to_string_lossy()))?;
    debug!("config: {:#?}", config);

    let app = App::new(&config)?;
    if !config.bin_exists() {
        match config.fetcher {
            Fetcher::Local => {}
            _ => {
                let tmpdir = tempfile::tempdir()?;
                let archive = tmpdir.path().join("archive");
                app.fetch(&archive).await?;
                app.validate(&archive)?;
                app.extract(&archive)?;
            }
        }
    }
    app.exec(args)
}
