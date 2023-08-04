use clap::{arg, Args, Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(author="Anh Pham", version="0.0.1", about="Intelligent Git Generator", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Suggest a commit message
    Commit(CommitArgs),
    /// Suggest a PR template
    PR(PRArgs),
}

#[derive(Args, Debug)]
struct CommitArgs {
    /// The context for the commit
    #[arg(short, long)]
    message: Option<String>,
    /// suggest commit by only the names of the changed files
    #[arg(short, long, default_value_t = false)]
    name_only: bool,
}

#[derive(Args, Debug)]
struct PRArgs {
    /// The context for the PR
    #[arg(short, long)]
    message: Option<String>,
    /// The branch to be PR'ed in
    #[arg(default_value_t=String::from("main"))]
    branch: String,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Commit(args)) => {
            let output = Command::new("git")
                .arg("diff")
                .output()
                .expect("Failed to run git diff");

            if let Ok(s) = std::str::from_utf8(&output.stdout) {
                println!("As a string: {}", s);
            }
            println!("commit was used, name is: {:?}", args.message)
        }
        Some(Commands::PR(args)) => {
            let message = &args.message;
            let branch = &args.branch;
            println!("pr was used, name is: {:?}", message);
            println!("pr was used, name is: {:?}", branch)
        }
        None => {
            println!("Default subcommand");
        }
    }
}
