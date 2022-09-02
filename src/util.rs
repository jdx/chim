use color_eyre::Result;
use std::path::Path;
use url::Url;

pub fn get_filename_from_url(url: &str) -> Result<String> {
    Ok(Url::parse(url)?
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("download.file")
        .to_string())
}

pub fn path_to_str(p: &Path) -> String {
    p.to_str().unwrap().to_string()
}
