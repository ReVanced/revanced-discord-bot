use super::*;
use crate::utils::cure_names::cure;
use crate::utils::moderation::mute_on_join;

pub async fn guild_member_addition(ctx: &serenity::Context, new_member: &mut serenity::Member) {
    mute_on_join(ctx, new_member).await;

    cure(ctx, &None, new_member).await;
}
