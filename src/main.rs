mod logging;
mod source;
mod subcommands {
    mod add;
    pub mod init;
}

use crate::logging::{init_logger, LevelFilterArg};
use crate::subcommands::init;
use clap::{Parser, Subcommand};
use log::{error, info};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path and filename of the source file
    #[arg(long, default_value = "kunai.lock")]
    source_file: String,
    /// Logging level to print
    #[arg(long, value_enum, default_value_t = LevelFilterArg::Info)]
    log_level: LevelFilterArg,
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

    init_logger(cli.log_level.into());

    match cli.command {
        Command::Init => match init::init(cli.source_file.as_ref()) {
            Ok(()) => info!("Successfully created {}", cli.source_file),
            Err(init::InitError::SourceFileExists) => error!("{} already exists", cli.source_file),
            Err(e) => error!("{}", e),
        },
        Command::Add { unpack: _ } => todo!(),
    }
}
