use std::collections::HashMap;

use poise::{CacheHttp, ReactionType, Timestamp};
use tokio::join;
use tracing::log::{info, trace};

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

    let auth_token = ""; // get via api

    component
        .create_interaction_response(&ctx.http, |r| {
            r.interaction_response_data(|m| {
                let allowed = member.joined_at.unwrap() <= min_join_date;
                if allowed {
                    // TODO: get the url from the api server

                    m.components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.label("Vote")
                                    .emoji(ReactionType::Unicode("🗳️".to_string()))
                                    .style(ButtonStyle::Link)
                                    .url("https://revanced.app")
                            })
                        })
                    })
                } else {
                    m
                }
                .ephemeral(true)
                .embed(|e| {
                    if allowed {
                        e.title("Cast your vote")
                            .description("You can now vote on the poll.")
                    } else {
                        info!("Member {} failed to vote.", member.display_name());
                        e.title("You can not vote")
                            .description("You are not eligible to vote on this poll.")
                    }
                    .color(data.configuration.general.embed_color)
                    .thumbnail(member.user.face())
                })
            })
        })
        .await?;

    Ok(())
}
