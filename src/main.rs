mod cli;
mod commands;

use crate::cli::Cli;
use crate::commands::execute_command;
use clap::Parser;
use tokio;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(command) => execute_command(command).await,
        None => println!("Default subcommand"),
    }
}
