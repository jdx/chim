use color_eyre::eyre::Result;
use color_eyre::Section;

mod app;
mod archive;
mod bin;
mod checksum;
mod chim_file;
mod cli;
mod config;
mod env;
mod fetchers;
mod hooks;
mod logger;
mod platform;

#[macro_use]
extern crate log;

#[tokio::main]
#[cfg(not(tarpaulin_include))]
pub async fn main() -> Result<()> {
    logger::init();
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();
    cli::parse(args).await.section("See https://chim.sh")
}
