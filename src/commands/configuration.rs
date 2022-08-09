use crate::{utils::load_configuration, Context, Error};
use tracing::debug;

#[poise::command(slash_command, prefix_command)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    // Update the configuration
    let configuration = load_configuration();
    // Use the embed color from the updated configuration
    let embed_color = configuration.general.embed_color;
    // Also save the new configuration to the user data
    *ctx.data().write().await = configuration;

    debug!("{:?} reloaded the configuration.", ctx.author().name);

    ctx.send(|f| {
        f.ephemeral(true).embed(|f| {
            f.description("Successfully reloaded configuration.")
                .color(embed_color)
        })
    })
    .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    debug!("{:?} stopped the bot.", ctx.author().name);

    let color = ctx.data().read().await.general.embed_color;
    ctx.send(|f| {
        f.ephemeral(true)
            .embed(|f| f.description("Stopped the bot.").color(color))
    })
    .await?;

    ctx.discord().shard.shutdown_clean();

    Ok(())
}

#[poise::command(prefix_command, slash_command, ephemeral = true)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
