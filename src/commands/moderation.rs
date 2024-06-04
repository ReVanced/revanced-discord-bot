use bson::{doc, Document};
use chrono::Utc;
use mongodb::options::{UpdateModifications, UpdateOptions};
use poise::serenity_prelude::{
    CreateEmbedFooter, EditMessage, GetMessages, Mentionable, UserId,
};
use poise::CreateReply;
use tracing::{debug, trace};

use crate::db::model::Muted;
use crate::utils::bot::get_member;
use crate::utils::macros::to_user;
use crate::utils::moderation::{
    ban_moderation, queue_unmute_member, respond_moderation, BanKind, ModerationKind,
};
use crate::utils::{create_default_embed, parse_duration};
use crate::{Context, Error};

/// Unmute a member.
#[poise::command(slash_command)]
pub async fn unmute(
    ctx: Context<'_>,
    #[description = "The member to unmute"] member: UserId,
) -> Result<(), Error> {
    let user = to_user!(member, ctx);
    let id = user.id;
    ctx.defer().await.expect("Failed to defer");

    let data = &ctx.data().read().await;
    let configuration = &data.configuration;

    if let Some(pending_unmute) = data.pending_unmutes.get(&id.get()) {
        trace!("Cancelling pending unmute for {}", id);
        pending_unmute.abort();
    }

    let author = ctx.author();

    let queue = queue_unmute_member(
        ctx.serenity_context().clone(),
        data.database.clone(),
        ctx.guild_id().unwrap(),
        id,
        configuration.mute.role,
        0,
    )
    .await
    .unwrap()
    .err();

    respond_moderation(
        &ctx,
        &ModerationKind::Unmute(user, author.clone(), queue),
        configuration,
    )
    .await
}

/// Mute a member.
#[poise::command(slash_command)]
pub async fn mute(
    ctx: Context<'_>,
    #[description = "The member to mute"] member: UserId,
    #[description = "The duration of the mute"] duration: String,
    #[description = "The reason of the mute"] reason: String,
) -> Result<(), Error> {
    let user = to_user!(member, ctx);
    let id = user.id;
    let now = Utc::now();
    let mute_duration = parse_duration(duration).map_err(|e| Error::from(format!("{:?}", e)))?;

    let data = &mut *ctx.data().write().await;
    let configuration = &data.configuration;
    let author = ctx.author();

    let mute = &configuration.mute;
    let guild_id = ctx.guild_id().unwrap();

    let discord = ctx.serenity_context();

    let unmute_time = if !mute_duration.is_zero() {
        Some((now + mute_duration).timestamp() as u64)
    } else {
        None
    };

    let mut updated = Muted {
        guild_id: Some(guild_id.to_string()),
        expires: unmute_time,
        reason: Some(reason.clone()),
        ..Default::default()
    };

    let result = async {
        if let Some(mut member) = get_member(discord, guild_id, id).await? {
            let (is_currently_muted, removed_roles) =
                crate::utils::moderation::mute_moderation(&ctx, &mut member, mute).await?;
            // Prevent the bot from overriding the "take" field.
            // This would happen otherwise, because the bot would accumulate the users roles and then override the value in the database
            // resulting in the user being muted to have no roles to add back later.
            if !is_currently_muted {
                updated.taken_roles = Some(removed_roles.iter().map(ToString::to_string).collect());
            }
        }

        let query: Document = Muted {
            user_id: Some(id.to_string()),
            ..Default::default()
        }
        .into();

        let updated: Document = updated.into();
        data.database
            .update::<Muted>(
                "muted",
                query.clone(),
                UpdateModifications::Document(doc! { "$set": updated }),
                Some(UpdateOptions::builder().upsert(true).build()),
            )
            .await?;

        if let Some(pending_unmute) = data.pending_unmutes.get(&id.get()) {
            trace!("Cancelling pending unmute for {}", id);
            pending_unmute.abort();
        }

        if unmute_time.is_none() {
            data.database
                .update::<Muted>(
                    "muted",
                    query,
                    UpdateModifications::Document(doc! { "$unset": { "expires": "" } }),
                    None,
                )
                .await?;
        } else {
            data.pending_unmutes.insert(
                id.get(),
                queue_unmute_member(
                    discord.clone(),
                    data.database.clone(),
                    guild_id,
                    id,
                    mute.role,
                    mute_duration.num_seconds() as u64,
                ),
            );
        }
        Ok(())
    }
    .await;

    let duration = unmute_time.map(|time| format!("<t:{time}:F>"));

    respond_moderation(
        &ctx,
        &ModerationKind::Mute(user, author.clone(), reason, duration, result.err()),
        configuration,
    )
    .await
}

/// Delete recent messages of a member. Cannot delete messages older than 14 days.
#[poise::command(slash_command)]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "Member"] user: Option<UserId>,
    #[min = 1]
    #[max = 1000]
    #[description = "Count"]
    count: Option<i64>,
    #[description = "Until message"] until: Option<String>,
) -> Result<(), Error> {
    let user = if let Some(id) = user {
        Some(to_user!(id, ctx))
    } else {
        None
    };
    // The maximum amount of times to page through messages. If paged over MAX_PAGES amount of times without deleting messages, break.
    const MAX_PAGES: i8 = 2;
    // The maximal amount of messages that we can fetch at all
    const MAX_BULK_DELETE: i64 = 100;
    // Discord does not let us bulk-delete messages older than 14 days
    const MAX_BULK_DELETE_AGO_SECS: i64 = 60 * 60 * 24 * 14;

    let data = ctx.data().read().await;
    let configuration = &data.configuration;
    let channel = ctx.channel_id();
    let too_old_timestamp = Utc::now().timestamp() - MAX_BULK_DELETE_AGO_SECS;

    let current_user = ctx.serenity_context().http.get_current_user().await?;
    let image = current_user.face();

    let author = ctx.author();

    let handle = ctx
        .send(CreateReply {
            embeds: vec![create_default_embed(configuration)
                .title("Purging messages")
                .description("Accumulating...")
                .thumbnail(&image)],
            ..Default::default()
        })
        .await?;
    let mut response = handle.message().await?;

    ctx.defer().await?;

    let count_to_delete = count.unwrap_or(MAX_BULK_DELETE) as usize;
    let mut deleted_amount = 0;
    let mut empty_pages: i8 = 0;

    loop {
        // Filter out messages that are too old
        let mut messages = channel
            .messages(
                &ctx.serenity_context(),
                GetMessages::new()
                    .limit(count_to_delete as u8)
                    .before(response.id),
            )
            .await?
            .into_iter()
            .take_while(|m| m.timestamp.timestamp() > too_old_timestamp)
            .collect::<Vec<_>>();

        // Filter for messages from the user
        if let Some(ref user) = user {
            messages = messages
                .into_iter()
                .filter(|msg| msg.author.id == user.id)
                .collect::<Vec<_>>();

            debug!("Filtered messages by {}. Left: {}", user, messages.len());
        }

        // Filter for messages until the g/mutiven id
        if let Some(ref message_id) = until {
            if let Ok(message_id) = message_id.parse::<u64>() {
                messages = messages
                    .into_iter()
                    .take_while(|m| m.id.get() > message_id)
                    .collect::<Vec<_>>();
                debug!(
                    "Filtered messages until {}. Left: {}",
                    message_id,
                    messages.len()
                );
            }
        }

        let purge_count = messages.len();
        if purge_count > 0 {
            deleted_amount += purge_count;
            channel
                .delete_messages(&ctx.serenity_context(), &messages)
                .await?;
        } else {
            empty_pages += 1;
        }

        if empty_pages >= MAX_PAGES || deleted_amount >= count_to_delete {
            break;
        }
    }

    response
        .to_mut()
        .edit(
            &ctx.serenity_context(),
            EditMessage::new().embed(
                create_default_embed(configuration)
                    .title("Purge successful")
                    .field("Deleted messages", deleted_amount.to_string(), false)
                    .field("Action by", author.mention().to_string(), false)
                    .thumbnail(&image)
                    .footer(CreateEmbedFooter::new("ReVanced").icon_url(image)),
            ),
        )
        .await?;
    Ok(())
}

/// Ban a member.
#[poise::command(slash_command)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "Member"] user: UserId,
    #[description = "Amount of days to delete messages"] dmd: Option<u8>,
    #[description = "Reason for the ban"] reason: Option<String>,
) -> Result<(), Error> {
    // We cannot use `User` as a parameter for the moderation commands because of a bug in serenity. See: https://github.com/revanced/revanced-discord-bot/issues/38
    let user = to_user!(user, ctx);
    handle_ban(&ctx, &BanKind::Ban(user, dmd, reason)).await
}

/// Unban a user.
#[poise::command(slash_command)]
pub async fn unban(ctx: Context<'_>, #[description = "User"] user: UserId) -> Result<(), Error> {
    let user = to_user!(user, ctx);
    handle_ban(&ctx, &BanKind::Unban(user)).await
}

async fn handle_ban(ctx: &Context<'_>, kind: &BanKind) -> Result<(), Error> {
    let data = ctx.data().read().await;

    let ban_result = ban_moderation(ctx, kind).await;

    let author = ctx.author();

    respond_moderation(
        ctx,
        &match kind {
            BanKind::Ban(user, _, reason) => {
                ModerationKind::Ban(user.clone(), author.clone(), reason.clone(), ban_result)
            },
            BanKind::Unban(user) => ModerationKind::Unban(user.clone(), author.clone(), ban_result),
        },
        &data.configuration,
    )
    .await
}
