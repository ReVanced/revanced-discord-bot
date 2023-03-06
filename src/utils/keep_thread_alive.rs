use bson::Document;

use crate::db::model::KeepAliveThread;

use crate::{serenity, Error};

use super::bot::get_data_lock;

pub async fn handle_keep_thread_alive(
    ctx: &serenity::Context,
    thread: &serenity::GuildChannel,
) -> Result<(), Error> {
    let data = get_data_lock(ctx).await;

    let database = &data.read().await.database;

    let query: Document = KeepAliveThread {
        thread_id: Some(thread.id.to_string()),
    }
    .into();
    if (database
        .find_one::<KeepAliveThread>("keep_alive", query, None)
        .await?).is_some()
    {
        thread.edit_thread(&ctx, |t| t.archived(false)).await?;
    }
    Ok(())
}
