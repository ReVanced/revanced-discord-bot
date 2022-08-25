use super::*;
use crate::utils::decancer::cure;

pub async fn guild_member_addition(ctx: &serenity::Context, new_member: &serenity::Member) {
    cure(ctx, &None, new_member).await;
}
