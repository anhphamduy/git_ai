use crate::cli::{Commands, CommitArgs, PRArgs};
use crate::config::ConfigManager;
use async_openai::types::ChatCompletionFunctionsArgs;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};
use serde_json::json;
use std::io::{self, Write};
use std::process::Command;
use crate::code_improvements::Improvements;

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
        let client = Client::with_config(
            OpenAIConfig::new().with_api_key(ConfigManager::get_open_api_key()),
        );

        let system_message = ChatCompletionRequestMessageArgs::default()
            .role(Role::System)
            .content(String::from(""))
            .build();
        let mut conversation = ChatConversation {
            client,
            messages: Vec::new(),
        };

        if let Ok(message) = system_message {
            conversation.messages.push(message);
        }

        conversation
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
            .functions([ChatCompletionFunctionsArgs::default()
                .name("suggest_code_improvements")
                .description("Get a list of improvements for the code changes")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "improvements": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "severity": {
                                        "type": "string",
                                        "enum": ["high", "low", "medium"]
                                    } ,
                                    "code": {
                                        "type": "string",
                                        "description": "the piece of code to be fixed",
                                    },
                                    "reason": {
                                        "type": "string",
                                        "description": "reason for such suggestion",
                                    }
                                }
                            },
                            "description": "A list of code improvements",
                        },
                    },
                    "required": ["improvements"],
                }))
                .build()?])
            .function_call("auto")
            .build()?;

        let response = self.client.chat().create(request).await?;

        if let Some(content) = &response.choices.first().unwrap().message.content {
            let message = ChatCompletionRequestMessageArgs::default()
                .role(Role::Assistant)
                .content(content.clone())
                .build()?;
            self.messages.push(message);

            Ok(content.clone())
        } else {
            if let Some(function_call) = &response.choices.first().unwrap().message.function_call {
                let name = &function_call.name;
                if *name == "suggest_code_improvements" {
                    let arguments = &function_call.arguments;

                    let mut data: Improvements = serde_json::from_str(arguments.as_str())
                        .expect("Error deserializing the JSON");
                    data.improvements.sort_by(|a, b| b.severity.cmp(&a.severity));
                    data.display();
                }
            }

            Ok(String::from("Hello"))
        }
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
