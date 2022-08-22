use poise::serenity_prelude::{self as serenity, MessageId};
use poise::ReplyHandle;

use crate::{Context, Error};

/// Make the Discord bot sentient.
#[poise::command(slash_command)]
pub async fn reply(
    ctx: Context<'_>,
    #[description = "The message id to reply to"] reply_message: Option<String>,
    #[description = "The message to send"] message: String,
) -> Result<(), Error> {
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

    send_ephermal(&ctx, &format!("Response: {}", message)).await?;
    Ok(())
}

async fn send_ephermal<'a>(
    ctx: &Context<'a>,
    content: &str,
) -> Result<ReplyHandle<'a>, serenity::Error> {
    ctx.send(|f| f.ephemeral(true).content(content)).await
}
