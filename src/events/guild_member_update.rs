use super::*;

pub async fn guild_member_update(
    ctx: &serenity::Context,
    _old_if_available: &Option<serenity::Member>,
    new: &serenity::Member,
) {
    crate::utils::cure(ctx, new).await;
}