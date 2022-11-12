use super::*;
use serenity::model::application::component::ComponentType;
use serenity::model::interactions::message_component::{
    MessageComponentInteraction, MessageComponentInteractionData,
};

pub async fn handle_role(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    role_id: u64,
    data: &Data,
) -> Result<(), Error> {
    let role_embed_config = &data.configuration.role_embed;

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

    let mut component_interaction = interaction.clone().message_component().unwrap();
    let member = component_interaction.member.as_mut().unwrap();
    if member.roles.contains(&serenity::RoleId(role_id)) {
        member.remove_role(&ctx.http, role_id).await?;
    } else {
        member.add_role(&ctx.http, role_id).await?;
    }

    component_interaction
        .create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|msg| msg.ephemeral(true).content(role_id))
        })
        .await?;
    Ok(())
}

pub async fn interaction(ctx: &serenity::Context, interaction: &serenity::Interaction, data: &Data) {
    match interaction {
        serenity::Interaction::MessageComponent(MessageComponentInteraction {
            data:
                MessageComponentInteractionData {
                    component_type: ComponentType::Button,
                    custom_id,
                    ..
                },
            ..
        }) => {
            if custom_id.starts_with("role:") {
                if let Ok(id) = (&custom_id["role:".len()..]).parse() {
                    let _ = handle_role(ctx, interaction, id, data).await;
                }
            }
        },
        _ => (),
    }
}
