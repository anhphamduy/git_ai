use crate::cli::{Commands, CommitArgs, PRArgs};
use std::process::Command;

pub fn execute_command(command: &Commands) {
    match command {
        Commands::Commit(args) => execute_commit(args),
        Commands::PR(args) => execute_pr(args),
    }
}

fn execute_commit(args: &CommitArgs) {
    let output = Command::new("git")
        .arg("diff")
        .output()
        .expect("Failed to run git diff");

    if let Ok(s) = std::str::from_utf8(&output.stdout) {
        if s.trim().is_empty() {
            println!("Nothing to be committed");
        } else {
            println!("As a string: {}", s);
        }
    }
    println!("commit was used, name is: {:?}", args.message);
}

fn execute_pr(args: &PRArgs) {
    let message = &args.message;
    let branch = &args.branch;
    println!("pr was used, name is: {:?}", message);
    println!("pr was used, name is: {:?}", branch);
}
