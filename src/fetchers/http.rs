use crate::config::Config;
use color_eyre::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Response;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub async fn fetch(config: &Config, url: &str, output: &Path) -> Result<()> {
    let mut response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let mut file = File::create(output)?;
    let pb = get_content_length(&response)
        .map(|l| create_progress_bar(config, l))
        .unwrap_or_else(ProgressBar::hidden);

    while let Some(chunk) = response.chunk().await? {
        pb.inc(chunk.len() as u64);
        file.write_all(&chunk)?;
    }

    Ok(())
}

fn get_content_length(response: &Response) -> Option<u64> {
    response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
}

fn create_progress_bar(config: &Config, length: u64) -> ProgressBar {
    if config.quiet {
        return ProgressBar::hidden();
    }
    let pb = ProgressBar::new(length);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("=>-"));
    //.with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())

    pb
}
