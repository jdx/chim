mod abs;
mod gcs;
mod http;
mod s3;
mod scp;

use crate::config;
use crate::config::Config;
use async_trait::async_trait;
use color_eyre::Result;
use std::path::Path;

pub fn new<'a>(config: &'a Config) -> Box<dyn Fetcher + 'a> {
    match config.fetcher {
        config::Fetcher::Http => http::new(config),
        config::Fetcher::S3 => s3::new(config),
        config::Fetcher::Gcs => gcs::new(config),
        config::Fetcher::Abs => abs::new(config),
        config::Fetcher::Scp => scp::new(config),
        _ => panic!("unsupported fetcher"),
    }
}

#[async_trait]
pub trait Fetcher {
    async fn fetch(&self, url: &str, tmpfile: &Path) -> Result<()>;
}
