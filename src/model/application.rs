use std::{
	fs::File,
	io::{Read, Result, Write},
};

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Configuration {
	pub administrators: Administrators,
	pub thread_introductions: Vec<Introduction>,
	pub message_responses: Vec<MessageResponse>,
}

const CONFIG_PATH: &str = "configuration.json";

impl Configuration {
	fn save(&self) -> Result<()> {
		let mut file = File::create(CONFIG_PATH)?;
		let json = serde_json::to_string_pretty(&self)?;
		file.write_all(json.as_bytes())?;
		Ok(())
	}

	pub fn load() -> Result<Configuration> {
		let mut file = match File::open(CONFIG_PATH) {
			Ok(file) => file,
			Err(_) => {
				let configuration = Configuration::default();
				configuration.save()?;
				return Ok(configuration);
			},
		};

		let mut buf = String::new();
		file.read_to_string(&mut buf)?;

		Ok(serde_json::from_str(&buf)?)
	}
}

#[derive(Default, Serialize, Deserialize)]
pub struct Administrators {
	pub roles: Vec<u64>,
	pub users: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct Introduction {
	pub channels: Vec<u64>,
	pub response: Response,
}

#[derive(Serialize, Deserialize)]
pub struct MessageResponse {
	pub includes: Includes,
	pub excludes: Excludes,
	pub condition: Condition,
	pub response: Response,
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
	pub channels: Vec<u64>,
	#[serde(rename = "match", with = "serde_regex")]
	pub match_field: Vec<Regex>,
}

#[derive(Serialize, Deserialize)]
pub struct Excludes {
	pub roles: Vec<u64>,
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
