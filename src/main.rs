mod logging;
mod source;
mod subcommands {
    pub mod add;
    pub mod delete;
    pub mod edit;
    pub mod init;
    pub mod update;
}
mod updater;

use crate::logging::{init_logger, LevelFilterArg};
use crate::subcommands::{add, delete, edit, init, update};
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
    /// Update sources
    Update(update::UpdateArgs),
    /// Edit a key for an existing source
    Edit {
        #[arg(value_name = "SOURCE")]
        source_name: String,
        key: edit::EditableSourceKey,
        value: String,
    },
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
        Command::Add(args) => add::add(&cli.source_file, args),
        Command::Update(args) => update::update(&cli.source_file, args),
        Command::Edit {
            source_name,
            key,
            value,
        } => edit::edit(&cli.source_file, &source_name, key, &value),
        Command::Delete { source_names } => delete::delete(&cli.source_file, source_names),
    }
}
