mod abs;
mod gcs;
mod http;
mod s3;
mod scp;

use crate::config;
use crate::config::Config;
use color_eyre::Result;
use std::path::Path;

pub struct Fetcher<'a> {
    config: &'a Config,
}

pub fn new(config: &Config) -> Fetcher {
    Fetcher { config }
}

impl<'a> Fetcher<'a> {
    pub async fn fetch(&self, url: &str, tmpfile: &Path) -> Result<()> {
        match self.config.fetcher {
            config::Fetcher::Http => http::fetch(url, tmpfile).await,
            config::Fetcher::S3 => s3::fetch(self.config, url, tmpfile),
            config::Fetcher::Gcs => gcs::fetch(url, tmpfile),
            config::Fetcher::Abs => abs::fetch(url, tmpfile),
            config::Fetcher::Scp => scp::fetch(url, tmpfile),
            _ => panic!("unsupported fetcher"),
        }
    }
}
