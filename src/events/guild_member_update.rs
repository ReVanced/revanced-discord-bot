use super::*;
use crate::utils::decancer::cure;

pub async fn guild_member_update(
    ctx: &serenity::Context,
    old_if_available: &Option<Member>,
    new: &Option<Member>,
) {
    if let Some(member) = new {
        cure(ctx, old_if_available, member).await;
    }
}
