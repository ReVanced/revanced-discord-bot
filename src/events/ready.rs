use crate::model::application::RoleInfo;
use crate::utils::role_embed::{update_role_embed, get_role_name_from_id};
use crate::serenity::{RoleId, model::application::component::ComponentType};
use chrono::Utc;
use tracing::trace;

use super::*;
use crate::db::model::Muted;
use crate::utils::bot::get_data_lock;
use crate::utils::moderation::queue_unmute_member;

use tracing::log::{error, warn};

// TODO: uhh...
use poise::futures_util::StreamExt;

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
pub async fn role_embed_ready(ctx: serenity::Context) -> Result<(), serenity::Error> {
    let data = get_data_lock(&ctx).await;
    let data = &*data.read().await;

    let mut stream = update_role_embed(&ctx, data)
        .await?
        .await_component_interactions(&ctx.shard)
        // Filters away all invalid interactions (except those that don't exist in the configs):
        // Might not be necessary but why not.
        .filter(move |interaction| {
            if interaction.data.component_type != ComponentType::Button {
                return false;
            }

            // Should be a RoleId (u64)
            interaction.data.custom_id.parse::<u64>().is_ok()
        })
        .build();

    tokio::task::spawn(async move {
        while let Some(interaction) = stream.next().await {
            // Get config data again incase we reloaded it.
            let data = get_data_lock(&ctx).await;
            let data = &*data.read().await;

            if let Err(err) = handle_role(
                &ctx,
                &*interaction,
                interaction.data.custom_id.parse().unwrap(),
                &data.configuration.role_embed,
            )
            .await
            {
                error!(
                    "Could not give/take role to/from user {}: {}",
                    interaction.user.id, err
                );
            }
        }
        warn!("Role Embed interactions collector stopped!");
    });

    Ok(())
}

async fn handle_role(
    ctx: &serenity::Context,
    interaction: &serenity::MessageComponentInteraction,
    role_id: u64,
    role_embed_config: &[RoleInfo],
) -> Result<(), Error> {
    // Not sure if this check is actually necessary...
    if role_embed_config
        .iter()
        .find(|entry| entry.id == role_id)
        .is_none()
    {
        // sussy custom id
        return Ok(());
    }

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
