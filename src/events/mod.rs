use std::sync::Arc;

use crate::{BotData, Data, Error};
use poise::serenity_prelude::{self as serenity, Member};
use tokio::sync::RwLock;

mod guild_member_addition;
mod guild_member_update;
mod message_create;
mod ready;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    data: &BotData,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { .. } => ready::load_muted_members(ctx, data).await,
        serenity::FullEvent::Message { new_message } => {
            message_create::message_create(ctx, new_message, data).await;
        },
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            guild_member_addition::guild_member_addition(ctx, new_member, data).await
        },
        serenity::FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            ..
        } => guild_member_update::guild_member_update(ctx, old_if_available, new).await,
        _ => {},
    }
    Ok(())
}
