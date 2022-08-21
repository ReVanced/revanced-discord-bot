use ::decancer::Decancer;
use tracing::{error, info};

use super::*;

const DECANCER: Decancer = Decancer::new();

pub async fn cure(ctx: &serenity::Context, member: &serenity::Member) {
    println!("Curing {}", member.display_name());
    let name = member.display_name().to_string();

    let mut cured_name = DECANCER
        .cure(&name)
        .replace(|c: char| !(c == ' ' || c.is_ascii_alphanumeric()), "");

    if cured_name.len() == 0 {
        cured_name = "ReVanced user" .to_string();
    }

    if name.to_lowercase() == cured_name {
        return; // username is already cured
    }

    match member
        .guild_id
        .edit_member(&ctx.http, member.user.id, |edit_member| {
            edit_member.nickname(cured_name)
        })
        .await
    {
        Ok(_) => info!("Cured user {}", member.user.tag()),
        Err(err) => error!("Failed to cure user {}: {}", name, err),
    }
}
