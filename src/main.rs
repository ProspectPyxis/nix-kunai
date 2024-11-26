mod source;
mod subcommands {
    mod add;
    pub mod init;
}

use crate::subcommands::init;
use clap::{Parser, Subcommand};
use env_logger::fmt::style as anstyle;
use log::{error, info};
use log::{Level, LevelFilter};
use std::io::Write;

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

    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            let log_style = anstyle::Style::new().bold();
            let log_style = log_style.fg_color(Some(
                (match record.level() {
                    Level::Trace => anstyle::AnsiColor::Magenta,
                    Level::Debug => anstyle::AnsiColor::Blue,
                    Level::Info => anstyle::AnsiColor::Green,
                    Level::Warn => anstyle::AnsiColor::Yellow,
                    Level::Error => anstyle::AnsiColor::Red,
                })
                .into(),
            ));

            writeln!(
                buf,
                "{} {log_style}{:5}{log_style:#} {}",
                buf.timestamp_seconds(),
                record.level(),
                record.args()
            )
        })
        .init();

    match cli.command {
        Command::Init => match init::init(cli.source_file.as_ref()) {
            Ok(()) => info!("Successfully created {}", cli.source_file),
            Err(init::InitError::LockfileExists) => error!("{} already exists", cli.source_file),
            Err(e) => error!("{}", e),
        },
        Command::Add { unpack: _ } => todo!(),
    }
}
