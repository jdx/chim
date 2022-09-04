use crate::checksum::get_checksum;
use crate::config::Config;
use crate::fetchers;
use crate::platform::split_platform_name;
use color_eyre::eyre::{eyre, Result};
use color_eyre::Report;
use sha2::Sha256;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use toml_edit::{value, Document};

pub async fn run(args: Vec<String>) -> Result<()> {
    let filename = Path::new(args.get(2).ok_or_else(usage)?);
    let mut doc = read(filename)?;
    trace!("{}", doc.to_string());

    doc["macos-arm64"]["checksum"] = value("1234");
    for (platform, values) in doc.iter_mut() {
        values["checksum"] = value(fetch_checksum(filename, &platform).await?);
    }

    debug!("{}", doc.to_string());
    write(filename, doc)?;

    info!("updated checksums in {:?}", filename);
    Ok(())
}

fn read(filename: &Path) -> Result<Document> {
    trace!("reading {:?}", filename);
    let toml = fs::read_to_string(filename)?;
    let doc = toml.parse::<Document>()?;

    Ok(doc)
}

fn write(filename: &Path, doc: Document) -> Result<()> {
    trace!("writing {:?}", filename);
    fs::write(filename, doc.to_string())?;

    Ok(())
}

fn usage() -> Report {
    eyre!("usage: chim checksum <filename>")
}

async fn fetch_checksum(filename: &Path, platform: &str) -> Result<String> {
    let (os, arch) = split_platform_name(platform);
    let config = Config::from_chim_file(filename, &os, &arch)?;
    let tmpdir = tempdir()?;
    let tmpfile = tmpdir.path().join("archive");

    info!("fetching checksum for {}", config.url);
    fetchers::new(&config).fetch(&config.url, &tmpfile).await?;

    let checksum = format!("sha256:{}", get_checksum::<Sha256>(&tmpfile)?);
    info!("checksum: {}", checksum);

    Ok(checksum)
}
