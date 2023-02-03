use chrono::Utc;
use poise::serenity_prelude::{self as serenity, MessageId, ReactionType};
use poise::ReplyHandle;

use crate::{Context, Error};

/// Make the Discord bot sentient.
#[poise::command(slash_command)]
pub async fn reply(
    ctx: Context<'_>,
    #[description = "The message id to reply to"] reply_message: Option<String>,
    #[description = "The message to send"] message: String,
) -> Result<(), Error> {
    async fn send_ephermal<'a>(
        ctx: &Context<'a>,
        content: &str,
    ) -> Result<ReplyHandle<'a>, serenity::Error> {
        ctx.send(|f| f.ephemeral(true).content(content)).await
    }

    let http = &ctx.discord().http;
    let channel = &ctx.channel_id();

    if let Some(reply_message) = reply_message {
        if let Ok(reply_message) = reply_message.parse::<u64>() {
            match channel.message(http, MessageId(reply_message)).await {
                Ok(reply_message) => {
                    reply_message.reply(http, &message).await?;
                },
                Err(_) => {
                    send_ephermal(
                        &ctx,
                        "The message you are trying to reply to does not exist.",
                    )
                    .await?;
                },
            }
        } else {
            send_ephermal(&ctx, "Invalid message id.").await?;
        }
    } else {
        channel.say(http, &message).await?;
    }

    send_ephermal(&ctx, &format!("Response: {message}")).await?;
    Ok(())
}

/// Start a poll.
#[poise::command(slash_command)]
pub async fn poll(
    ctx: Context<'_>,
    #[description = "The id of the poll"] id: u64,
    #[description = "The poll message"] message: String,
    #[description = "The poll title"] title: String,
    #[description = "The minumum server age in days to allow members to poll"] age: u16,
) -> Result<(), Error> {
    let data = ctx.data().read().await;
    let configuration = &data.configuration;
    let embed_color = configuration.general.embed_color;

    ctx.send(|m| {
        m.embed(|e| {
            let guild = &ctx.guild().unwrap();
            if let Some(url) = guild.icon_url() {
                e.thumbnail(url.clone()).footer(|f| {
                    f.icon_url(url).text(format!(
                        "{} • {}",
                        guild.name,
                        Utc::today().format("%Y/%m/%d")
                    ))
                })
            } else {
                e
            }
            .title(title)
            .description(message)
            .color(embed_color)
        })
        .components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.label("Vote")
                        .emoji(ReactionType::Unicode("🗳️".to_string()))
                        .custom_id(format!("poll:{id}:{age}"))
                })
            })
        })
    })
    .await?;
    Ok(())
}
