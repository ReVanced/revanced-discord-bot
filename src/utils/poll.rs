use poise::serenity_prelude::{ButtonStyle, ReactionType, Timestamp};

use sha3::{Digest, Sha3_256};
use tracing::log::{error, info, trace};

use super::bot::get_data_lock;
use super::*;

pub async fn handle_poll(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    poll_id: u64,
    min_join_date: Timestamp,
) -> Result<(), crate::serenity::SerenityError> {
    trace!("Handling poll: {}.", poll_id);

    let data = get_data_lock(ctx).await;
    let data = data.read().await;

    let component = &interaction.clone().message_component().unwrap();

    let member = component.member.as_ref().unwrap();

    let eligible = member.joined_at.unwrap() <= min_join_date;
    let auth_token = if eligible {
        let mut hasher = Sha3_256::new();
        hasher.update(&member.user.id.to_string());
        let result = data
            .api
            // We cannot use the entire hash because Discord rejects URLs with more than 512 characters.
            .authenticate(&hex::encode(hasher.finalize())[..2^5])
            .await
            .map(|auth| auth.access_token);

        if let Err(ref e) = result {
            error!("API Request error: {}", e)
        }
        result.ok()
    } else {
        None
    };

    component
        .create_interaction_response(&ctx.http, |r| {
            r.interaction_response_data(|m| {
                if let Some(token) = auth_token.as_deref() {
                    let url = format!("https://revanced.app/polling#{}", token);
                    m.components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.label("Vote")
                                    .emoji(ReactionType::Unicode("🗳️".to_string()))
                                    .style(ButtonStyle::Link)
                                    .url(&url)
                            })
                        })
                    })
                } else {
                    m
                }
                .ephemeral(true)
                .embed(|e| {
                    if auth_token.is_some() {
                        e.title("Cast your vote")
                            .description("You can now vote on the poll.")
                    } else if !eligible {
                        info!("Member {} failed to vote.", member.display_name());
                        e.title("You can not vote")
                            .description("You are not eligible to vote on this poll.")
                    } else {
                        e.title("Error")
                            .description("An error has occured. Please try again later.")
                    }
                    .color(data.configuration.general.embed_color)
                    .thumbnail(member.user.face())
                })
            })
        })
        .await?;

    Ok(())
}
