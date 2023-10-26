extern crate decancer;

use poise::serenity_prelude::EditMember;
use tracing::{error, info, trace};

use super::*;

pub async fn cure(
    ctx: &serenity::Context,
    old_if_available: &Option<serenity::Member>,
    member: &serenity::Member,
) {
    if member.user.bot {
        trace!("Skipping decancer for bot {}.", member.user.tag());
        return;
    }

    let name = member.display_name().to_string();

    if let Some(old) = old_if_available {
        if *old.display_name() == name {
            trace!(
                "Skipping decancer for {} because their name hasn't changed",
                member.user.tag()
            );
            return;
        }
    }

    let mut cured_name = decancer::cure(&name).replace(
        |c: char| !(c == ' ' || c == '-' || c == '_' || c.is_ascii_alphanumeric()),
        "",
    );

    if cured_name.is_empty() || !cured_name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        cured_name = "ReVanced member".to_string();
    }

    if name.to_lowercase() == cured_name {
        return; // username is already cured
    }

    match member
        .guild_id
        .edit_member(
            &ctx.http,
            member.user.id,
            EditMember::default().nickname(cured_name),
        )
        .await
    {
        Ok(_) => info!("Cured user {}", member.user.tag()),
        Err(err) => error!("Failed to cure user {}: {}", name, err),
    }
}
