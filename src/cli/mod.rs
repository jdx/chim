mod checksums;
mod run;
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
    if let Some(arg) = args.get(1) {
        match arg.as_ref() {
            arg if arg_is_path(arg) => return run::run(args).await,
            "-v" | "version" => return version::run(),
            _ => {}
        }
    };

    match Cli::parse_from(args).command.unwrap() {
        Commands::Checksums(args) => checksums::run(args).await,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version() {
        parse(vec!["chim".to_string(), "-v".to_string()])
            .await
            .unwrap();
    }
}
