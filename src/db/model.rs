use std::fmt::Display;

use bson::Document;
use serde::{Deserialize, Serialize};
use serde_with_macros::skip_serializing_none;

// Models
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Muted {
    pub user_id: Option<String>,
    pub guild_id: Option<String>,
    pub taken_roles: Option<Vec<String>>,
    pub expires: Option<u64>,
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
