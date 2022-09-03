use color_eyre::{Result, Section};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ChimFile {
    #[serde(default)]
    pub debug: bool,

    #[serde(default)]
    pub quiet: bool,

    pub url: Option<String>,
    pub path: Option<String>,
    pub checksum: Option<String>,
    pub archive: Option<String>,
    pub execvp: Option<bool>,

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

    pub pre_execute_interval: Option<i64>,

    #[serde(flatten)]
    pub platforms: HashMap<String, Platform>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct Platform {
    pub url: Option<String>,
    pub path: Option<String>,
    pub checksum: Option<String>,
    pub archive: Option<String>,
    pub execvp: Option<bool>,

    // s3
    pub aws_profile: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub aws_access_token: Option<String>,
    pub aws_region: Option<String>,
}

pub fn from_str(s: String) -> Result<ChimFile> {
    let mut c: ChimFile = toml::from_str(&s).suggestion("Ensure chim is valid TOML.")?;
    c = normalize_platforms(c);
    //dbg!(&c);

    Ok(c)
}

pub fn from_file(filename: &str) -> Result<ChimFile> {
    from_str(
        fs::read_to_string(&filename)
            .with_suggestion(|| format!("ensure {filename} exists and can be read"))?,
    )
}

impl Platform {
    pub fn new() -> Platform {
        Platform {
            url: None,
            path: None,
            checksum: None,
            archive: None,
            execvp: None,

            aws_profile: None,
            aws_access_key_id: None,
            aws_secret_access_key: None,
            aws_access_token: None,
            aws_region: None,
        }
    }
}

fn split_platform_name(name: &str) -> (String, String) {
    let mut parts = name.split('-');
    return (
        normalize_os(parts.next().unwrap()).to_string(),
        normalize_arch(parts.next().unwrap()).to_string(),
    );
}

fn normalize_os(os: &str) -> &str {
    match os {
        "darwin" => "macos",
        "win" => "windows",
        _ => os,
    }
}

fn normalize_arch(arch: &str) -> &str {
    match arch {
        "arm64" => "aarch64",
        "x64" => "x86_64",
        _ => arch,
    }
}

fn normalize_platforms(mut cf: ChimFile) -> ChimFile {
    let keys = cf.platforms.keys().cloned().collect::<Vec<String>>();
    for k in keys {
        let (os, arch) = split_platform_name(&k.to_lowercase());
        let platform = cf.platforms.remove(&k).unwrap();
        cf.platforms.insert(format!("{os}-{arch}"), platform);
    }

    cf
}
