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

#[cfg(test)]
#[cfg(feature = "test-e2e")]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    #[tokio::test]
    async fn test_app() {
        if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
            warn!("skipping test_app on linux-aarch64 since node binary does not work in alpine");
            return;
        }
        let dir = tempdir().unwrap();
        let chim_path = create_chim(&dir);
        run(args_to_str(vec!["node", chim_path.to_str().unwrap(), "-v"]))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_jq() {
        let chim_path = Path::new("example/jq");
        run(args_to_str(vec!["jq", chim_path.to_str().unwrap(), "-V"]))
            .await
            .unwrap();
    }

    fn create_chim(dir: &TempDir) -> PathBuf {
        let filename = dir.path().join("node");
        let mut file = File::create(&filename).unwrap();
        file.write_all(
            br#"#!/usr/bin/env chim

post_execute = 'echo post_execute' # disables execvp

[macos-arm64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-darwin-arm64.tar.gz'
path = 'node-v18.7.0-darwin-arm64/bin/node'
checksum = "sha256:ea24b35067bd0dc40ea8fda1087acc87672cbcbba881f7477dbd432e3c03343d"

[darwin-x86_64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-darwin-x64.tar.gz'
path = 'node-v18.7.0-darwin-x64/bin/node'
checksum = "sha256:ce95b924b450edbcfeaf422b3635a6b44b17ad23cd1f5efff6b051c60db548c8"

[linux-x64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-linux-x64.tar.xz'
path = 'node-v18.7.0-linux-x64/bin/node'
checksum = "sha256:8bc6a1b9deaed2586d726fc62d4bee9c1bfc5a30b96c1c4cff7edd15225a11a2"

[linux-arm64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-linux-arm64.tar.xz'
path = 'node-v18.7.0-linux-arm64/bin/node'
checksum = "sha256:c3ac4905ec3993d00a45d2c7af8417e79e907be51b8ffecb54c5b9ab8ef0bc9f"

[win-x64]
url = 'https://nodejs.org/dist/v18.7.0/node-v18.7.0-win-x64.zip'
path = 'node-v18.7.0-win-x64\node.exe'
checksum = "sha256:9c0abfe32291dd5bed717463cb3590004289f03ab66011e383daa0fcec674683"
"#,
        )
        .unwrap();

        filename
    }

    fn args_to_str(input: Vec<&str>) -> Vec<String> {
        input.into_iter().map(String::from).collect()
    }
}
