use std::fs::{self, File};
use std::io::{Read, Result, Write};
use std::path::Path;

use dirs::config_dir;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Configuration {
    pub general: General,
    pub administrators: Administrators,
    pub message_responses: Vec<MessageResponse>,
}

const CONFIG_PATH: &str = "configuration.json";

impl Configuration {
    fn save(&self) -> Result<()> {
        let sys_config_dir = config_dir().expect("find config dir");

        fs::create_dir_all(format!(
            "{}/revanced-discord-bot",
            sys_config_dir.to_string_lossy()
        ))
        .expect("create config dir");

        let mut file = File::create(CONFIG_PATH)?;
        let json = serde_json::to_string_pretty(&self)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load() -> Result<Configuration> {
        let sys_config_dir = config_dir().expect("Can not find the configuration directory.");
        let sys_config = format!(
            "{}/revanced-discord-bot/{CONFIG_PATH}",
            sys_config_dir.to_string_lossy()
        );

        // config file in current dir
        let mut file = if Path::new(CONFIG_PATH).exists() {
            File::open(CONFIG_PATH)?
        }
        // config file in system dir (on *nix: `~/.config/revanced-discord-bot/`)
        else if Path::new(&sys_config).exists() {
            File::open(sys_config)?
        }
        // create defalt config
        else {
            let default_config = Configuration::default();
            default_config.save()?;

            File::open(sys_config)?
        };

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        Ok(serde_json::from_str(&buf)?)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct General {
    pub embed_color: i32,
    pub mute: Mute,
    pub logging_channel: u64,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Mute {
    pub role: u64,
    pub take: Vec<u64>,
}
#[derive(Default, Serialize, Deserialize)]
pub struct Administrators {
    pub roles: Vec<u64>,
    pub users: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct MessageResponse {
    pub includes: Option<Includes>,
    pub excludes: Option<Excludes>,
    pub condition: Option<Condition>,
    pub response: Response,
    pub thread_options: Option<ThreadOptions>,
}

#[derive(Serialize, Deserialize)]
pub struct ThreadOptions {
    pub close_on_response: bool,
    pub lock_on_response: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub message: Option<String>,
    pub embed: Option<Embed>,
}

#[derive(Serialize, Deserialize)]
pub struct Embed {
    pub title: String,
    pub description: String,
    pub color: i32,
    pub fields: Vec<Field>,
    pub footer: Footer,
    pub image: Image,
    pub thumbnail: Thumbnail,
    pub author: Author,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Footer {
    pub text: String,
    pub icon_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub icon_url: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Includes {
    pub channels: Option<Vec<u64>>,
    #[serde(rename = "match", with = "serde_regex")]
    pub match_field: Vec<Regex>,
}

#[derive(Serialize, Deserialize)]
pub struct Excludes {
    pub roles: Option<Vec<u64>>,
    #[serde(rename = "match", with = "serde_regex")]
    pub match_field: Vec<Regex>,
}

#[derive(Serialize, Deserialize)]
pub struct Condition {
    pub user: User,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub server_age: i64,
}
