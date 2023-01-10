use crate::{utils::keep_thread_alive::handle_keep_thread_alive};


use super::*;

pub async fn thread_update(ctx: &serenity::Context, thread: &serenity::GuildChannel) {
    handle_keep_thread_alive(ctx, thread).await
}