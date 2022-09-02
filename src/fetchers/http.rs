use crate::config::Config;
use crate::fetchers::Fetcher;
use async_trait::async_trait;
use color_eyre::Result;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

struct Http<'a> {
    #[allow(dead_code)]
    config: &'a Config,
}

pub fn new<'a>(config: &'a Config) -> Box<dyn Fetcher + 'a> {
    Box::new(Http { config })
}

#[async_trait]
impl<'a> Fetcher for Http<'a> {
    async fn fetch(&self, url: &str, output: &Path) -> Result<()> {
        let response = reqwest::get(url).await?;
        let mut file = File::create(output)?;
        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;

        Ok(())
    }
}
