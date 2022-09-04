mod checksums;
mod run;
mod usage;
mod version;

use clap::{AppSettings, Parser, Subcommand};
use color_eyre::Result;
use std::path::Path;

#[derive(Debug, Parser)]
#[clap(version, about, long_about=None, name="chim", help_expected=true, 
    setting=AppSettings::SubcommandRequiredElseHelp,
)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Checksums(checksums::Args),
}

pub async fn parse(args: Vec<String>) -> Result<()> {
    match args.get(1) {
        Some(arg) => match arg.as_ref() {
            arg if arg_is_path(arg) => return run::run(args).await,
            "-v" | "version" => return version::run(),
            _ => {}
        },
        None => {}
    };

    match Cli::parse_from(args).command {
        Some(Commands::Checksums(args)) => checksums::run(args).await,
        None => usage::run(),
    }
}

/// detects if we should attempt to run a chim or not
/// this should be true if called from a shebang or .bat file
fn arg_is_path(arg: &str) -> bool {
    arg.starts_with('.')
        || arg.starts_with('/')
        || arg.starts_with('~')
        || Path::new(arg).is_absolute()
}
