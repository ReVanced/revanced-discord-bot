use ::decancer::Decancer;
use tracing::{error, info};

use super::*;

const DECANCER: Decancer = Decancer::new();

pub async fn cure(ctx: &serenity::Context, member: &serenity::Member) {
    let display_name = member.display_name();
    let name = display_name.to_string();

    let cured_user_name = DECANCER.cure(&name);

    if name.to_lowercase() == cured_user_name {
        return; // username is already cured
    }

    match member
        .guild_id
        .edit_member(&ctx.http, member.user.id, |edit_member| {
            edit_member.nickname(cured_user_name)
        })
        .await
    {
        Ok(_) => info!("Cured user {}", member.user.tag()),
        Err(err) => error!("Failed to cure user {}: {}", name, err),
    }
}
