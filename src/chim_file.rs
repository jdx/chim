use crate::platform::split_platform_name;
use color_eyre::eyre::Context;
use color_eyre::{Result, Section};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
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

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Default)]
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

impl ChimFile {
    pub fn from_file(filename: &Path) -> Result<ChimFile> {
        let body =
            fs::read_to_string(filename).suggestion("ensure file exists and can be read")?;
        ChimFile::from_str(body).wrap_err("error parsing toml")
    }

    fn from_str(s: String) -> Result<ChimFile> {
        let mut c: ChimFile = toml::from_str(&s).suggestion("Ensure chim is valid TOML.")?;
        c.normalize_platforms();

        Ok(c)
    }

    fn normalize_platforms(&mut self) {
        let keys = self.platforms.keys().cloned().collect::<Vec<String>>();
        for k in keys {
            let (os, arch) = split_platform_name(&k.to_lowercase());
            let platform = self.platforms.remove(&k).unwrap();
            self.platforms.insert(format!("{os}-{arch}"), platform);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_normalize_platforms() {
        let mut cf = ChimFile {
            platforms: vec![("darwin-arm64".to_string(), Platform::default())]
                .into_iter()
                .collect(),
            ..ChimFile::default()
        };

        cf.normalize_platforms();

        assert_eq!(cf.platforms.len(), 1);
        assert!(cf.platforms.contains_key("macos-aarch64"));
    }
}
