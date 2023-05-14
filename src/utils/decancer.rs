use censor::Censor;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use tracing::{error, info, trace};

use super::{bot, serenity};

lazy_static! {
    static ref CENSOR: OnceCell<censor::Censor> = OnceCell::new();
}

// Initialize the censor
async fn censor(ctx: &serenity::Context) -> &'static Censor {
    let data_lock = bot::get_data_lock(ctx).await;
    let censor_config = &data_lock.read().await.configuration.general.censor;
    let additions = &censor_config.additions;
    let removals = &censor_config.removals;

    CENSOR.get_or_init(|| {
        let mut censor = censor::Standard;

        for addition in additions {
            censor += addition;
        }

        for removal in removals {
            censor -= removal;
        }

        censor
    })
}

// Reinitialize the censor when the configuration is reloaded
pub async fn reinit_censor(ctx: &serenity::Context) {
    match CENSOR.set(censor(ctx).await.clone()) {
        Ok(_) => info!("Reinitialized censor"),
        Err(_) => error!("Failed to reinitialize censor"),
    }
}


pub async fn cure(
    ctx: &serenity::Context,
    old_if_available: &Option<serenity::Member>,
    member: &serenity::Member,
) {
    if member.user.bot {
        trace!("Skipping decancer for bot {}.", member.user.tag());
        return;
    }
    
    let censor = censor(ctx).await;

    let name = member.display_name().to_string();

    if let Some(old) = old_if_available {
        if old.display_name().to_string() == name {
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

    if cured_name.is_empty()
        || !cured_name.starts_with(|c: char| c.is_ascii_alphabetic())
        || censor.check(&cured_name)
    {
        cured_name = "ReVanced member".to_string();
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
