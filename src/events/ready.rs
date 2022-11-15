use crate::model::application::RoleInfo;
use crate::serenity::{ChannelId, RoleId};
use chrono::Utc;
use tracing::trace;

use super::*;
use crate::db::model::Muted;
use crate::utils::bot::get_data_lock;
use crate::utils::moderation::queue_unmute_member;

use serenity::model::application::component::ComponentType;
use serenity::model::application::interaction::message_component::{
    MessageComponentInteraction, MessageComponentInteractionData,
};

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

pub async fn role_embed(ctx: &serenity::Context) -> Result<(), serenity::Error> {
    let data = get_data_lock(ctx).await;
    let data = &mut *data.write().await;

    let color = data.configuration.general.embed_color;
    let role_config = data.configuration.role_embed.clone();

    let mut message = {
        // TODO: this should be getting the message id from the database lol
        ChannelId(1040532157061398550)
            .send_message(&ctx.http, |f| f.content("Wait for it!"))
            .await?
    };

    message
        .edit(&ctx.http, |f| {
            f.content("a").components(|c| {
                c.create_action_row(|r| {
                    for role in &role_config[..] {
                        if !role.button {
                            continue;
                        }
                        // im sorry
                        let role = RoleId::from(role.id).to_role_cached(&ctx.cache).unwrap();
                        r.create_button(|btn| {
                            btn.label(role.name).custom_id(format!("role:{}", role.id))
                        });
                    }
                    r
                })
            })
        })
        .await?;

    let task_ctx = ctx.clone();
    let mut stream = message.await_component_interactions(&ctx).build();

    tokio::task::spawn(async move {
        while let Some(interaction) = stream.next().await {
            match &*interaction {
                MessageComponentInteraction {
                    data:
                        MessageComponentInteractionData {
                            component_type: ComponentType::Button,
                            custom_id,
                            ..
                        },
                    ..
                } => {
                    if custom_id.starts_with("role:") {
                        if let Ok(id) = (&custom_id["role:".len()..]).parse() {
                            let _ =
                                handle_role(&task_ctx, &*interaction, id, &role_config[..]).await;
                        }
                    }
                },
                _ => (),
            }
        }
    });

    Ok(())
}

pub async fn handle_role(
    ctx: &serenity::Context,
    interaction: &serenity::MessageComponentInteraction,
    role_id: u64,
    role_embed_config: &[RoleInfo],
) -> Result<(), Error> {
    let mut allowed = false;
    for role in role_embed_config {
        if role.id == role_id {
            allowed = true;
            break;
        }
    }

    if !allowed {
        return Ok(());
    }

    let mut member = interaction.member.as_ref().unwrap().clone();
    if member.roles.contains(&RoleId(role_id)) {
        member.remove_role(&ctx.http, role_id).await?;
    } else {
        member.add_role(&ctx.http, role_id).await?;
    }

    interaction
        .create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|msg| msg.ephemeral(true).content(role_id))
        })
        .await?;
    Ok(())
}
