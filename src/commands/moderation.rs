use bson::{doc, Document};
use chrono::{Duration, Utc};
use mongodb::options::{UpdateModifications, UpdateOptions};
use poise::serenity_prelude::{
    self as serenity,
    Member,
    PermissionOverwrite,
    Permissions,
    RoleId,
    User,
};
use tracing::log::error;
use tracing::{debug, warn, trace};

use crate::db::model::{LockedChannel, Muted};
use crate::utils::moderation::{
    ban_moderation,
    queue_unmute_member,
    respond_moderation,
    BanKind,
    ModerationKind,
};
use crate::{Context, Error};

/// Lock a channel.
#[poise::command(slash_command)]
pub async fn lock(ctx: Context<'_>) -> Result<(), Error> {
    let data = &ctx.data().read().await;
    let configuration = &data.configuration;
    let database = &data.database;
    let discord = &ctx.discord();
    let cache = &discord.cache;
    let http = &discord.http;

    let channel_id = ctx.channel_id().0;
    let channel = &cache.guild_channel(channel_id).unwrap();

    let query: Document = LockedChannel {
        channel_id: Some(channel_id.to_string()),
        ..Default::default()
    }
    .into();

    // Check if channel is already muted, if so succeed.
    if let Ok(mut cursor) = database
        .find::<LockedChannel>("locked", query.clone(), None)
        .await
    {
        if cursor.advance().await.unwrap() {
            respond_moderation(
                &ctx,
                &ModerationKind::Lock(
                    channel.name.clone(),
                    Some(Error::from("Channel already locked")),
                ),
                configuration,
            )
            .await?;
            return Ok(());
        }
    }

    // accumulate all roles with write permissions
    let permission_overwrites: Vec<_> = channel
        .permission_overwrites
        .iter()
        .filter_map(|r| {
            if r.allow.send_messages() || !r.deny.send_messages() {
                Some(r.clone())
            } else {
                None
            }
        })
        .collect();

    // save the original overwrites
    let updated: Document = LockedChannel {
        overwrites: Some(permission_overwrites.clone()),
        ..Default::default()
    }
    .into();

    database
        .update::<LockedChannel>(
            "locked",
            query,
            UpdateModifications::Document(doc! { "$set": updated}),
            Some(UpdateOptions::builder().upsert(true).build()),
        )
        .await?;

    // lock the channel by and creating the new permission overwrite
    for permission_overwrite in &permission_overwrites {
        let permission = Permissions::SEND_MESSAGES & Permissions::ADD_REACTIONS;

        if let Err(err) = channel
            .create_permission(http, &PermissionOverwrite {
                allow: permission_overwrite.allow & !permission,
                deny: permission_overwrite.deny | permission,
                kind: permission_overwrite.kind,
            })
            .await
        {
            error!("Failed to create the new permission: {:?}", err);
        }
    }

    respond_moderation(
        &ctx,
        &ModerationKind::Lock(channel.name.clone(), None),
        configuration,
    )
    .await
}

/// Unlock a channel.
#[poise::command(slash_command)]
pub async fn unlock(ctx: Context<'_>) -> Result<(), Error> {
    let data = &ctx.data().read().await;
    let configuration = &data.configuration;
    let database = &data.database;
    let discord = &ctx.discord();
    let cache = &discord.cache;
    let http = &discord.http;

    let channel_id = ctx.channel_id().0;

    let delete_result = database
        .find_and_delete::<LockedChannel>(
            "locked",
            LockedChannel {
                channel_id: Some(channel_id.to_string()),
                ..Default::default()
            }
            .into(),
            None,
        )
        .await;

    let channel = cache.guild_channel(channel_id).unwrap();
    let mut error = None;
    if let Ok(Some(locked_channel)) = delete_result {
        for overwrite in &locked_channel.overwrites.unwrap() {
            channel.create_permission(http, overwrite).await?;
        }
    } else {
        error = Some(Error::from("Channel already unlocked"))
    }

    respond_moderation(
        &ctx,
        &ModerationKind::Unlock(channel.name.clone(), error), // TODO: handle error
        configuration,
    )
    .await
}

/// Unmute a member.
#[poise::command(slash_command)]
pub async fn unmute(
    ctx: Context<'_>,
    #[description = "The member to unmute"] member: Member,
) -> Result<(), Error> {
    ctx.defer().await.expect("Failed to defer");

    let data = &ctx.data().read().await;
    let configuration = &data.configuration;

    if let Some(pending_unmute) = data.pending_unmutes.get(&member.user.id.0) {
        trace!("Cancelling pending unmute for {}", member.user.id.0);
        pending_unmute.abort();
    }

    let queue = queue_unmute_member(
        &ctx.discord().http,
        &data.database,
        &member,
        configuration.general.mute.role,
        0,
    )
    .await
    .unwrap();

    respond_moderation(
        &ctx,
        &ModerationKind::Unmute(member.user, queue),
        configuration,
    )
    .await
}

/// Mute a member.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command)]
pub async fn mute(
    ctx: Context<'_>,
    #[description = "The member to mute"] mut member: Member,
    #[description = "Seconds"] seconds: Option<i64>,
    #[description = "Minutes"] minutes: Option<i64>,
    #[description = "Hours"] hours: Option<i64>,
    #[description = "Days"] days: Option<i64>,
    #[description = "Months"] months: Option<i64>,
    #[description = "The reason of the mute"] reason: String,
) -> Result<(), Error> {
    let now = Utc::now();
    let mut mute_duration = Duration::zero();

    if let Some(seconds) = seconds {
        mute_duration = mute_duration
            .checked_add(&Duration::seconds(seconds))
            .unwrap();
    }
    if let Some(minutes) = minutes {
        mute_duration = mute_duration
            .checked_add(&Duration::minutes(minutes))
            .unwrap();
    }
    if let Some(hours) = hours {
        mute_duration = mute_duration.checked_add(&Duration::hours(hours)).unwrap();
    }
    if let Some(days) = days {
        mute_duration = mute_duration.checked_add(&Duration::days(days)).unwrap();
    }
    if let Some(months) = months {
        const DAYS_IN_MONTH: i64 = 30;
        mute_duration = mute_duration
            .checked_add(&Duration::days(months * DAYS_IN_MONTH))
            .unwrap();
    }

    let unmute_time = now + mute_duration;

    let data = &mut *ctx.data().write().await;
    let configuration = &data.configuration;
    let mute = &configuration.general.mute;
    let mute_role_id = mute.role;
    let take = &mute.take;
    let is_currently_muted = member.roles.iter().any(|r| r.0 == mute_role_id);

    let result =
        if let Err(add_role_result) = member.add_role(&ctx.discord().http, mute_role_id).await {
            Some(Error::from(add_role_result))
        } else {
            // accumulate all roles to take from the member
            let removed_roles = member
                .roles
                .iter()
                .filter(|r| take.contains(&r.0))
                .map(|r| r.to_string())
                .collect::<Vec<_>>();
            // take them from the member, get remaining roles
            let remaining_roles = member
                .remove_roles(
                    &ctx.discord().http,
                    &take.iter().map(|&r| RoleId::from(r)).collect::<Vec<_>>(),
                )
                .await;

            if let Err(remove_role_result) = remaining_roles {
                Some(Error::from(remove_role_result))
            } else {
                // Roles which were removed from the user
                let updated: Document = Muted {
                    guild_id: Some(member.guild_id.0.to_string()),
                    expires: Some(unmute_time.timestamp() as u64),
                    reason: Some(reason.clone()),
                    taken_roles: if is_currently_muted {
                        // Prevent the bot from overriding the "take" field.
                        // This would happen otherwise, because the bot would accumulate the users roles and then override the value in the database
                        // resulting in the user being muted to have no roles to add back later.
                        None
                    } else {
                        Some(removed_roles)
                    },
                    ..Default::default()
                }
                .into();

                if let Err(database_update_result) = data
                    .database
                    .update::<Muted>(
                        "muted",
                        Muted {
                            user_id: Some(member.user.id.0.to_string()),
                            ..Default::default()
                        }
                        .into(),
                        UpdateModifications::Document(doc! { "$set": updated}),
                        Some(UpdateOptions::builder().upsert(true).build()),
                    )
                    .await
                {
                    Some(database_update_result)
                } else {
                    None
                }
            }
        };

    if let Some(pending_unmute) = data.pending_unmutes.get(&member.user.id.0) {
        trace!("Cancelling pending unmute for {}", member.user.id.0);
        pending_unmute.abort();
    }

    data.pending_unmutes.insert(
        member.user.id.0,
        queue_unmute_member(
            &ctx.discord().http,
            &data.database,
            &member,
            mute_role_id,
            mute_duration.num_seconds() as u64,
        ),
    );

    if result.is_none() {
        if let Err(e) = member.disconnect_from_voice(&ctx.discord().http).await {
            warn!("Could not disconnect member from voice channel: {}", e);
        }
    }

    respond_moderation(
        &ctx,
        &ModerationKind::Mute(
            member.user,
            reason,
            format!("<t:{}:F>", unmute_time.timestamp()),
            result,
        ),
        configuration,
    )
    .await
}

/// Delete recent messages of a user. Cannot delete messages older than 14 days.
#[poise::command(slash_command)]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "User"] user: Option<User>,
    #[description = "Until message"] until: Option<String>,
    #[min = 1]
    #[max = 1000]
    #[description = "Count"]
    count: Option<i64>,
) -> Result<(), Error> {
    // The maximum amount of times to page through messages. If paged over MAX_PAGES amount of times without deleting messages, break.
    const MAX_PAGES: i8 = 2;
    // The maximal amount of messages that we can fetch at all
    const MAX_BULK_DELETE: i64 = 100;
    // Discord does not let us bulk-delete messages older than 14 days
    const MAX_BULK_DELETE_AGO_SECS: i64 = 60 * 60 * 24 * 14;

    let data = ctx.data().read().await;
    let configuration = &data.configuration;
    let embed_color = configuration.general.embed_color;
    let channel = ctx.channel_id();
    let too_old_timestamp = Utc::now().timestamp() - MAX_BULK_DELETE_AGO_SECS;

    let current_user = ctx.discord().http.get_current_user().await?;
    let image = current_user.face();

    let handle = ctx
        .send(|f| {
            f.embed(|f| {
                f.title("Purging messages")
                    .description("Accumulating...")
                    .color(embed_color)
                    .thumbnail(&image)
            })
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
            .messages(&ctx.discord(), |m| {
                m.limit(count_to_delete as u64).before(response.id)
            })
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
                    .take_while(|m| m.id.0 > message_id)
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
            channel.delete_messages(&ctx.discord(), &messages).await?;
        } else {
            empty_pages += 1;
        }

        if empty_pages >= MAX_PAGES || deleted_amount >= count_to_delete {
            break;
        }
    }

    response
        .to_mut()
        .edit(&ctx.discord(), |e| {
            e.set_embed(
                serenity::CreateEmbed::default()
                    .title("Purge successful")
                    .field("Deleted messages", deleted_amount.to_string(), false)
                    .color(embed_color)
                    .thumbnail(image)
                    .clone(),
            )
        })
        .await?;
    Ok(())
}

/// Ban a user.
#[poise::command(slash_command)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User"] user: User,
    #[description = "Amount of days to delete messages"] dmd: Option<u8>,
    #[description = "Reason for the ban"] reason: Option<String>,
) -> Result<(), Error> {
    handle_ban(&ctx, &BanKind::Ban(user, dmd, reason)).await
}

/// Unban a user.
#[poise::command(slash_command)]
pub async fn unban(ctx: Context<'_>, #[description = "User"] user: User) -> Result<(), Error> {
    handle_ban(&ctx, &BanKind::Unban(user)).await
}

async fn handle_ban(ctx: &Context<'_>, kind: &BanKind) -> Result<(), Error> {
    let data = ctx.data().read().await;

    let ban_result = ban_moderation(ctx, kind).await;

    respond_moderation(
        ctx,
        &match kind {
            BanKind::Ban(user, _, reason) => {
                ModerationKind::Ban(user.clone(), reason.clone(), ban_result)
            },
            BanKind::Unban(user) => ModerationKind::Unban(user.clone(), ban_result),
        },
        &data.configuration,
    )
    .await
}
