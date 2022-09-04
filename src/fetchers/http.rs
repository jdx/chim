use color_eyre::Result;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

pub async fn fetch(url: &str, output: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = File::create(output)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;

    Ok(())
}
