mod source;
mod subcommands {
    mod add;
    pub mod init;
}

use crate::subcommands::init;
use clap::{Parser, Subcommand};
use log::{error, info};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path and filename of the source file
    #[arg(long, default_value = "kunai.lock")]
    source_file: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize the source file
    Init,
    /// Add a new source
    Add {
        #[arg(short, long)]
        unpack: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => match init::init(cli.source_file.as_ref()) {
            Ok(()) => info!("Successfully created {}", cli.source_file),
            Err(init::InitError::LockfileExists) => error!("{} already exists", cli.source_file),
            Err(e) => error!("{}", e),
        },
        Command::Add { unpack: _ } => todo!(),
    }
}
