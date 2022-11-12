use poise::serenity_prelude::CreateButton;
use poise::serenity_prelude::model::id::RoleId;

// TODO embeds
use crate::{Context, Error};

/// Role embed
#[poise::command(slash_command)]
pub async fn role_embed(ctx: Context<'_>) -> Result<(), Error> {
    // debug!("{} Ran role embed", ctx.author().name);

    let config = &ctx.data().read().await.configuration;
    let color = config.general.embed_color;
    let roles = &config.role_embed;

    ctx.send(|f| {
        f.content("a").components(|c| {
            c.create_action_row(|r| {
                for role in roles {
                    if !role.button {
                        continue;
                    }
                    // im sorry
                    let role = RoleId::from(role.id).to_role_cached(&ctx.discord().cache).unwrap();
                    r.create_button(|btn| btn.label(role.name).custom_id(format!("role:{}", role.id)));
                }
                r
            })
        })
    })
    .await?;
    Ok(())
}
