use std::sync::Arc;

use poise::serenity_prelude::{self as serenity, RwLock};

use crate::model::application::Configuration;
use crate::Data;

pub fn load_configuration() -> Configuration {
    Configuration::load().expect("Failed to load configuration")
}

// Share the lock reference between the threads in serenity framework
pub async fn get_data_lock(ctx: &serenity::Context) -> Arc<RwLock<Data>> {
    ctx.data
        .read()
        .await
        .get::<Data>()
        .expect("Expected Configuration in TypeMap.")
        .clone()
}
