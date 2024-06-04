use poise::CreateReply;
use tracing::debug;

use crate::utils::bot::load_configuration;
use crate::utils::create_default_embed;
use crate::{Context, Error};

/// Reload the Discord bot.
#[poise::command(slash_command)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    // Update the configuration.
    let configuration = load_configuration();
    let embed = create_default_embed(&configuration);
    ctx.data().write().await.configuration = configuration;

    debug!("{} reloaded the configuration.", ctx.author().name);

    ctx.send(CreateReply {
        embeds: vec![embed.description("Reloading configuration...")],
        ephemeral: Some(true),
        ..Default::default()
    })
    .await?;

    Ok(())
}

/// Stop the Discord bot.
#[poise::command(slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    debug!("{} stopped the bot.", ctx.author().name);

    let configuration = &ctx.data().read().await.configuration;

    ctx.send(CreateReply {
        ephemeral: Some(true),
        embeds: vec![create_default_embed(configuration).description("Stopping the bot...")],
        ..Default::default()
    })
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
