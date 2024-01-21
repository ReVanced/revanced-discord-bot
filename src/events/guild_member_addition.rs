use super::*;
use crate::utils::decancer::cure;
use crate::utils::moderation::mute_on_join;
use crate::BotData;

pub async fn guild_member_addition(
    ctx: &serenity::Context,
    new_member: &serenity::Member,
    data: &BotData,
) {
    mute_on_join(ctx, new_member, data).await;

    cure(ctx, &None, new_member).await;
}
