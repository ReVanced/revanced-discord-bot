use super::*;
use crate::utils::decancer::cure;

pub async fn guild_member_update(
    ctx: &serenity::Context,
    _old_if_available: &Option<serenity::Member>,
    new: &serenity::Member,
) {
    cure(ctx, new).await;
}
