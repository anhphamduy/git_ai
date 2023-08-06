use ini::Ini;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

pub struct ConfigManager;

impl ConfigManager {
    pub fn init() {
        print!("Please enter your OpenAI API key: ");

        let mut api_key = String::new();
        stdout().flush().unwrap();
        let _ = stdin().read_line(&mut api_key);
        api_key = api_key.trim().to_string();

        // Creating an INI file and setting the API key
        let mut conf = Ini::new();
        conf.with_section(None::<String>)
            .set("OPENAI_API_KEY", api_key);

        // Writing the INI file to disk
        let config_path = Self::get_config_path();
        conf.write_to_file(&config_path).unwrap();
    }

    pub fn get_open_api_key() -> String {
        let config_path = Self::get_config_path();
        if let Ok(conf) = Ini::load_from_file(&config_path) {
            if let Some(api_key) = conf.get_from(None::<String>, "OPENAI_API_KEY") {
                return api_key.to_string();
            }
        }
        Self::init();
        let conf = Ini::load_from_file(&config_path).unwrap();
        conf.get_from(None::<String>, "OPENAI_API_KEY")
            .unwrap()
            .to_string()
    }

    fn get_config_path() -> PathBuf {
        let home = env::var("HOME").unwrap();
        PathBuf::from(&home).join("git_ai.ini")
    }
}
