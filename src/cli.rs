use clap::{arg, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author="Anh Pham", version="0.0.1", about="Intelligent Git Generator", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Suggest a commit message
    Commit(CommitArgs),
    /// Suggest a PR template
    PR(PRArgs),
    /// Setup the tool
    Init,
}

#[derive(Args, Debug)]
pub struct CommitArgs {
    /// The context for the commit
    #[arg(short, long)]
    pub message: Option<String>,
    /// suggest commit by only the names of the changed files
    #[arg(short, long, default_value_t = false)]
    pub name_only: bool,
}

#[derive(Args, Debug)]
pub struct PRArgs {
    /// The context for the PR
    #[arg(short, long)]
    pub message: Option<String>,
    /// The branch to be PR'ed in
    #[arg(default_value_t=String::from("main"))]
    pub branch: String,
}
