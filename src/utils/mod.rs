use chrono::Duration;
use poise::serenity_prelude::{self as serenity, Member, RoleId};

pub mod autorespond;
pub mod bot;
pub mod code_embed;
pub mod decancer;
pub mod message;
pub mod macros;
pub mod media_channel;
pub mod moderation;
pub mod poll;

pub fn parse_duration(duration: String) -> Result<Duration, go_parse_duration::Error> {
    let d = go_parse_duration::parse_duration(&duration)?;
    Ok(Duration::nanoseconds(d))
}
