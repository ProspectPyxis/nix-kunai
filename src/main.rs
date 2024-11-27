mod logging;
mod source;
mod subcommands {
    pub mod add;
    pub mod delete;
    pub mod init;
}

use crate::logging::{init_logger, LevelFilterArg};
use crate::subcommands::{add, delete, init};
use clap::{Parser, Subcommand};
use std::process::ExitCode;

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
    Add(add::AddArgs),
    /// Delete an existing source
    Delete {
        /// Name of the source to delete
        source_name: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    init_logger(cli.log_level.into());

    match cli.command {
        Command::Init => init::init(&cli.source_file),
        Command::Add(args) => add::add(&cli.source_file, args),
        Command::Delete { source_name } => delete::delete(&cli.source_file, &source_name),
    }
}
