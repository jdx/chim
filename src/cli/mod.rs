mod usage;
mod version;

use crate::app;
use color_eyre::Result;

// #[derive(Parser)]
// #[clap(author, version, about, long_about=None, name="chim")]
// struct Cli {
//     #[clap(subcommand)]
//     command: Option<Commands>,
// }
//
// #[derive(Subcommand)]
// enum Commands {
//     Generate(generate::Generate),
// }
//
pub async fn parse(args: Vec<String>) -> Result<()> {
    match args.get(1) {
        Some(arg) => match arg.as_ref() {
            "-v" | "--version" | "version" => version::run(),
            arg if arg.starts_with('.') || arg.starts_with('/') => app::run(args).await,
            _ => usage::run(),
        },
        None => usage::run(),
    }
}
