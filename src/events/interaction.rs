use chrono::{Duration, Utc};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionData, ComponentInteractionDataKind,
};

use super::*;
use crate::{utils, BotData};

pub async fn interaction_create(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    data: &BotData,
) -> Result<(), serenity::prelude::SerenityError> {
    if let serenity::Interaction::Component(ComponentInteraction {
        data:
            ComponentInteractionData {
                kind: ComponentInteractionDataKind::Button,
                custom_id,
                ..
            },
        ..
    }) = interaction
    {
        if custom_id.starts_with("poll") {
            handle_poll(ctx, interaction, custom_id, data).await?
        }
    }

    Ok(())
}

pub async fn handle_poll(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    custom_id: &str,
    data: &BotData,
) -> Result<(), serenity::prelude::SerenityError> {
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

    utils::poll::handle_poll(ctx, interaction, poll_id, min_join_date, data).await
}
