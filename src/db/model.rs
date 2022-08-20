use std::fmt::Display;

use bson::Document;
use serde::{Deserialize, Serialize};

// Models
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Muted {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taken_roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl From<Muted> for Document {
    fn from(muted: Muted) -> Self {
        bson::to_document(&muted).unwrap()
    }
}

// Display trait
impl Display for Muted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "muted")
    }
}
