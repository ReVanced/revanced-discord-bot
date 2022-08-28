use super::*;
use crate::utils::autorespond::auto_respond;
use crate::utils::media_channel::handle_media_channel;

pub async fn message_create(ctx: &serenity::Context, new_message: &serenity::Message) {
    let is_media_channel = handle_media_channel(ctx, new_message).await;
    if !is_media_channel {
        auto_respond(ctx, new_message).await;
    }
}
