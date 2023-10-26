use base64::Engine;
use poise::serenity_prelude::{
    CreateActionRow,
    CreateButton,
    CreateEmbed,
    CreateEmbedFooter,
    CreateInteractionResponseMessage,
    ReactionType,
    Timestamp,
};
use reqwest::StatusCode;
use tracing::log::{error, trace};

use super::bot::get_data_lock;
use super::*;

pub async fn handle_poll(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    poll_id: u64,
    min_join_date: Timestamp,
) -> Result<(), serenity::prelude::SerenityError> {
    trace!("Handling poll: {}.", poll_id);

    let data = get_data_lock(ctx).await;
    let data = data.read().await;

    let component = &interaction.clone().message_component().unwrap();

    let member = component.member.as_ref().unwrap();

    let eligible = member.joined_at.unwrap() <= min_join_date;
    let result = if eligible {
        match data
            .api
            .authenticate(&base64::engine::general_purpose::STANDARD_NO_PAD.encode(
                hmac_sha256::HMAC::mac(member.user.id.to_string(), &data.poll_secret),
            ))
            .await
        {
            Ok(auth) => Ok(auth.access_token),
            Err(err) => match err.status() {
                Some(StatusCode::PRECONDITION_FAILED) => Err("You can only vote once."),
                _ => {
                    error!("API Request error: {:?}", err);
                    Err("API Request failed. Please try again later.")
                },
            },
        }
    } else {
        Err("You are not eligible to vote on this poll because you joined this server too recently.")
    };

    let icon_url = component
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .unwrap()
        .icon_url()
        .unwrap();

    component
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::Message(if let Ok(token) = result.as_deref() {
                let url = format!("https://revanced.app/poll#{token}");

                CreateInteractionResponseMessage::new().components(vec![CreateActionRow::Buttons(
                    vec![CreateButton::new_link(url)
                        .label("Vote")
                        .emoji(ReactionType::Unicode("🗳️".to_string()))],
                )])
            } else {
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .embed(
                        match result {
                            Ok(_) => CreateEmbed::new()
                                .title("Cast your vote")
                                .description("You can now vote on the poll."),
                            Err(msg) => CreateEmbed::new().title("Error").description(msg),
                        }
                        .color(data.configuration.general.embed_color)
                        .thumbnail(&icon_url)
                        .footer(CreateEmbedFooter::new("ReVanced").icon_url(&icon_url)),
                    )
            }),
        )
        .await?;

    Ok(())
}
