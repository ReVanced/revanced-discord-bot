use poise::serenity_prelude::{CreateActionRow, CreateAllowedMentions, CreateButton, ReactionType};
use poise::{CreateReply, ReplyHandle};

use crate::utils::message::clone_message;
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

/// Start a poll.
#[poise::command(slash_command)]
pub async fn poll(
    ctx: Context<'_>,
    #[description = "The id of the poll"] id: u64, /* This is currently unused in the API, leaving as a placeholder in case it is required. */
    #[description = "A link to a message to clone"] message_link: String,
    #[description = "The minumum server age in days to allow members to poll"] age: u16,
    #[description = "Enable pings"] ping: bool,
) -> Result<(), Error> {
    let get_id =
        |segments: &mut std::str::Split<char>| segments.next_back().unwrap().parse::<u64>();

    let url = reqwest::Url::parse(&message_link)?;
    let mut segments = url.path_segments().ok_or("Invalid Discord message link")?;

    if segments.clone().count() != 4 {
        return Err("Invalid Discord message link".into());
    }

    let message_id = get_id(&mut segments)?;
    let channel_id = get_id(&mut segments)?;

    let message = ctx
        .serenity_context()
        .http
        .get_message(channel_id.into(), message_id.into())
        .await?;

    let message = clone_message(&message).components(vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("poll:{id}:{age}"))
            .label("Vote")
            .emoji(ReactionType::Unicode("🗳️".to_string())),
    ])]);

    ctx.send(if ping {
        message.allowed_mentions(
            CreateAllowedMentions::default()
                .all_roles(true)
                .all_users(true)
                .everyone(true),
        )
    } else {
        message
    })
    .await?;

    Ok(())
}
