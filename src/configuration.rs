use std::fs::File;
use std::io::{Error, Read, Write};

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotConfiguration {
	#[serde(rename = "discord-authorization-token")]
	pub discord_authorization_token: String,
	pub administrators: Administrators,
	#[serde(rename = "thread-introductions")]
	pub thread_introductions: Vec<Introduction>,
	#[serde(rename = "message-responders")]
	pub message_responders: Vec<MessageResponder>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Administrators {
	pub roles: Vec<u64>,
	pub users: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Introduction {
	pub channels: Vec<u64>,
	pub response: Response,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponder {
	pub includes: Includes,
	pub excludes: Excludes,
	pub condition: Condition,
	pub response: Response,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
	pub message: Option<String>,
	pub embed: Option<Embed>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct Field {
	pub name: String,
	pub value: String,
	pub inline: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Footer {
	pub text: String,
	#[serde(rename = "icon_url")]
	pub icon_url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
	pub url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
	pub url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
	pub name: String,
	#[serde(rename = "icon_url")]
	pub icon_url: String,
	pub url: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Includes {
	pub channels: Vec<u64>,
	#[serde(rename = "match")]
	pub match_field: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Excludes {
	pub roles: Vec<u64>,
	#[serde(rename = "match")]
	pub match_field: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
	pub user: User,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
	#[serde(rename = "server-age")]
	pub server_age: i64,
}

impl BotConfiguration {
	fn save(&self) -> Result<(), Error> {
		let mut file = File::create("configuration.json")?;
		let json = serde_json::to_string_pretty(&self)?;
		file.write(json.as_bytes())?;
		Ok(())
	}

	pub fn load() -> Result<BotConfiguration, Error> {
		let mut file = match File::open("configuration.json") {
			Ok(file) => file,
			Err(_) => {
				let configuration = BotConfiguration::default();
				configuration.save()?;
				return Ok(configuration);
			},
		};

		let mut buf = String::new();
		file.read_to_string(&mut buf)?;
		Ok(serde_json::from_str(&buf)?)
	}
}
