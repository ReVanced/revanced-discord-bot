use std::sync::Arc;

use poise::serenity_prelude::{self as serenity};

use crate::model::application::Configuration;
use crate::Data;

pub fn load_configuration() -> Configuration {
    Configuration::load().expect("Failed to load configuration")
}

// Share the lock reference between the threads in serenity framework
pub async fn get_data_lock(
    ctx: &serenity::Context,
) -> Arc<poise::serenity_prelude::prelude::RwLock<Data>> {
    ctx.data.read().await.get::<Data>().unwrap().clone()
}

pub async fn get_member(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
) -> serenity::Result<Option<serenity::Member>> {
    match guild_id.member(ctx, user_id).await {
        Ok(member) => Ok(Some(member)),
        Err(serenity::prelude::SerenityError::Http(err))
            if matches!(
                err.status_code(),
                Some(serenity::http::StatusCode::NOT_FOUND)
            ) =>
        {
            Ok(None)
        },
        Err(err) => Err(err),
    }
}
