use crate::app;
use color_eyre::Result;
use std::process::exit;

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
            "-v" | "--version" | "version" => version(),
            _ => app::run(args).await,
        },
        None => usage(),
    }

    // let cli = Cli::parse();
    //
    // match &cli.command {
    //     Some(Commands::Generate(cmd)) => {
    //         cmd.run()?;
    //     }
    //     None => usage(),
    // }
}

fn usage() -> ! {
    println!("Usage: chim <command> [options]");
    println!("More info at https://chim.sh");
    exit(0);
}

fn version() -> ! {
    println!("chim {}", env!("CARGO_PKG_VERSION"));
    exit(0);
}
