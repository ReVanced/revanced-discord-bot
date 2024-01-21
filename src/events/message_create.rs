use super::*;
use crate::utils::code_embed::utils::code_preview;
use crate::utils::message_response::handle_message_response;
use crate::BotData;

pub async fn message_create(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    data: &BotData,
) {
    tokio::join!(
        handle_message_response(ctx, new_message, data),
        code_preview(ctx, new_message)
    );
}
