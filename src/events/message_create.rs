use super::*;
use crate::utils::autorespond::auto_respond;
use crate::utils::code_embed::utils::handle_code_url;
use crate::utils::media_channel::handle_media_channel;

pub async fn message_create(ctx: &serenity::Context, new_message: &serenity::Message) {
    let is_media_channel = handle_media_channel(ctx, new_message).await;

    if is_media_channel {
        return;
    };

    auto_respond(ctx, new_message).await;

    handle_code_url(ctx, new_message).await;
}
