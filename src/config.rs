use ini::Ini;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_open_api_key() -> String {
        let home = env::var("HOME").unwrap();
        let config_path = Path::new(&home).join("git_ai.ini");
        if let Ok(conf) = Ini::load_from_file(&config_path) {
            if let Some(api_key) = conf.get_from(None::<String>, "OPENAI_API_KEY") {
                return api_key.to_string();
            }
        }
        Self::execute_init();
        let conf = Ini::load_from_file(&config_path).unwrap();
        conf.get_from(None::<String>, "OPENAI_API_KEY")
            .unwrap()
            .to_string()
    }

    pub fn execute_init() {
        print!("Please enter your OpenAI API key: ");
        let _ = stdout().flush(); // Flushes the stdout buffer to ensure the printed text is displayed before reading input

        let mut api_key = String::new();
        let _ = stdin().read_line(&mut api_key);
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
}
