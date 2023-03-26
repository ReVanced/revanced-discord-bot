use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use api::client::Api;
use commands::{configuration, misc, moderation};
use db::database::Database;
use events::Handler;
use poise::serenity_prelude::{self as serenity, RwLock, UserId};
use tokio::task::JoinHandle;
use tracing::{error, trace};
use utils::bot::load_configuration;

use crate::model::application::Configuration;

mod api;
mod commands;
mod db;
mod events;
mod logger;
mod model;
mod utils;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<RwLock<Data>>, Error>;

impl serenity::TypeMapKey for Data {
    type Value = Arc<RwLock<Data>>;
}

pub struct Data {
    configuration: Configuration,
    database: Arc<Database>,
    pending_unmutes: HashMap<u64, JoinHandle<Result<(), Error>>>,
    poll_secret: String,
    api: Api,
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
        moderation::mute(),
        moderation::unmute(),
        moderation::purge(),
        moderation::ban(),
        moderation::unban(),
        moderation::lock(),
        moderation::unlock(),
        misc::reply(),
        misc::poll(),
    ];
    poise::set_qualified_names(&mut commands);

    let configuration = load_configuration();

    let owners = configuration
        .administrators
        .users
        .iter()
        .cloned()
        .map(UserId)
        .collect::<Vec<UserId>>()
        .into_iter()
        .collect();

    let data = Arc::new(RwLock::new(Data {
        configuration,
        database: Arc::new(
            Database::new(
                &env::var("MONGODB_URI").expect("MONGODB_URI environment variable not set"),
                "revanced_discord_bot",
            )
            .await
            .unwrap(),
        ),
        pending_unmutes: HashMap::new(),
        poll_secret: env::var("POLL_SECRET").expect("POLL_SECRET environment variable not set"),
        api: Api::new(
            reqwest::Url::parse(
                &env::var("API_SERVER").expect("API_SERVER environment variable not set"),
            )
            .expect("Invalid API_SERVER"),
            env::var("API_CLIENT_ID").expect("API_CLIENT_ID environment variable not set"),
            env::var("API_CLIENT_SECRET").expect("API_CLIENT_SECRET environment variable not set"),
        ),
    }));

    let handler = Arc::new(Handler::new(
        poise::FrameworkOptions {
            owners,
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
                        let data_lock = &ctx.data().read().await;
                        let configuration = &data_lock.configuration;
                        let administrators = &configuration.administrators;

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
                            if let Err(e) = ctx
                                .send(|m| {
                                    m.ephemeral(true).embed(|e| {
                                        e.title("Permission error")
                                            .description(
                                                "You do not have permission to use this command.",
                                            )
                                            .color(configuration.general.embed_color)
                                            .thumbnail(member.user.avatar_url().unwrap_or_else(
                                                || member.user.default_avatar_url(),
                                            ))
                                    })
                                })
                                .await
                            {
                                error!("Error sending message: {:?}", e)
                            }
                            trace!("{} is not an administrator.", member.user.name);
                            return Ok(false); // Not an administrator, don't allow command execution
                        }
                    }
                    Ok(true)
                })
            }),
            event_handler: |_ctx, event, _framework, _data| {
                Box::pin(async move {
                    tracing::trace!("{:?}", event.name());
                    Ok(())
                })
            },
            ..Default::default()
        },
        data.clone(), // Pass configuration as user data for the framework
    ));

    let mut client = serenity::Client::builder(
        env::var("DISCORD_AUTHORIZATION_TOKEN")
            .expect("DISCORD_AUTHORIZATION_TOKEN environment variable not set"),
        serenity::GatewayIntents::non_privileged()
            | serenity::GatewayIntents::MESSAGE_CONTENT
            | serenity::GatewayIntents::GUILD_MEMBERS,
    )
    .event_handler_arc(handler.clone())
    .await
    .unwrap();

    client.data.write().await.insert::<Data>(data);

    handler
        .set_shard_manager(client.shard_manager.clone())
        .await;

    client.start().await.unwrap();
}
