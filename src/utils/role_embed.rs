use crate::serenity::{self, ChannelId, RoleId};

// Panics if the role doesn't exist.
pub fn get_role_name_from_id(ctx: &serenity::Context, id: u64, include_icon: bool) -> String {
    let role = RoleId(id).to_role_cached(&ctx.cache).unwrap();
    match role.icon {
        Some(icon) if include_icon => format!("{} {}", icon, role.name),
        _ => role.name,
    }
}

/// Creates or updates the role embed.
pub async fn update_role_embed(
    ctx: &serenity::Context,
    data: &crate::Data,
) -> Result<serenity::Message, serenity::Error> {
    let color = data.configuration.general.embed_color;
    let thumbnail = ctx.http.get_current_user().await?.face();
    let role_config = &data.configuration.role_embed;

    let mut message = {
        // TODO: this should be getting the message id from the database lol
        ChannelId(1040532157061398550)
            .send_message(&ctx.http, |f| f.embed(|e| e.title("Loading role embed...")))
            .await?
    };

    message
        .edit(&ctx.http, |f| {
            f.embed(|e| {
                e.title("Roles")
                    .color(color)
                    .thumbnail(thumbnail)
                    .description("Click the respective button to toggle the role.")
                    .fields(role_config.iter().map(|role| {
                        (
                            get_role_name_from_id(ctx, role.id, true),
                            role.description.as_str(),
                            false,
                        )
                    }))
            })
            .components(|c| {
                c.create_action_row(|r| {
                    for role in role_config {
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

    Ok(message)
}
