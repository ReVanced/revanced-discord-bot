use super::*;

pub async fn guild_member_addition(ctx: &serenity::Context, new_member: &serenity::Member) {
    crate::utils::cure(ctx, new_member).await;
}