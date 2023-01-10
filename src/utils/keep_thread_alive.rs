use bson::Document;
use tracing::error;

use crate::db::model::KeepAliveThread;

use crate::serenity;

use super::bot::get_data_lock;

pub async fn handle_keep_thread_alive(ctx: &serenity::Context, thread: &serenity::GuildChannel) {
    let data = get_data_lock(ctx).await;
    let data = &mut *data.write().await;
    let database = &data.database;
    let query: Document = KeepAliveThread {
        thread_id: Some(thread.id.to_string()),
        ..Default::default()
    }
    .into();
    if let Ok(Some(_)) = database.find_one::<KeepAliveThread>("keep_alive", query, None)
        .await
    {
        if let Err(why) = thread.edit_thread(&ctx, |t| t.archived(false)).await {
            error!("Error unarchiving thread: {:?}", why);
        }
    }
}
