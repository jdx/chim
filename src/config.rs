use crate::chim_file::ChimFile;
use crate::chim_file::Platform;
use crate::env;
use color_eyre::eyre::{eyre, Report, Result};
use color_eyre::Section;
use reqwest::Url;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Fetcher {
    Local,
    Http,
    S3,
    Gcs,
    Abs,
    Scp,
    // Git,
}

#[derive(Debug)]
pub enum Archive {
    TarGz,
    TarXz,
    TarBz2,
    Tar,
    Zip,
    Gz,
    Xz,
    Bz2,
    None,
}

#[derive(Debug)]
pub struct Config {
    pub chim_path: PathBuf,
    pub name: String,

    pub fetcher: Fetcher,
    pub archive: Archive,
    pub url: String,
    pub checksum: Option<String>,
    pub execvp: bool,
    pub paranoid: bool,
    pub quiet: bool,

    pub bin_path: PathBuf,
    pub cache_path: PathBuf,

    // s3
    pub aws_profile: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub aws_access_token: Option<String>,
    pub aws_region: Option<String>,

    // hooks
    pub pre_fetch: Option<String>,
    pub pre_extract: Option<String>,
    pub pre_execute: Option<String>,
    pub post_execute: Option<String>,
    //pub pre_execute_interval: Option<i64>,
}

impl Config {
    pub fn from_chim_file(chim_path: &Path, os: &str, arch: &str) -> Result<Config> {
        let chim_file = ChimFile::from_file(chim_path)?;
        let name = chim_path.file_name().unwrap().to_string_lossy().to_string();
        let chim_dir = chim_path.parent().unwrap();

        let default_platform = Platform::default();
        let platform = chim_file
            .platforms
            .get(&format!("{}-{}", os, arch))
            .or_else(|| match (os, arch) {
                ("macos", "aarch64") => chim_file.platforms.get("macos-x86_64"),
                _ => None,
            })
            .unwrap_or(&default_platform);

        let url = get_url(&chim_file, platform);
        let archive = get_archive(&chim_file, platform, &url)?;
        let fetcher = get_fetcher(&url)?;
        let path = get_path(&chim_file, platform, &url, &archive)
            .ok_or_else(|| show_no_url_or_path_error(&fetcher, os, arch))?;

        let cache_path = get_cache_path(&url)?;
        let bin_path = get_bin_path(&fetcher, chim_dir, &cache_path, &path);

        Ok(Config {
            chim_path: chim_path.to_path_buf(),
            name,
            fetcher,
            archive,
            url,
            checksum: get_checksum(&chim_file, platform),
            bin_path,
            cache_path,
            execvp: get_execvp(&chim_file, platform),
            paranoid: get_paranoid(),
            quiet: get_quiet(&chim_file),

            // s3
            aws_profile: get_aws_profile(&chim_file, platform),
            aws_access_key_id: get_aws_access_key_id(&chim_file, platform),
            aws_secret_access_key: get_aws_secret_access_key(&chim_file, platform),
            aws_access_token: get_aws_access_token(&chim_file, platform),
            aws_region: get_aws_region(&chim_file, platform),

            // hooks
            pre_fetch: chim_file.pre_fetch,
            pre_extract: chim_file.pre_extract,
            pre_execute: chim_file.pre_execute,
            post_execute: chim_file.post_execute,
            //pre_execute_interval: chim_file.pre_execute_interval,
        })
    }

    pub fn bin_exists(&self) -> bool {
        Path::exists(self.bin_path.as_path())
    }
}

/// sha256 encode a string as hex
fn str_to_sha256(s: &str) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(s.as_bytes());

    hex::encode(sha256.finalize())
}

fn get_url(chim_file: &ChimFile, platform: &Platform) -> String {
    platform
        .url
        .clone()
        .or_else(|| chim_file.url.clone())
        .unwrap_or_else(|| String::from("local:"))
}

fn get_cache_path(url: &str) -> Result<PathBuf> {
    Ok(get_cache_root()?.join(str_to_sha256(url)))
}

fn get_bin_path(fetcher: &Fetcher, chim_dir: &Path, cache_path: &Path, path: &str) -> PathBuf {
    match fetcher {
        Fetcher::Local => {
            if path.starts_with('/') {
                Path::new(path)
            } else {
                chim_dir
            }
        }
        _ => cache_path,
    }
    .join(path)
}

fn get_path(
    chim_file: &ChimFile,
    platform: &Platform,
    url: &str,
    archive: &Archive,
) -> Option<String> {
    platform
        .path
        .clone()
        .or_else(|| chim_file.path.clone())
        .or_else(|| match archive {
            Archive::Gz | Archive::Bz2 | Archive::Xz | Archive::None => {
                if url == "local:" {
                    return None;
                }
                Some(
                    get_filename_from_url(url)
                        .unwrap()
                        .trim_end_matches(".gz")
                        .trim_end_matches(".xz")
                        .trim_end_matches(".bz2")
                        .to_string(),
                )
            }
            _ => None,
        })
}

fn get_checksum(chim_file: &ChimFile, platform: &Platform) -> Option<String> {
    match platform
        .checksum
        .clone()
        .or_else(|| chim_file.checksum.clone())
    {
        Some(checksum) if !checksum.contains(':') => Some(format!("sha256:{}", checksum)),
        Some(checksum) => Some(checksum),
        None => None,
    }
}

fn get_fetcher(url: &str) -> Result<Fetcher> {
    match url.split(':').next().unwrap() {
        "local" => Ok(Fetcher::Local),
        "http" | "https" => Ok(Fetcher::Http),
        "s3" => Ok(Fetcher::S3),
        "gs" => Ok(Fetcher::Gcs),
        "abs" => Ok(Fetcher::Abs),
        "scp" => Ok(Fetcher::Scp),
        _ => Err(eyre!("unsupported url protocol: {}", url)),
    }
}

fn get_archive(chim_file: &ChimFile, platform: &Platform, url: &str) -> Result<Archive> {
    let archive = platform
        .archive
        .clone()
        .or_else(|| chim_file.archive.clone());
    match &archive {
        Some(archive) => match extension_to_archive(&format!(".{archive}")) {
            Archive::None => Err(eyre!("unsupported archive: {}", archive)),
            a => Ok(a),
        },
        None => {
            let filename = get_filename_from_url(url)?;
            Ok(extension_to_archive(&filename))
        }
    }
}

fn extension_to_archive(f: &str) -> Archive {
    match f {
        f if f.ends_with(".tar.xz") || f.ends_with(".txz") => Archive::TarXz,
        f if f.ends_with(".tar.gz") || f.ends_with(".tgz") => Archive::TarGz,
        f if f.ends_with(".tar.bz2") || f.ends_with(".tbz2") => Archive::TarBz2,
        f if f.ends_with(".tar") => Archive::Tar,
        f if f.ends_with(".zip") => Archive::Zip,
        f if f.ends_with(".xz") => Archive::Xz,
        f if f.ends_with(".gz") => Archive::Gz,
        f if f.ends_with(".bz2") => Archive::Bz2,
        _ => Archive::None,
    }
}

fn get_execvp(chim_file: &ChimFile, platform: &Platform) -> bool {
    if env::var_is_false("CHIM_EXECVP") || chim_file.post_execute.is_some() {
        return false;
    }
    match platform.execvp {
        Some(execvp) => execvp,
        None => chim_file.execvp.unwrap_or(true),
    }
}

fn get_aws_profile(chim_file: &ChimFile, platform: &Platform) -> Option<String> {
    match &platform.aws_profile {
        Some(aws_profile) => Some(aws_profile.clone()),
        None => chim_file.aws_profile.clone(),
    }
}

fn get_aws_access_key_id(chim_file: &ChimFile, platform: &Platform) -> Option<String> {
    match &platform.aws_access_key_id {
        Some(aws_access_key_id) => Some(aws_access_key_id.clone()),
        None => chim_file.aws_access_key_id.clone(),
    }
}

fn get_aws_secret_access_key(chim_file: &ChimFile, platform: &Platform) -> Option<String> {
    match &platform.aws_secret_access_key {
        Some(aws_secret_access_key) => Some(aws_secret_access_key.clone()),
        None => chim_file.aws_secret_access_key.clone(),
    }
}

fn get_aws_access_token(chim_file: &ChimFile, platform: &Platform) -> Option<String> {
    match &platform.aws_access_token {
        Some(aws_access_token) => Some(aws_access_token.to_string()),
        None => chim_file.aws_access_token.clone(),
    }
}

fn get_aws_region(chim_file: &ChimFile, platform: &Platform) -> Option<String> {
    match &platform.aws_region {
        Some(aws_region) => Some(aws_region.clone()),
        None => chim_file.aws_region.clone(),
    }
}

fn get_paranoid() -> bool {
    env::var_is_true("CHIM_PARANOID")
}

fn get_cache_root() -> Result<PathBuf> {
    match std::env::var("CHIM_CACHE_DIR") {
        Ok(v) => Ok(PathBuf::from(&v)),
        Err(_) => {
            let base = dirs::cache_dir().ok_or_else(|| eyre!("cache dir not found"))?;
            Ok(base.join("chim"))
        }
    }
}

fn show_no_url_or_path_error(fetcher: &Fetcher, os: &str, arch: &str) -> Report {
    let url_or_path = match fetcher {
        Fetcher::Local => "url or path",
        _ => "path",
    };

    eyre!("no {url_or_path} found for {os}-{arch} platform")
        .suggestion(format!("add a {url_or_path} field to chim"))
}

fn get_filename_from_url(url: &str) -> Result<String> {
    Ok(Url::parse(url)?
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("download.file")
        .to_string())
}

fn get_quiet(chim_file: &ChimFile) -> bool {
    env::var_is_true("CHIM_QUIET") || (chim_file.quiet && !env::var_is_false("CHIM_QUIET"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_node_config() {
        let chim_path = Path::new("example/node");
        let c = Config::from_chim_file(chim_path, "macos", "aarch64").unwrap();

        assert_eq!(
            c.url,
            "https://nodejs.org/dist/v18.7.0/node-v18.7.0-darwin-arm64.tar.gz"
        );
    }
}
