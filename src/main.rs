mod cli;
mod commands;

use crate::cli::Cli;
use crate::commands::execute_command;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(command) => execute_command(command),
        None => println!("Default subcommand"),
    }
}
