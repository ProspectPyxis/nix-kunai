mod logging;
mod source;
mod subcommands {
    pub mod add;
    pub mod delete;
    pub mod init;
    pub mod update;
}
mod updater;

use crate::logging::{init_logger, LevelFilterArg};
use crate::subcommands::{add, delete, init, update};
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
    ///
    /// Each updater scheme requires different possible arguments;
    /// see each corresponding help page for details.
    #[command(
        subcommand_value_name = "UPDATE_SCHEME",
        subcommand_help_heading = "Update schemes"
    )]
    Add(Box<add::AddArgs>),
    /// Update sources
    Update(update::UpdateArgs),
    /// Delete existing sources
    Delete {
        /// Name of sources to delete
        #[arg(required = true, value_name = "SOURCES")]
        source_names: Vec<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    init_logger(cli.log_level.into());

    match cli.command {
        Command::Init => init::init(&cli.source_file),
        Command::Add(args) => add::add(&cli.source_file, *args),
        Command::Update(args) => update::update(&cli.source_file, args),
        Command::Delete { source_names } => delete::delete(&cli.source_file, source_names),
    }
}
