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

const COMMIT_MESSAGE_TEMPLATE: &str = r#"
Commit Message Format
Format:
<type>: <subject>

<body>

<footer>
Type: This refers to the kind of change that you've made. Options include:
- feat: A new feature
- fix: A bug fix
- docs: Documentation only changes
- style: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc.)
- refactor: A code change that neither fixes a bug nor adds a feature
- perf: A code change that improves performance
- test: Adding missing or correcting existing tests
- chore: Changes to the build process or auxiliary tools and libraries such as documentation generation
Example:
feat: add login functionality

The new login functionality allows users to log in with their username and 
password. It includes input validation and error handling.

Related issues: #123, #456 or N/A
"#;

const PR_TEMPLATE: &str = r#"
PR Template:
Title: [Feature/Bugfix/Refactor]: Brief Description of Change

Description:
Please provide a detailed summary of the changes introduced in this PR. Include the context and motivation for the change.

Related Issue(s):
Please link to any related issues or tasks in your tracking system (e.g., JIRA ticket or GitHub issue).

Changes:
- Change 1
- Change 2
- Change 3

How to Test:
Provide instructions for how to test the changes, including any necessary setup and the expected outcome.

Dependencies: List any dependencies that must be resolved before/after merging this PR. Hide this section if no 
dependencies exist.
"#;

pub struct GitAICommandExecutor;

pub struct OpenAIHelper {
    client: Client<OpenAIConfig>,
}

impl OpenAIHelper {
    pub fn new(api_key: String) -> Self {
        let client = Client::with_config(OpenAIConfig::new().with_api_key(api_key));
        OpenAIHelper { client }
    }

    pub async fn generate_message(
        &self,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let message = ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content(content)
            .build()?;

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4")
            .messages([message])
            .build()?;

        let response = self.client.chat().create(request).await?;
        Ok(response
            .choices
            .first()
            .unwrap()
            .message
            .content
            .as_ref()
            .unwrap()
            .clone())
    }
}

impl GitAICommandExecutor {
    pub async fn execute_command(command: &Commands) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            Commands::Commit(args) => Self::execute_commit(args).await?,
            Commands::PR(args) => Self::execute_pr(args).await?,
            Commands::Init => Self::execute_init(),
        }
        Ok(())
    }

    async fn execute_commit(args: &CommitArgs) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("git");
        command.arg("diff");

        if args.name_only {
            command.arg("--name-only");
        }

        let output = command.output().expect("Failed to run git diff");
        let output = std::str::from_utf8(&output.stdout).unwrap().to_string();
        if output.trim().is_empty() {
            println!("Nothing to be committed");
        } else {
            let api_key = Self::get_open_api_key();
            let helper = OpenAIHelper::new(api_key);
            let message = format!(
                "Create me a commit message for these changes:\nThe context is: {}\n{}{}",
                args.message.as_ref().unwrap_or(&"".to_string()),
                output,
                COMMIT_MESSAGE_TEMPLATE
            );

            let response = helper.generate_message(&message).await?;
            let response = textwrap::wrap(&response, 72).join("\n");
            println!("{}", response);
        }
        Ok(())
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

    async fn execute_pr(args: &PRArgs) -> Result<(), Box<dyn std::error::Error>> {
        let branch = &args.branch;

        let output = Command::new("git")
            .arg("log")
            .arg(format!("{}..HEAD", branch))
            .output()
            .expect("Failed to execute git log command");
        let result = std::str::from_utf8(&output.stdout).unwrap().to_string();

        let mut message = format!(
            "Create me a PR for these changes:\nThe context is: {}\nAll the commit messages are:\n{}",  
            args.message.as_ref().unwrap_or(&"".to_string()), 
            result
        );

        let output = Command::new("git")
            .arg("diff")
            .arg(branch)
            .output()
            .expect("Failed to execute git diff command");
        let result = std::str::from_utf8(&output.stdout).unwrap().to_string();

        message += &format!("Changes are:\n{}{}", result, PR_TEMPLATE);

        let api_key = Self::get_open_api_key();
        let helper = OpenAIHelper::new(api_key);
        let response = helper.generate_message(&message).await?;
        let response = textwrap::wrap(&response, 72).join("\n");
        println!("{}", response);

        Ok(())
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
        Self::execute_init();

        // Then read the API key from the file again, as it should now exist
        let conf = Ini::load_from_file(&config_path).unwrap();
        conf.get_from(None::<String>, "OPENAI_API_KEY")
            .unwrap()
            .to_string()
    }
}
