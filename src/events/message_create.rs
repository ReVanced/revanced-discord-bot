use super::*;
use crate::utils::code_embed::utils::code_preview;
use crate::utils::message_response::handle_message_response;

pub async fn message_create(ctx: &serenity::Context, new_message: &serenity::Message) {
    tokio::join!(
        handle_message_response(ctx, new_message),
        code_preview(ctx, new_message)
    );
}
