use poise::serenity_prelude::{self as serenity};

use crate::model::application::Configuration;

pub fn load_configuration() -> Configuration {
    Configuration::load().expect("Failed to load configuration")
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
