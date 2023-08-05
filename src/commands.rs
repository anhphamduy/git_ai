use crate::cli::{Commands, CommitArgs, PRArgs};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use ini::Ini;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub async fn execute_command(command: &Commands) {
    match command {
        Commands::Commit(args) => execute_commit(args).await,
        Commands::PR(args) => execute_pr(args),
        Commands::Init => execute_init(),
    }
}

fn execute_init() {
    print!("Please enter your OpenAI API key: ");
    let _ = io::stdout().flush(); // Flushes the stdout buffer to ensure the printed text is displayed before reading input

    let mut api_key = String::new();
    let _ = io::stdin().read_line(&mut api_key);
    api_key = api_key.trim().to_string(); // Removes any trailing newline characters

    // Getting the home directory
    let home = env::var("HOME").unwrap();

    // Constructing the path to the config file
    let config_path = Path::new(&home).join("git_ai.ini");

    // Creating an INI file and setting the API key
    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("OPENAI_API_KEY", api_key);

    // Writing the INI file to disk
    conf.write_to_file(&config_path).unwrap();

    println!("{} has been updated", config_path.display());
}

fn get_open_api_key() -> String {
    // Getting the home directory
    let home = env::var("HOME").unwrap();

    // Constructing the path to the config file
    let config_path = Path::new(&home).join("git_ai.ini");

    // Trying to load the INI file
    if let Ok(conf) = Ini::load_from_file(&config_path) {
        if let Some(api_key) = conf.get_from(None::<String>, "OPENAI_API_KEY") {
            return api_key.to_string();
        }
    }

    // If the code reaches here, the API key was not found, so call execute_init
    execute_init();

    // Then read the API key from the file again, as it should now exist
    let conf = Ini::load_from_file(&config_path).unwrap();
    conf.get_from(None::<String>, "OPENAI_API_KEY")
        .unwrap()
        .to_string()
}

async fn execute_commit(_: &CommitArgs) {
    let output = Command::new("git")
        .arg("diff")
        .output()
        .expect("Failed to run git diff");

    if let Ok(s) = std::str::from_utf8(&output.stdout) {
        if s.trim().is_empty() {
            println!("Nothing to be committed");
        } else {
            let api_key = get_open_api_key();
            let client = Client::with_config(OpenAIConfig::new().with_api_key(api_key));

            let message = ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content("Where was it played?")
                .build();
            match message {
                Ok(content) => {
                    let request = CreateChatCompletionRequestArgs::default()
                        .model("gpt-4")
                        .messages([content])
                        .build()
                        .unwrap();

                    let response = client
                        .chat() // Get the API "group" (completions, images, etc.) from the client
                        .create(request) // Make the API call in that "group"
                        .await
                        .unwrap();

                    println!(
                        "{}",
                        response
                            .choices
                            .first()
                            .unwrap()
                            .message
                            .content
                            .as_ref()
                            .unwrap()
                    );
                }
                Err(_) => todo!(),
            };
        }
    }
}

fn execute_pr(args: &PRArgs) {
    let message = &args.message;
    let branch = &args.branch;
    println!("pr was used, name is: {:?}", message);
    println!("pr was used, name is: {:?}", branch);
}
