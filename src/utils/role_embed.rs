use crate::db::model::SavedRoleEmbed;
use crate::serenity::{self, ChannelId, RoleId};

use bson::{doc, Document};
use mongodb::options::{UpdateModifications, UpdateOptions};
use tracing::{debug, error};

// Panics if the role doesn't exist.
pub fn get_role_name_from_id(ctx: &serenity::Context, id: u64, include_icon: bool) -> String {
    let role = RoleId(id).to_role_cached(&ctx.cache).unwrap();
    match role.icon {
        Some(icon) if include_icon => format!("{} {}", icon, role.name),
        _ => role.name,
    }
}
async fn get_msg_from_db(
    query: Document,
    ctx: &serenity::Context,
    db: &crate::db::database::Database,
    channel_id: ChannelId,
) -> Option<serenity::Message> {
    let message_id = db
        .find_one::<SavedRoleEmbed>("saved_role_embed", query, None)
        .await
        .unwrap_or_else(|e| {
            error!("Error while querying database: {}", e);
            None
        })
        .and_then(|doc| doc.message_id)?;

    match ctx.cache.message(channel_id, message_id) {
        None => ctx.http.get_message(channel_id.0, message_id).await.ok(),
        cached => cached,
    }
}

/// Creates or updates the role embed.
pub async fn update_role_embed(
    ctx: &serenity::Context,
    data: &mut crate::Data,
) -> Result<(), serenity::Error> {
    let color = data.configuration.general.embed_color;
    let thumbnail = ctx.http.get_current_user().await?.face();
    let role_embed = &data.configuration.role_embed;
    let channel_id = ChannelId(role_embed.channel_id);

    let query: Document = SavedRoleEmbed {
        channel_id: Some(channel_id.0),
        ..Default::default()
    }
    .into();

    let mut message =
        if let Some(msg) = get_msg_from_db(query.clone(), ctx, &data.database, channel_id).await {
            msg
        } else {
            // No message in database (or it failed).
            debug!("No saved role embed message found. Sending a new one...");
            let msg = channel_id
                .send_message(&ctx.http, |f| {
                    f.embed(|e| e.color(color).title("Loading role embed..."))
                })
                .await?;
            let update: Document = SavedRoleEmbed {
                message_id: Some(msg.id.0),
                ..Default::default()
            }
            .into();
            if let Err(e) = data
                .database
                .update::<SavedRoleEmbed>(
                    "saved_role_embed",
                    query,
                    UpdateModifications::Document(doc! { "$set": update }),
                    Some(UpdateOptions::builder().upsert(true).build()),
                )
                .await
            {
                error!("Could not add role embed message id to the database: {}", e);
            }
            msg
        };

    message
        .edit(&ctx.http, |f| {
            f.embed(|e| {
                e.title("Roles")
                    .color(color)
                    .thumbnail(thumbnail)
                    .description("Click the respective button to toggle the role.")
                    .fields(role_embed.roles.iter().map(|role| {
                        (
                            get_role_name_from_id(ctx, role.id, true),
                            role.description.as_str(),
                            false,
                        )
                    }))
            })
            .components(|c| {
                c.create_action_row(|r| {
                    for role in &role_embed.roles {
                        if !role.button {
                            continue;
                        }

                        let name = get_role_name_from_id(ctx, role.id, true);
                        r.create_button(|btn| btn.label(name).custom_id(role.id));
                    }
                    r
                })
            })
        })
        .await?;

    data.role_embed_msg_id = Some(message.id);

    Ok(())
}
