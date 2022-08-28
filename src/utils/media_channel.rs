use tracing::error;

use super::bot::get_data_lock;
use super::*;

pub async fn handle_media_channel(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
) -> bool {
    let current_channel = new_message.channel_id.0;

    let data_lock = get_data_lock(ctx).await;

    let configuration = &data_lock.read().await.configuration;

    let is_media_channel = configuration
        .general
        .media_channels
        .iter()
        .any(|&channel| channel == current_channel);

    if !configuration
        .administrators
        .users
        .contains(&new_message.author.id.0)
        && is_media_channel
    {
        if let Err(why) = new_message.delete(&ctx.http).await {
            error!("Error deleting message: {:?}", why);
        }
    }

    is_media_channel
}
