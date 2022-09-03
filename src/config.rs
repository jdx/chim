use crate::chim_file::ChimFile;
use crate::chim_file::Platform;
use crate::util::get_filename_from_url;
use crate::{env, util};
use color_eyre::eyre::{eyre, Result};
use color_eyre::Section;
use sha2::{Digest, Sha256};
use std::env::consts::{ARCH, OS};
use std::ffi::OsStr;
use std::path::Path;

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
    pub name: String,
    pub args: Vec<String>,

    pub fetcher: Fetcher,
    pub archive: Archive,
    pub url: String,
    pub checksum: Option<String>,
    pub execvp: bool,
    pub paranoid: bool,

    pub bin_path: String,
    pub cache_path: String,

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

pub fn from_chim_file(chim_file: ChimFile, args: Vec<String>) -> Result<Config> {
    let chim_path = Path::new(&args[1]);
    let name = osstr_to_str(chim_path.file_name().unwrap());
    let chim_dir = util::path_to_str(chim_path.parent().unwrap());

    let default_platform = Platform::new();
    let platform = chim_file
        .platforms
        .get(&format!("{}-{}", OS, ARCH))
        .or_else(|| match (OS, ARCH) {
            ("macos", "aarch64") => chim_file.platforms.get("macos-x86_64"),
            _ => None,
        })
        .unwrap_or(&default_platform);

    let url = get_url(&chim_file, platform);
    let archive = get_archive(&chim_file, platform, &url)?;
    let fetcher = get_fetcher(&url)?;
    let path = get_path(&chim_file, platform, &url, &archive)?;

    let cache_path = get_cache_path(&url)?;
    let bin_path = get_bin_path(&fetcher, &chim_dir, &cache_path, &path);

    Ok(Config {
        name,
        args,
        fetcher,
        archive,
        url,
        checksum: get_checksum(&chim_file, platform),
        bin_path,
        cache_path,
        execvp: get_execvp(&chim_file, platform),
        paranoid: get_paranoid(),

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

fn get_cache_root() -> Result<String> {
    match std::env::var("CHIM_CACHE_DIR") {
        Ok(v) => Ok(v),
        Err(_) => {
            let base = dirs::cache_dir().ok_or_else(|| eyre!("cache dir not found"))?;
            Ok(base.join("chim").into_os_string().into_string().unwrap())
        }
    }
}

fn str_to_sha256(s: &str) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(s.as_bytes());

    hex::encode(sha256.finalize())
}

fn osstr_to_str(s: &OsStr) -> String {
    s.to_str().unwrap().to_string()
}

fn get_url(chim_file: &ChimFile, platform: &Platform) -> String {
    platform
        .url
        .clone()
        .or_else(|| chim_file.url.clone())
        .unwrap_or_else(|| String::from("local:"))
}

fn get_cache_path(url: &str) -> Result<String> {
    let cache_root = get_cache_root()?;

    Ok(util::path_to_str(
        &Path::new(&cache_root).join(str_to_sha256(url)),
    ))
}

fn get_bin_path(fetcher: &Fetcher, chim_dir: &str, cache_path: &str, path: &str) -> String {
    let base = Path::new(match fetcher {
        Fetcher::Local => {
            if path.starts_with('/') {
                path
            } else {
                chim_dir
            }
        }
        _ => cache_path,
    });

    util::path_to_str(&base.join(path))
}

fn get_path(
    chim_file: &ChimFile,
    platform: &Platform,
    url: &str,
    archive: &Archive,
) -> Result<String> {
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
        .ok_or_else(|| {
            let url_or_path = if url == "local:" {
                "url or path"
            } else {
                "path"
            };

            eyre!("no {url_or_path} found for {OS}-{ARCH} platform")
                .suggestion(format!("add a {url_or_path} field to chim"))
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

fn get_paranoid() -> bool {
    env::var_is_true("CHIM_PARANOID")
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
