use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Result, Write};
use std::path::Path;

use dirs::config_dir;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Configuration {
    pub administrators: Administrators,
    pub default_embed: Embed,
    pub mute: Mute,
    pub log_channel: u64,
    pub responses: Vec<Response>,
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

        // Config file in the current directory.
        let mut file = if Path::new(CONFIG_PATH).exists() {
            File::open(CONFIG_PATH)?
        }
        // Config file in the system directory.
        else if Path::new(&sys_config).exists() {
            File::open(sys_config)?
        }
        // Create a default config file.
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
pub struct Mute {
    pub role: u64,
    pub take: HashSet<u64>,
}
#[derive(Default, Serialize, Deserialize)]
pub struct Administrators {
    pub roles: HashSet<u64>,
    pub users: HashSet<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub whitelist: Option<Trigger>,
    pub blacklist: Option<Trigger>,
    pub message: Message,
    pub respond_to_reference: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub content: Option<String>,
    pub embed: Option<Embed>,
}

#[derive(Serialize, Deserialize)]
#[derive(Default)]
pub struct Embed {
    pub author: Option<Author>,
    pub color: Option<i32>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub fields: Option<Vec<Field>>,
    pub footer: Option<Footer>,
    pub image_url: Option<String>,
    pub thumbnail_url: Option<String>,
}


#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub inline: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Footer {
    pub text: String,
    pub icon_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub icon_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Trigger {
    pub channels: Option<HashSet<u64>>,
    pub roles: Option<HashSet<u64>>,
    #[serde(with = "serde_regex")]
    pub regex: Vec<Regex>,
}
