use dirs;
use failure::err_msg;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{stdin, Write},
    path::{Path, PathBuf},
};

const FILE_NAME: &str = "client.yml";
const CONFIG_DIR: &str = ".config";
const APP_CONFIG_DIR: &str = "barnacle";

pub const BANNER: &str = r#"
888                                            888         
888                                            888         
888                                            888         
88888b.  8888b. 888d88888888b.  8888b.  .d8888b888 .d88b.  
888 "88b    "88b888P"  888 "88b    "88bd88P"   888d8P  Y8b 
888  888.d888888888    888  888.d888888888     88888888888 
888 d88P888  888888    888  888888  888Y88b.   888Y8b.     
88888P" "Y888888888    888  888"Y888888 "Y8888P888 "Y8888 
"#;

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
}

pub struct ConfigPaths {
    pub config_file_path: PathBuf,
}

impl ClientConfig {
    pub fn new() -> ClientConfig {
        ClientConfig {
            client_id: "".to_string(),
            client_secret: "".to_string(),
        }
    }

    pub fn get_or_build_paths(&self) -> Result<ConfigPaths, failure::Error> {
        match dirs::home_dir() {
            Some(home) => {
                let path = Path::new(&home);
                let home_config_dir = path.join(CONFIG_DIR);
                let app_config_dir = home_config_dir.join(APP_CONFIG_DIR);

                if !home_config_dir.exists() {
                    fs::create_dir(&home_config_dir)?;
                }

                if !app_config_dir.exists() {
                    fs::create_dir(&app_config_dir)?;
                }

                let config_file_path = &app_config_dir.join(FILE_NAME);

                let paths = ConfigPaths {
                    config_file_path: config_file_path.to_path_buf(),
                };

                Ok(paths)
            }
            None => Err(err_msg("No $HOME directory found for client config")),
        }
    }

    pub fn load_config(&mut self) -> Result<(), failure::Error> {
        let paths = self.get_or_build_paths()?;
        if paths.config_file_path.exists() {
            let config_string = fs::read_to_string(&paths.config_file_path)?;
            let config_yml: ClientConfig = serde_yaml::from_str(&config_string)?;

            self.client_id = config_yml.client_id;
            self.client_secret = config_yml.client_secret;

            Ok(())
        } else {
            println!("{}", BANNER);

            println!(
                "Config will be saved to {}",
                paths.config_file_path.display()
            );

            println!("\nHow to get setup:\n");

            let instructions = [
                "Go to the Github dashboard - https://github.com/settings/tokens",
                "Click `Generate New Token` and select Gist",
                "Copy Token and paste at prompt`",
                "You are now ready to authenticate with Gist!",
            ];

            let mut number = 1;
            for item in instructions.iter() {
                println!("  {}. {}", number, item);
                number += 1;
            }

            let mut client_secret = String::new();
            println!("\nEnter your Github Personal Token: ");
            stdin().read_line(&mut client_secret)?;

            let mut client_id = String::new();
            println!("\nEnter your Gist id: ");
            stdin().read_line(&mut client_id)?;

            let config_yml = ClientConfig {
                client_id: client_id.trim().to_string(),
                client_secret: client_secret.trim().to_string(),
            };

            let content_yml = serde_yaml::to_string(&config_yml)?;

            let mut new_config = fs::File::create(&paths.config_file_path)?;
            write!(new_config, "{}", content_yml)?;

            self.client_id = config_yml.client_id;
            self.client_secret = config_yml.client_secret;

            Ok(())
        }
    }
}
