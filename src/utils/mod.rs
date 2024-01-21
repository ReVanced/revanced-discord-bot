use chrono::Duration;
use poise::serenity_prelude::{self as serenity, Member, RoleId};

pub mod bot;
pub mod code_embed;
pub mod decancer;
pub mod macros;
pub mod message_response;
pub mod moderation;

pub fn parse_duration(duration: String) -> Result<Duration, parse_duration::parse::Error> {
    let d = parse_duration::parse(&duration)?;
    Ok(Duration::nanoseconds(d.as_nanos() as i64))
}
