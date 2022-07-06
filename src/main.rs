use std::sync::Arc;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use configuration::BotConfiguration;
use log::{error, info, trace, LevelFilter};
use logger::logging::SimpleLogger;
use serenity::client::{Context, EventHandler};
use serenity::model::application::command::Command;
use serenity::model::channel::{GuildChannel, Message};
use serenity::model::gateway::Ready;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::model::Timestamp;
use serenity::prelude::{GatewayIntents, RwLock, TypeMapKey};
use serenity::{async_trait, Client};
mod configuration;
mod logger;

static LOGGER: SimpleLogger = SimpleLogger;

struct Configuration;

impl TypeMapKey for Configuration {
	type Value = Arc<RwLock<BotConfiguration>>;
}

pub struct Handler;

async fn get_configuration_lock(ctx: &Context) -> Arc<RwLock<BotConfiguration>> {
	ctx.data
		.read()
		.await
		.get::<Configuration>()
		.expect("Expected Configuration in TypeMap.")
		.clone()
}

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		trace!("Created an interaction: {:?}", interaction);

		if let Interaction::ApplicationCommand(command) = interaction {
			let content = match command.data.name.as_str() {
				"reload" => {
					trace!("{:?} reloading configuration.", command.user);

					let configuration_lock = get_configuration_lock(&ctx).await;

					let mut configuration = configuration_lock.write().await;

					let new_config =
						BotConfiguration::load().expect("Could not load configuration.");

					configuration.administrators = new_config.administrators;
					configuration.message_responders = new_config.message_responders;
					configuration.thread_introductions = new_config.thread_introductions;

					"Successfully reload configuration.".to_string()
				},
				_ => "Unknown command.".to_string(),
			};

			if let Err(why) = command
				.create_interaction_response(&ctx.http, |response| {
					response
						.kind(InteractionResponseType::ChannelMessageWithSource)
						.interaction_response_data(|message| message.content(content))
				})
				.await
			{
				error!("Cannot respond to slash command: {}", why);
			}
		}
	}

	async fn message(&self, ctx: Context, msg: Message) {
		trace!("Received message: {}", msg.content);

		let configuration_lock = get_configuration_lock(&ctx).await;
		let configuration = configuration_lock.read().await;

		if let Some(message_responders) = &configuration.message_responders {
			if let Some(responder) = message_responders.iter().find(|responder| {
				responder.includes.iter().any(|include| {
					include.channels.iter().any(|channel| todo!("Implement inclusion check"))
				}) && responder.excludes.iter().all(|exclude| todo!("Implement exclusion check"))
			}) {
				if let Some(condition) = &responder.condition {
					let join_date = ctx
						.http
						.get_member(msg.guild_id.unwrap().0, msg.author.id.0)
						.await
						.unwrap()
						.joined_at
						.unwrap();

					let member_age = Timestamp::now().unix_timestamp() - join_date.unix_timestamp();

					if let Some(age) = condition.user.server_age {
						todo!("Implement age check")
					}
				}
			}
		}
	}

	async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
		trace!("Thread created: {}", thread.name);

		let configuration_lock = get_configuration_lock(&ctx).await;
		let configuration = configuration_lock.read().await;

		if let Some(introducers) = &configuration.thread_introductions {
			if let Some(introducer) = introducers.iter().find(|introducer| {
				introducer
					.channels
					.iter()
					.any(|channel_id| *channel_id == thread.parent_id.unwrap().0)
			}) {
				if let Err(why) = thread.say(&ctx.http, &introducer.message).await {
					error!("Error sending message: {:?}", why);
				}
			}
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		info!("Connected as {}", ready.user.name);

		Command::create_global_application_command(&ctx.http, |command| {
			command.name("reload").description("Reloads the configuration.")
		})
		.await
		.expect("Could not create command.");
	}
}

#[tokio::main]
async fn main() {
	log::set_logger(&LOGGER)
		.map(|()| log::set_max_level(LevelFilter::Info))
		.expect("Could not set logger.");

	let configuration = BotConfiguration::load().expect("Failed to load configuration");

	let mut client = Client::builder(
		&configuration.discord_authorization_token,
		GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT,
	)
	.event_handler(Handler)
	.await
	.expect("Failed to create client");

	client.data.write().await.insert::<Configuration>(Arc::new(RwLock::new(
		BotConfiguration::load().expect("Failed to load configuration"),
	)));

	if let Err(why) = client.start().await {
		error!("{:?}", why);
	} else {
		info!("Client started.");
	}
}
