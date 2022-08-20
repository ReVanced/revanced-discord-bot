use tracing::{debug, error};

use super::*;
use crate::utils::bot::get_data_lock;

pub async fn thread_create(ctx: &serenity::Context, thread: &serenity::GuildChannel) {
    if thread.member.is_some() {
        debug!("Thread was joined. Block dispatch.");
        return;
    }

    debug!("Thread created: {:?}", thread);

    let data_lock = get_data_lock(ctx).await;
    let configuration_lock = data_lock.read().await;

    let thread_introductions = &configuration_lock
        .configuration
        .read()
        .await
        .thread_introductions;

    if let Some(introducer) = thread_introductions.iter().find(|introducer| {
        introducer
            .channels
            .iter()
            .any(|channel_id| *channel_id == thread.parent_id.unwrap().0)
    }) {
        if let Err(why) = thread
            .say(&ctx.http, &introducer.response.message.as_ref().unwrap())
            .await
        {
            error!("Error sending message: {:?}", why);
        }
    }
}
