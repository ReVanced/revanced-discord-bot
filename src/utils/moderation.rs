use std::cmp;
use std::sync::Arc;

use mongodb::options::FindOptions;
use poise::serenity_prelude::{ChannelId, Http, User};
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use super::bot::get_data_lock;
use super::*;
use crate::db::database::Database;
use crate::db::model::Muted;
use crate::model::application::Configuration;
use crate::serenity::SerenityError;
use crate::{Context, Error};

pub enum ModerationKind {
    Mute(String, String, Option<Error>), // Reason, Expires, Error
    Unmute(Option<Error>),               // Error
    Ban(Option<String>, Option<SerenityError>), // Reason, Error
    Unban(Option<SerenityError>),        // Error
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
        if cursor.advance().await.is_ok() {
            trace!("Muted member {} rejoined the server", new_member.user.tag());
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
        error!("Failed to query database for muted users");
    }
}

pub fn queue_unmute_member(
    http: &Arc<Http>,
    database: &Arc<Database>,
    member: &Member,
    mute_role_id: u64,
    mute_duration: u64,
) -> JoinHandle<Option<Error>> {
    let http = http.clone();
    let database = database.clone();
    let mut member = member.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(mute_duration)).await;

        let delete_result = database
            .find_and_delete::<Muted>(
                "muted",
                Muted {
                    user_id: Some(member.user.id.0.to_string()),
                    ..Default::default()
                }
                .into(),
                None,
            )
            .await;

        if let Err(database_remove_result) = delete_result {
            Some(database_remove_result)
        } else if let Some(find_result) = delete_result.unwrap() {
            let taken_roles = find_result
                .taken_roles
                .unwrap()
                .into_iter()
                .map(|r| RoleId::from(r.parse::<u64>().unwrap()))
                .collect::<Vec<_>>();

            if let Err(add_role_result) = member.add_roles(&http, &taken_roles).await {
                Some(Error::from(add_role_result))
            } else if let Err(remove_result) = member.remove_role(http, mute_role_id).await {
                Some(Error::from(remove_result))
            } else {
                None
            }
        } else {
            None
        }
    })
}

// TODO: refactor
pub async fn respond_moderation<'a>(
    ctx: &Context<'_>,
    moderation: &ModerationKind,
    user: &serenity::User,
    configuration: &Configuration,
) -> Result<(), Error> {
    let create_embed = |f: &mut serenity::CreateEmbed| {
        let tag = user.tag();
        match moderation {
            ModerationKind::Mute(reason, expires, error) => match error {
                Some(err) => f.title(format!("Failed to mute {}", tag)).field(
                    "Exception",
                    err.to_string(),
                    false,
                ),
                None => f.title(format!("Muted {}", tag)),
            }
            .field("Reason", reason, false)
            .field("Expires", expires, false),
            ModerationKind::Unmute(error) => match error {
                Some(err) => f.title(format!("Failed to unmute {}", tag)).field(
                    "Exception",
                    err.to_string(),
                    false,
                ),
                None => f.title(format!("Unmuted {}", tag)),
            },
            ModerationKind::Ban(reason, error) => {
                let f = match error {
                    Some(err) => f.title(format!("Failed to ban {}", tag)).field(
                        "Exception",
                        err.to_string(),
                        false,
                    ),
                    None => f.title(format!("Banned {}", tag)),
                };
                if let Some(reason) = reason {
                    f.field("Reason", reason, false)
                } else {
                    f
                }
            },
            ModerationKind::Unban(error) => match error {
                Some(err) => f.title(format!("Failed to unban {}", tag)).field(
                    "Exception",
                    err.to_string(),
                    false,
                ),
                None => f.title(format!("Unbanned {}", tag)),
            },
        }
        .color(configuration.general.embed_color)
        .thumbnail(
            &user
                .avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
        );
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
        .send_message(&ctx.discord().http, |reply| {
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
                    false,
                )
            })
        })
        .await?;

    Ok(())
}

pub async fn ban_moderation(ctx: &Context<'_>, kind: &BanKind) -> Option<SerenityError> {
    let guild_id = ctx.guild_id().unwrap().0;
    let http = &ctx.discord().http;

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
