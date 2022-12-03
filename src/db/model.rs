use std::fmt::Display;

use bson::Document;
use poise::serenity_prelude::PermissionOverwrite;
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LockedChannel {
    pub channel_id: Option<String>,
    pub overwrites: Option<Vec<PermissionOverwrite>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Poll {
    pub author: Option<PollAuthor>,
    pub image_url: Option<String>,
    pub votes: Option<u16>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PollAuthor {
    pub name: Option<String>,
    pub id: Option<u64>,
}

impl From<Muted> for Document {
    fn from(muted: Muted) -> Self {
        to_document(&muted)
    }
}

impl From<LockedChannel> for Document {
    fn from(locked: LockedChannel) -> Self {
        to_document(&locked)
    }
}

fn to_document<T>(t: &T) -> Document
where
    T: Serialize,
{
    bson::to_document(t).unwrap()
}

// Display trait
impl Display for Muted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "muted")
    }
}
