mod cli;
mod commands;
mod config;

use crate::cli::Cli;
use crate::commands::GitAICommandExecutor;
use clap::Parser;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let cli = Cli::parse();

    match &cli.command {
        Some(command) => GitAICommandExecutor::execute_command(command).await?,
        None => println!("Default subcommand"),
    }

    Ok(())
}
