use super::*;
use crate::utils::cure_names::cure;

pub async fn guild_member_update(
    ctx: &serenity::Context,
    old_if_available: &Option<serenity::Member>,
    new: &serenity::Member,
) {
    cure(ctx, old_if_available, new).await;
}
