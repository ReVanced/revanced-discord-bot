use chrono::Utc;

use super::*;
use crate::db::model::Muted;
use crate::utils::bot::get_data_lock;
use crate::utils::moderation::queue_unmute_member;

pub async fn load_muted_members(ctx: &serenity::Context, _: &serenity::Ready) {
    let data = get_data_lock(ctx).await;
    let data = &mut *data.write().await;

    let mut cursor = data
        .database
        .find::<Muted>(
            "muted",
            Muted {
                ..Default::default()
            }
            .into(),
            None,
        )
        .await
        .unwrap();

    while cursor.advance().await.unwrap() {
        let current: Muted = cursor.deserialize_current().unwrap();
        let Some(expires) = current.expires else { continue };
        let guild_id = current.guild_id.unwrap().parse::<u64>().unwrap();
        let user_id = current.user_id.unwrap().parse::<u64>().unwrap();

        let amount_left = std::cmp::max(expires as i64 - Utc::now().timestamp(), 0);

        data.pending_unmutes.insert(
            user_id,
            queue_unmute_member(
                ctx.clone(),
                data.database.clone(),
                serenity::GuildId(guild_id),
                serenity::UserId(user_id),
                data.configuration.general.mute.role,
                amount_left as u64, // i64 as u64 is handled properly here
            ),
        );
    }
}
