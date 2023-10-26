use super::*;
use crate::model::application::Configuration;
use crate::utils::bot::get_data_lock;
use crate::utils::decancer::cure;

pub async fn presence_update(ctx: &serenity::Context, new_data: &Presence) {
    let data = get_data_lock(ctx).await;
    let configuration: &Configuration = &data.read().await.configuration;

    if !configuration.general.cure_on_presence_update {
        return;
    }

    cure(
        ctx,
        &None,
        &new_data
            .guild_id
            .unwrap()
            .member(&ctx.http, new_data.user.id)
            .await
            .unwrap(),
    )
    .await;
}
