use std::cmp;
use std::sync::Arc;

use mongodb::options::FindOptions;
use poise::serenity_prelude::{ChannelId, GuildChannel, GuildId, Mentionable, User, UserId};
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

use super::bot::get_data_lock;
use super::*;
use crate::db::database::Database;
use crate::db::model::Muted;
use crate::model::application::{Configuration, Mute};
use crate::serenity::SerenityError;
use crate::utils::bot::get_member;
use crate::{Context, Error};

pub enum ModerationKind {
    Mute(User, User, String, Option<String>, Option<Error>), /* User, Command author, Reason, Expires, Error */
    Unmute(User, User, Option<Error>),                       // User, Command author, Error
    Ban(User, User, Option<String>, Option<SerenityError>),  // User, Command author, Reason, Error
    Unban(User, User, Option<SerenityError>),                // User, Command author, Error
    Lock(GuildChannel, User, Option<Error>),                 // Channel name, Command author, Error
    Unlock(GuildChannel, User, Option<Error>),               // Channel name, Command author, Error
}
pub enum BanKind {
    Ban(User, Option<u8>, Option<String>), // User, Amount of days to delete messages, Reason
    Unban(User),                           // User
}
pub async fn mute_on_join(ctx: &serenity::Context, new_member: &mut serenity::Member) {
    let data = get_data_lock(ctx).await;
    let data = data.read().await;

    if let Ok(mut cursor) = data
        .database
        .find::<Muted>(
            "muted",
            Muted {
                user_id: Some(new_member.user.id.to_string()),
                ..Default::default()
            }
            .into(),
            Some(FindOptions::builder().limit(1).build()),
        )
        .await
    {
        if let Ok(found) = cursor.advance().await {
            if found {
                debug!("Muted member {} rejoined the server", new_member.user.tag());
                if new_member
                    .add_role(&ctx.http, RoleId(data.configuration.general.mute.role))
                    .await
                    .is_ok()
                {
                    debug!(
                        "Muted member {} was successfully muted",
                        new_member.user.tag()
                    );
                } else {
                    error!(
                        "Failed to mute member {} after rejoining the server",
                        new_member.user.tag()
                    );
                }
            }
        } else {
            error!("Failed to advance the cursor");
        }
    } else {
        error!("Failed to query database for muted users");
    }
}

pub fn queue_unmute_member(
    ctx: serenity::Context,
    database: Arc<Database>,
    guild_id: GuildId,
    user_id: UserId,
    mute_role_id: u64,
    mute_duration: u64,
) -> JoinHandle<Result<(), Error>> {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(mute_duration)).await;
        let db_result = database
            .find_and_delete::<Muted>(
                "muted",
                Muted {
                    user_id: Some(user_id.0.to_string()),
                    ..Default::default()
                }
                .into(),
                None,
            )
            .await?
            .ok_or("User was not muted.")?;

        // Update roles if they didn't leave the guild.
        if let Some(mut member) = get_member(&ctx, guild_id, user_id).await? {
            let http = &ctx.http;

            if let Some(taken_roles) = db_result.taken_roles {
                member
                    .add_roles(
                        http,
                        &taken_roles
                            .into_iter()
                            .map(|r| RoleId::from(r.parse::<u64>().unwrap()))
                            .collect::<Vec<_>>(),
                    )
                    .await?;
            }
            member.remove_role(http, mute_role_id).await?;
        }
        Ok(())
    })
}

// TODO: refactor
pub async fn respond_moderation<'a>(
    ctx: &Context<'_>,
    moderation: &ModerationKind,
    configuration: &Configuration,
) -> Result<(), Error> {
    let current_user = ctx.serenity_context().http.get_current_user().await?;

    let create_embed = |f: &mut serenity::CreateEmbed| {
        let mut moderated_user: Option<&User> = None;

        let result = match moderation {
            ModerationKind::Mute(user, author, reason, expires, error) => {
                moderated_user = Some(user);

                let embed = match error {
                    Some(err) => f
                        .title(format!("Failed to mute {}", user.tag()))
                        .field("Exception", err.to_string(), false)
                        .field(
                            "Action",
                            format!(
                                "{} was muted by {} but failed",
                                user.mention(),
                                author.mention()
                            ),
                            false,
                        ),
                    None => f.title(format!("Muted {}", user.tag())).field(
                        "Action",
                        format!("{} was muted by {}", user.mention(), author.mention()),
                        false,
                    ),
                }
                .field("Reason", reason, true);

                // add expiration date to embed if mute has a duration
                if let Some(expire) = expires {
                    embed.field("Expires", expire, true);
                }
                embed
            },
            ModerationKind::Unmute(user, author, error) => {
                moderated_user = Some(user);
                match error {
                    Some(err) => f
                        .title(format!("Failed to unmute {}", user.tag()))
                        .field("Exception", err.to_string(), false)
                        .field(
                            "Action",
                            format!(
                                "{} was unmuted by {} but failed",
                                user.mention(),
                                author.mention()
                            ),
                            false,
                        ),
                    None => f.title(format!("Unmuted {}", user.tag())).field(
                        "Action",
                        format!("{} was unmuted by {}", user.mention(), author.mention()),
                        false,
                    ),
                }
            },
            ModerationKind::Ban(user, author, reason, error) => {
                moderated_user = Some(user);
                let f = match error {
                    Some(err) => f
                        .title(format!("Failed to ban {}", user.tag()))
                        .field("Exception", err.to_string(), false)
                        .field(
                            "Action",
                            format!(
                                "{} was banned by {} but failed",
                                user.mention(),
                                author.mention()
                            ),
                            false,
                        ),
                    None => f.title(format!("Banned {}", user.tag())).field(
                        "Action",
                        format!("{} was banned by {}", user.mention(), author.mention()),
                        false,
                    ),
                };
                if let Some(reason) = reason {
                    f.field("Reason", reason, true)
                } else {
                    f
                }
            },
            ModerationKind::Unban(user, author, error) => {
                moderated_user = Some(user);
                match error {
                    Some(err) => f
                        .title(format!("Failed to unban {}", user.tag()))
                        .field("Exception", err.to_string(), false)
                        .field(
                            "Action",
                            format!(
                                "{} was unbanned by {} but failed",
                                user.mention(),
                                author.mention()
                            ),
                            false,
                        ),
                    None => f.title(format!("Unbanned {}", user.tag())).field(
                        "Action",
                        format!("{} was unbanned by {}", user.mention(), author.mention()),
                        false,
                    ),
                }
            },
            ModerationKind::Lock(channel, author, error) => match error {
                Some(err) => f
                    .title(format!("Failed to lock {} ", channel.name()))
                    .field("Exception", err.to_string(), false)
                    .field(
                        "Action",
                        format!(
                            "{} was locked by {} but failed",
                            channel.mention(),
                            author.mention()
                        ),
                        false,
                    ),
                None => f
                    .title(format!("Locked {}", channel.name()))
                    .description(
                        "Unlocking the channel will restore the original permission overwrites.",
                    )
                    .field(
                        "Action",
                        format!("{} was locked by {}", channel.mention(), author.mention()),
                        false,
                    ),
            },
            ModerationKind::Unlock(channel, author, error) => match error {
                Some(err) => f
                    .title(format!("Failed to unlock {}", channel.name()))
                    .field("Exception", err.to_string(), false)
                    .field(
                        "Action",
                        format!(
                            "{} was unlocked by {} but failed",
                            channel.mention(),
                            author.mention()
                        ),
                        false,
                    ),
                None => f
                    .title(format!("Unlocked {}", channel.name()))
                    .description("Restored original permission overwrites.")
                    .field(
                        "Action",
                        format!("{} was unlocked by {}", channel.mention(), author.mention()),
                        false,
                    ),
            },
        }
        .color(configuration.general.embed_color);

        let user = if let Some(user) = moderated_user {
            user.face()
        } else {
            current_user.face()
        };

        result.thumbnail(&user).footer(|f| {
            f.text("ReVanced");
            f.icon_url(current_user.face())
        });
    };

    let reply = ctx
        .send(|reply| {
            reply.embed(|embed| {
                create_embed(embed);
                embed
            })
        })
        .await?;

    let response = reply.message().await?;
    ChannelId(configuration.general.logging_channel)
        .send_message(&ctx.serenity_context().http, |reply| {
            reply.embed(|embed| {
                create_embed(embed);
                embed.field(
                    "Reference",
                    format!(
                        "[Jump to message](https://discord.com/channels/{}/{}/{})",
                        ctx.guild_id().unwrap().0,
                        response.channel_id,
                        response.id
                    ),
                    true,
                )
            })
        })
        .await?;

    Ok(())
}

pub async fn ban_moderation(ctx: &Context<'_>, kind: &BanKind) -> Option<SerenityError> {
    let guild_id = ctx.guild_id().unwrap().0;

    let http = &ctx.serenity_context().http;

    match kind {
        BanKind::Ban(user, dmd, reason) => {
            let reason = reason
                .clone()
                .or_else(|| Some("None specified".to_string()))
                .unwrap();

            let ban_result = http
                .ban_user(
                    guild_id,
                    user.id.0,
                    cmp::min(dmd.unwrap_or(0), 7),
                    reason.as_ref(),
                )
                .await;

            if let Err(err) = ban_result {
                error!("Failed to ban user {}: {}", user.id.0, err);
                Some(err)
            } else {
                None
            }
        },
        BanKind::Unban(user) => {
            let unban_result = http.remove_ban(guild_id, user.id.0, None).await;

            if let Err(err) = unban_result {
                error!("Failed to unban user {}: {}", user.id.0, err);
                Some(err)
            } else {
                None
            }
        },
    }
}

pub async fn mute_moderation(
    ctx: &Context<'_>,
    member: &mut Member,
    config: &Mute,
) -> Result<(bool, Vec<serenity::RoleId>), SerenityError> {
    let mute_role_id = config.role;
    let take = &config.take;

    let is_currently_muted = member.roles.iter().any(|r| r.0 == mute_role_id);

    member
        .add_role(&ctx.serenity_context().http, mute_role_id)
        .await?;

    // accumulate all roles to take from the member
    let removed_roles = member
        .roles
        .iter()
        .filter(|r| take.contains(&r.0))
        .copied()
        .collect::<Vec<_>>();
    // take them from the member.
    member
        .remove_roles(
            &ctx.serenity_context().http,
            &take.iter().map(|&r| RoleId::from(r)).collect::<Vec<_>>(),
        )
        .await?;

    if let Err(e) = member
        .disconnect_from_voice(&ctx.serenity_context().http)
        .await
    {
        warn!("Could not disconnect member from voice channel: {}", e);
    }

    Ok((is_currently_muted, removed_roles))
}

