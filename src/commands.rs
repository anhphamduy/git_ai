use crate::cli::{Commands, CommitArgs, PRArgs};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};
use std::io::{self, Write};
use std::process::Command;
use crate::config::ConfigManager; 

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

pub struct ChatConversation {
    client: Client<OpenAIConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
}

impl ChatConversation {
    pub fn new() -> Self {
        let client = Client::with_config(OpenAIConfig::new().with_api_key(ConfigManager::get_open_api_key()));
        ChatConversation {
            client,
            messages: Vec::new(),
        }
    }

    pub async fn generate_message(
        &mut self,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let message = ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content(content)
            .build()?;
        self.messages.push(message);

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4")
            .messages(&*self.messages)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let response = response
            .choices
            .first()
            .unwrap()
            .message
            .content
            .as_ref()
            .unwrap()
            .clone();

        let message = ChatCompletionRequestMessageArgs::default()
            .role(Role::Assistant)
            .content(content.clone())
            .build()?;
        self.messages.push(message);

        Ok(response)
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
        let mut output = std::str::from_utf8(&output.stdout).unwrap().to_string();
        if output.trim().is_empty() {
            // Re-run the command with the `--staged` option if the output is empty
            command.arg("--staged");
            let output_staged = command.output().expect("Failed to run git diff --staged");
            output = std::str::from_utf8(&output_staged.stdout)
                .unwrap()
                .to_string();
        }

        if output.trim().is_empty() {
            println!("Nothing to be committed");
        } else {
            let mut conversation = ChatConversation::new();
            let mut message = format!(
                "Create me a commit message for these changes:\nThe context is: {}\n{}{}",
                args.message.as_ref().unwrap_or(&"".to_string()),
                output,
                COMMIT_MESSAGE_TEMPLATE
            );

            loop {
                let response = conversation.generate_message(&message).await?;
                let response = textwrap::wrap(&response, 72).join("\n");
                println!("{}", response);

                // Prompt the user for another input
                print!("Anything you want to edit? ");
                io::stdout().flush().unwrap(); // Ensure the prompt is immediately visible
                message.clear();
                io::stdin().read_line(&mut message).unwrap();

                println!("You entered: {}", message.trim());
            }
        }
        Ok(())
    }

    fn execute_init() {
        ConfigManager::init();
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

        let mut conversation = ChatConversation::new();
        let response = conversation.generate_message(&message).await?;
        let response = textwrap::wrap(&response, 72).join("\n");
        println!("{}", response);

        Ok(())
    }

}
