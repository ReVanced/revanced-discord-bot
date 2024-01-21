use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use tracing::debug;

use crate::utils::bot::load_configuration;
use crate::{Context, Error};

/// Reload the Discord bot.
#[poise::command(slash_command)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    // Update the configuration
    let configuration = load_configuration();
    // Use the embed color from the updated configuration
    let embed_color = configuration.general.embed_color;
    // Also save the new configuration to the user data
    ctx.data().write().await.configuration = configuration;

    debug!("{} reloaded the configuration.", ctx.author().name);

    ctx.send(
        CreateReply {
            embeds: vec![
                CreateEmbed::new()
                    .description("Reloading configuration...")
                    .color(embed_color),
            ],
            ephemeral: Some(true),
            ..Default::default()
        }
    )
    .await?;

    Ok(())
}

/// Stop the Discord bot.
#[poise::command(slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    debug!("{} stopped the bot.", ctx.author().name);

    let color = ctx.data().read().await.configuration.general.embed_color;

    ctx.send(
        CreateReply {
            ephemeral: Some(true),
            embeds: vec![
                CreateEmbed::new()
                    .description("Stopping the bot...")
                    .color(color),
            ],
            ..Default::default()
        }
    )
    .await?;

    ctx.framework().shard_manager().shutdown_all().await;

    Ok(())
}

/// Register slash commands.
#[poise::command(prefix_command, slash_command, ephemeral = true)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
