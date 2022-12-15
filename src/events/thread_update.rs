use tracing::error;

use crate::utils::bot::get_data_lock;

use super::*;

pub async fn thread_update(ctx: &serenity::Context, thread: &serenity::GuildChannel) {
    if thread.thread_metadata.is_some_and(|&t| t.archived && !t.locked) {
        let data_lock = get_data_lock(ctx).await;

        let configuration = &data_lock.read().await.configuration;

        let is_auto_unarchive_thread = configuration
            .general
            .auto_unarchive_threads
            .iter()
            .any(|&thread| thread == thread);

        if is_auto_unarchive_thread {
            if let Err(why) = thread.edit_thread(&ctx, |t| t.archived(false)).await {
                error!("Error unarchiving thread: {:?}", why);
            }
        }
    }
}
