use chrono::{Duration, Utc};
use poise::serenity_prelude::{
    ComponentType,
    MessageComponentInteraction,
    MessageComponentInteractionData,
};

use super::*;
use crate::utils;
pub async fn interaction_create(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
) -> Result<(), crate::serenity::SerenityError> {
    if let serenity::Interaction::MessageComponent(MessageComponentInteraction {
        data:
            MessageComponentInteractionData {
                component_type: ComponentType::Button,
                custom_id,
                ..
            },
        ..
    }) = interaction
    {
        if custom_id.starts_with("poll") {
            handle_poll(ctx, interaction, custom_id).await?
        }
    }

    Ok(())
}

pub async fn handle_poll(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    custom_id: &str,
) -> Result<(), crate::serenity::SerenityError> {
    fn parse<T>(str: &str) -> T
    where
        <T as std::str::FromStr>::Err: std::fmt::Debug,
        T: std::str::FromStr,
    {
        str.parse::<T>().unwrap()
    }

    let poll: Vec<_> = custom_id.split(':').collect::<Vec<_>>();

    let poll_id = parse::<u64>(poll[1]);
    let min_age = parse::<i64>(poll[2]);

    let min_join_date = serenity::Timestamp::from(Utc::now() - Duration::days(min_age));

    utils::poll::handle_poll(ctx, interaction, poll_id, min_join_date).await
}
