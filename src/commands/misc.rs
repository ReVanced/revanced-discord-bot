use bson::{doc, Document};
use chrono::Utc;
use mongodb::options::{UpdateModifications, UpdateOptions};
use poise::serenity_prelude::{self as serenity, MessageId, ReactionType};
use poise::ReplyHandle;

use crate::db::model::KeepAliveThread;

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

    send_ephermal(&ctx, &format!("Response: {}", message)).await?;
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
                        .custom_id(format!("poll:{}:{}", id, age))
                })
            })
        })
    })
    .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn keep_thread_alive(ctx: Context<'_>) -> Result<(), Error> {
    let data = &ctx.data().read().await;
    let database = &data.database;
    let discord = &ctx.discord();
    let cache = &discord.cache;

    let channel_id = ctx.channel_id().0;
    let channel = &cache.guild_channel(channel_id).unwrap();

    let author = ctx.author();
    let current_user = ctx.discord().http.get_current_user().await?;
    let image = current_user.face();

    let query: Document = KeepAliveThread {
        thread_id: Some(channel_id.to_string()),
    }
    .into();

    let (title, description, ephemeral) = if channel.thread_metadata.is_none() {
        ("Error", "Channel is not a thread.", true)
    } else if (database
        .find_one::<KeepAliveThread>("keep_alive", query.clone(), None)
        .await?).is_some()
    {
        ("Error", "Thread is already kept alive", true)
    } else {
        database
            .update::<KeepAliveThread>(
                "keepalive",
                query,
                UpdateModifications::Document(doc! { "$set": channel_id.to_string()}),
                Some(UpdateOptions::builder().upsert(true).build()),
            )
            .await?;
        (
            "Keeping the thread alive",
            "Killing the thread will make the thread close per discord's inactivity setting.",
            false,
        )
    };

    ctx.send(|f| {
        f.ephemeral(ephemeral).embed(|f| {
            f.title(title)
                .description(description)
                .footer(|f| {
                    f.text("ReVanced");
                    f.icon_url(&image)
                })
                .thumbnail(
                    &author
                        .avatar_url()
                        .unwrap_or_else(|| author.default_avatar_url()),
                )
        })
    })
    .await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn kill_thread(ctx: Context<'_>) -> Result<(), Error> {
    let data = &ctx.data().read().await;
    let database = &data.database;
    let discord = &ctx.discord();
    let cache = &discord.cache;

    let channel_id = ctx.channel_id().0;
    let current_user = ctx.discord().http.get_current_user().await?;
    let image = current_user.face();

    let channel = cache.guild_channel(channel_id).unwrap();

    let author = ctx.author();

    let (title, description, ephemeral) = if channel.thread_metadata.is_none() {
        ("Error", "Channel is not a thread.", true)
    } else if database
        .delete(
            "keep_alive",
            KeepAliveThread {
                thread_id: Some(channel_id.to_string()),
            }
            .into(),
            None,
        )
        .await?
        .deleted_count
        != 0
    {
        ("Killed thread", "Thread will no longer be kept alive, it will be closed per discord's inactivity setting.", false)
    } else {
        ("Error", "Thread already not kept alive", true)
    };

    ctx.send(|f| {
        f.ephemeral(ephemeral).embed(|f| {
            f.title(title)
                .description(description)
                .footer(|f| {
                    f.text("ReVanced");
                    f.icon_url(&image)
                })
                .thumbnail(
                    &author
                        .avatar_url()
                        .unwrap_or_else(|| author.default_avatar_url()),
                )
        })
    })
    .await?;
    Ok(())
}
