use crate::checksum::get_checksum;
use crate::config::Config;
use crate::fetchers;
use crate::platform::split_platform_name;
use color_eyre::eyre::Result;
use sha2::Sha256;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use toml_edit::{value, Document};

#[derive(Debug, clap::Args)]
#[clap(about = "Regenerates all checksums in a chim")]
pub struct Args {
    #[clap(help = "The path to the chim file to update")]
    chim_file: PathBuf,
}

pub async fn run(args: Args) -> Result<()> {
    let filename = &args.chim_file;
    let mut doc = read(filename)?;
    trace!("{}", doc.to_string());

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    #[tokio::test]
    async fn test_checksums() {
        let dir = tempdir().unwrap();
        let chim_path = create_chim(&dir);
        run(Args {
            chim_file: chim_path.clone(),
        })
        .await
        .unwrap();

        assert_eq!(
            fs::read_to_string(&chim_path).unwrap(),
            r#"#!/usr/bin/env chim
[macos-arm64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-darwin-arm64.tar.gz'
path = 'node-v18.7.0-darwin-arm64/bin/node'
checksum = "sha256:ea24b35067bd0dc40ea8fda1087acc87672cbcbba881f7477dbd432e3c03343d"

[win-x64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-win-x64.zip'
path = 'node-v18.7.0-win-x64\node.exe'
checksum = "sha256:9c0abfe32291dd5bed717463cb3590004289f03ab66011e383daa0fcec674683"
"#
        );
    }

    fn create_chim(dir: &TempDir) -> PathBuf {
        let filename = dir.path().join("node");
        let mut file = File::create(&filename).unwrap();
        file.write_all(
            br#"#!/usr/bin/env chim
[macos-arm64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-darwin-arm64.tar.gz'
path = 'node-v18.7.0-darwin-arm64/bin/node'

[win-x64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-win-x64.zip'
path = 'node-v18.7.0-win-x64\node.exe'
"#,
        )
        .unwrap();

        filename
    }
}
