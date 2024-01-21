use poise::{CreateReply, ReplyHandle};

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
    ) -> Result<ReplyHandle<'a>, poise::serenity_prelude::Error> {
        ctx.send(CreateReply {
            ephemeral: Some(true),
            content: Some(content.to_string()),
            ..Default::default()
        })
        .await
    }

    let http = &ctx.serenity_context().http;
    let channel = &ctx.channel_id();

    if let Some(reply_message) = reply_message {
        if let Ok(reply_message) = reply_message.parse::<u64>() {
            match channel.message(http, reply_message).await {
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
