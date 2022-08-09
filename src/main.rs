use std::env;
use std::sync::Arc;

use commands::configuration;
use events::Handler;
use poise::serenity_prelude::{self as serenity, RwLock};
use utils::load_configuration;

use crate::model::application::Configuration;

mod commands;
mod events;
mod logger;
mod model;
mod utils;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<RwLock<Configuration>>, Error>;

impl serenity::TypeMapKey for Configuration {
    type Value = Arc<RwLock<Configuration>>;
}

#[tokio::main]
async fn main() {
    // Initialize the logging framework
    logger::init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Define poise framework commands (also in src/commands/mod.rs for serenity framework's manually dispatched events)
    let mut commands = vec![
        configuration::register(),
        configuration::reload(),
        configuration::stop(),
    ];
    poise::set_qualified_names(&mut commands);

    let configuration = Arc::new(RwLock::new(load_configuration()));

    let handler = Arc::new(Handler::new(
        poise::FrameworkOptions {
            commands,
            on_error: |error| {
                Box::pin(async {
                    poise::samples::on_error(error)
                        .await
                        .unwrap_or_else(|error| tracing::error!("{}", error));
                })
            },
            command_check: Some(|ctx| {
                Box::pin(async move {
                    if let Some(member) = ctx.author_member().await {
                        let administrators = &ctx.data().read().await.administrators;

                        if !(administrators
                            .users
                            // Check if the user is an administrator
                            .contains(&member.user.id.0)
                            || administrators
                                .roles
                                .iter()
                                // Has one of the administative roles
                                .any(|&role_id| {
                                    member
                                        .roles
                                        .iter()
                                        .any(|member_role| member_role.0 == role_id)
                                }))
                        {
                            return Ok(false); // Not an administrator, don't allow command execution
                        }
                    }
                    Ok(true)
                })
            }),
            listener: |_ctx, event, _framework, _data| {
                Box::pin(async move {
                    tracing::trace!("{:?}", event.name());
                    Ok(())
                })
            },
            ..Default::default()
        },
        configuration.clone(), // Pass configuration as user data for the framework
    ));

    let mut client = serenity::Client::builder(
        env::var("DISCORD_AUTHORIZATION_TOKEN")
            .expect("Could not load Discord authorization token"),
        serenity::GatewayIntents::non_privileged()
            | serenity::GatewayIntents::MESSAGE_CONTENT
            | serenity::GatewayIntents::GUILD_MEMBERS,
    )
    .event_handler_arc(handler.clone())
    .await
    .unwrap();

    client
        .data
        .write()
        .await
        .insert::<Configuration>(configuration);

    handler
        .set_shard_manager(client.shard_manager.clone())
        .await;

    client.start().await.unwrap();
}
