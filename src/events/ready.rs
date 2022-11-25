use crate::serenity::RoleId;
use crate::utils::role_embed::{get_role_name_from_id, update_role_embed};
use chrono::Utc;
use tracing::trace;

use super::*;
use crate::db::model::Muted;
use crate::utils::bot::get_data_lock;
use crate::utils::moderation::queue_unmute_member;

use tracing::log::{error, warn};

use futures_util::StreamExt;

pub async fn load_muted_members(ctx: &serenity::Context, _: &serenity::Ready) {
    let data = get_data_lock(ctx).await;
    let data = &mut *data.write().await;
    let mute_role_id = data.configuration.general.mute.role;

    let mut cursor = data
        .database
        .find::<Muted>(
            "muted",
            Muted {
                ..Default::default()
            }
            .into(),
            None,
        )
        .await
        .unwrap();

    let http_ref = &ctx.http;

    while cursor.advance().await.unwrap() {
        let current: Muted = cursor.deserialize_current().unwrap();
        let guild_id = current.guild_id.unwrap().parse::<u64>().unwrap();
        let member_id = current.user_id.unwrap().parse::<u64>().unwrap();

        if let Ok(guild) = http_ref.get_guild(guild_id).await {
            if let Ok(member) = guild.member(http_ref, member_id).await {
                let amount_left =
                    std::cmp::max(current.expires.unwrap() as i64 - Utc::now().timestamp(), 0);

                data.pending_unmutes.insert(
                    member.user.id.0,
                    queue_unmute_member(
                        &ctx.http,
                        &data.database,
                        &member,
                        mute_role_id,
                        amount_left as u64, // i64 as u64 is handled properly here
                    ),
                );
            } else {
                trace!("Failed to find member {} in guild {}", member_id, guild_id);
            }
        } else {
            trace!("Guild {} unavailable", guild_id);
        }
    }
}

/// Creates/updates the role embed and starts collecting interactions.
pub async fn prepare_role_embed(ctx: serenity::Context) -> Result<(), serenity::Error> {
    let data = get_data_lock(&ctx).await;
    let data = &mut *data.write().await;

    update_role_embed(&ctx, data).await?;

    tokio::task::spawn(async move {
        let mut stream = serenity::ComponentInteractionCollectorBuilder::new(&ctx).build();

        while let Some(interaction) = stream.next().await {
            // Get config data again (might have been reloaded).
            let data = get_data_lock(&ctx).await;
            let data = data.read().await;

            // `ComponentInteractionCollectorBuilder.filter()` does indeed exist, but tokio will explode if we try to lock the data using blocking calls inside a synchronous function called from an async context.
            // The other filtering functions cannot take configuration updates into account.

            if Some(interaction.message.id) != data.role_embed_msg_id {
                continue;
            }

            let Ok(role_id) = interaction.data.custom_id.parse() else {
                continue;
            };

            // Not sure if this check is actually necessary...
            if !data
                .configuration
                .role_embed
                .roles
                .iter()
                .any(|entry| entry.id == role_id && entry.obtainable)
            {
                continue;
            }

            // Anything that wants a write lock on `data` will break unless we do this because the lock won't be dropped until the next loop iteration.
            drop(data);

            if let Err(err) = handle_role(&ctx, &interaction, role_id).await {
                error!(
                    "Could not update the roles of user with id {}: {}",
                    interaction.user.id, err
                );
            }
        }

        warn!("The role embed interactions collector stopped!");
    });

    Ok(())
}

async fn handle_role(
    ctx: &serenity::Context,
    interaction: &serenity::MessageComponentInteraction,
    role_id: u64,
) -> Result<(), Error> {
    let mut member = interaction.member.as_ref().unwrap().clone();
    let has_role = member.roles.contains(&RoleId(role_id));
    if has_role {
        member.remove_role(&ctx.http, role_id).await?;
    } else {
        member.add_role(&ctx.http, role_id).await?;
    }

    let action_str = if has_role { "Removed" } else { "Added" };

    interaction
        .create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|msg| {
                msg.ephemeral(true).embed(|e| {
                    e.description(format!(
                        "{} `{}`",
                        action_str,
                        get_role_name_from_id(ctx, role_id, false)
                    ))
                })
            })
        })
        .await?;
    Ok(())
}
