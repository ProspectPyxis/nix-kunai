mod source;
mod subcommands {
    mod add;
    mod init;
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init,
    Add {
        #[arg(short, long)]
        unpack: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => todo!(),
        Command::Add { unpack: _ } => todo!(),
    }
}
