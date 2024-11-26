mod logging;
mod source;
mod subcommands {
    pub mod add;
    pub mod delete;
    pub mod init;
}

use crate::logging::{init_logger, LevelFilterArg};
use crate::source::SourceMapFromFileJsonError;
use crate::subcommands::add::AddError;
use crate::subcommands::delete::DeleteError;
use crate::subcommands::{add, delete, init};
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
    Add(add::AddArgs),
    /// Delete an existing source
    Delete {
        /// Name of the source to delete
        source_name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    init_logger(cli.log_level.into());

    match cli.command {
        Command::Init => match init::init(&cli.source_file) {
            Ok(()) => info!("successfully created {}", cli.source_file),
            Err(init::InitError::SourceFileExists) => error!("{} already exists", cli.source_file),
            Err(e) => error!("{e}"),
        },

        Command::Add(add_args) => match add::add(&cli.source_file, &add_args) {
            Ok(()) => info!("successfully added new source {}", add_args.source_name),
            Err(AddError::ReadSourceFromFileFailed(SourceMapFromFileJsonError::NotFound)) => {
                error!("source file not found at {}", cli.source_file)
            }
            Err(AddError::SourceNameAlreadyExists) => {
                error!("a source named \"{}\" already exists", add_args.source_name);
                error!("you may be trying to update, or if you want to override the source, delete it first");
            }
            Err(
                e @ AddError::ReadSourceFromFileFailed(
                    SourceMapFromFileJsonError::MalformedJson { .. }
                    | SourceMapFromFileJsonError::IncorrectSchema { .. },
                ),
            ) => {
                error!("{e}");
                error!("you may have to delete and remake the source file");
            }
            Err(e) => error!("{e}"),
        },

        Command::Delete { source_name } => match delete::delete(&cli.source_file, &source_name) {
            Ok(()) => info!("source \"{source_name}\" has been removed"),
            Err(DeleteError::ReadSourceFromFileFailed(SourceMapFromFileJsonError::NotFound)) => {
                error!("source file not found at {}", cli.source_file)
            }
            Err(
                e @ DeleteError::ReadSourceFromFileFailed(
                    SourceMapFromFileJsonError::MalformedJson { .. }
                    | SourceMapFromFileJsonError::IncorrectSchema { .. },
                ),
            ) => {
                error!("{e}");
                error!("you may have to delete and remake the source file");
            }
            Err(DeleteError::SourceNameNonexistent) => {
                error!("no source named \"{source_name}\" exists")
            }
            Err(e) => error!("{e}"),
        },
    }
}
