use poise::serenity_prelude::ThreadMetadata;

use crate::utils::keep_thread_alive::handle_keep_thread_alive;

use super::*;

pub async fn thread_update(ctx: &serenity::Context, thread: &serenity::GuildChannel) {
    if matches!(thread.thread_metadata, Some(ThreadMetadata { archived: true, locked: false, .. })) {
        if let Err(why) = handle_keep_thread_alive(ctx, thread).await {
            error!("Error unarchiving thread: {:?}", why);
        }
    }
}