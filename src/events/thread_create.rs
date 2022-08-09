use tracing::{debug, error};

use super::*;

pub async fn thread_create(ctx: &serenity::Context, thread: &serenity::GuildChannel) {
    if thread.member.is_some() {
        debug!("Thread was joined. Block dispatch.");
        return;
    }

    debug!("Thread created: {:?}", thread);

    let configuration_lock = get_configuration_lock(&ctx).await;
    let thread_introductions = &configuration_lock.read().await.thread_introductions;

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
