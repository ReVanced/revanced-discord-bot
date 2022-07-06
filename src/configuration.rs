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
	pub thread_introductions: Option<Vec<Introduction>>,
	#[serde(rename = "message-responders")]
	pub message_responders: Option<Vec<MessageResponder>>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Administrators {
	pub roles: Vec<u64>,
	pub users: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Introduction {
	pub channels: Vec<u64>,
	pub message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponder {
	pub includes: Option<Includes>,
	pub excludes: Option<Excludes>,
	pub condition: Option<Condition>,
	pub message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Includes {
	pub channels: Option<Vec<u64>>,
	#[serde(rename = "match")]
	pub match_field: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Excludes {
	pub roles: Option<Vec<u64>>,
	#[serde(rename = "match")]
	pub match_field: Option<Vec<String>>,
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
	pub server_age: Option<i16>,
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
