use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use configuration::application::Configuration;
use log::{error, info, trace, LevelFilter};
use logger::logging::SimpleLogger;
use regex::Regex;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::{GuildChannel, Message};
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType, MessageFlags};
use serenity::prelude::{GatewayIntents, RwLock, TypeMapKey};
use serenity::{async_trait, Client};
mod configuration;
mod logger;

static LOGGER: SimpleLogger = SimpleLogger;

struct BotConfiguration;

impl TypeMapKey for BotConfiguration {
	type Value = Arc<RwLock<Configuration>>;
}

pub struct Handler;

async fn get_configuration_lock(ctx: &Context) -> Arc<RwLock<Configuration>> {
	ctx.data
		.read()
		.await
		.get::<BotConfiguration>()
		.expect("Expected Configuration in TypeMap.")
		.clone()
}

fn contains_match(strings: &Vec<String>, text: &String) -> bool {
	strings.iter().any(|regex| Regex::new(regex).unwrap().is_match(&text))
}

fn load_configuration() -> Configuration {
	Configuration::load().expect("Failed to load configuration")
}

#[async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		trace!("Created an interaction: {:?}", interaction);

		if let Interaction::ApplicationCommand(command) = interaction {
			let configuration_lock = get_configuration_lock(&ctx).await;
			let mut configuration = configuration_lock.write().await;

			let administrators = &configuration.administrators;
			let member = command.member.as_ref().unwrap();
			let user_id = member.user.id.0;
			let mut stop_command = false;
			let mut permission_granted = false;

			// check if the user is an administrator
			if administrators.users.iter().any(|&id| user_id == id) {
				permission_granted = true
			}
			// check if the user has an administrating role
			if !permission_granted
				&& administrators
					.roles
					.iter()
					.any(|role_id| member.roles.iter().any(|member_role| member_role == role_id))
			{
				permission_granted = true
			}

			let content = if permission_granted {
				match command.data.name.as_str() {
					"reload" => {
						trace!("{:?} reloaded the configuration.", command.user);

						let new_config = load_configuration();

						configuration.administrators = new_config.administrators;
						configuration.message_responders = new_config.message_responders;
						configuration.thread_introductions = new_config.thread_introductions;

						"Successfully reloaded configuration.".to_string()
					},
					"stop" => {
						trace!("{:?} stopped the bot.", command.user);
						stop_command = true;
						"Stopped the bot.".to_string()
					},
					_ => "Unknown command.".to_string(),
				}
			} else {
				"You do not have permission to use this command.".to_string()
			};

			// send the response
			if let Err(why) = command
				.create_interaction_response(&ctx.http, |response| {
					response
						.kind(InteractionResponseType::ChannelMessageWithSource)
						.interaction_response_data(|message| {
							message.content(content).flags(MessageFlags::EPHEMERAL)
						})
				})
				.await
			{
				error!("Cannot respond to slash command: {}", why);
			}

			if stop_command {
				std::process::exit(0);
			}
		}
	}

	async fn message(&self, ctx: Context, msg: Message) {
		trace!("Received message: {}", msg.content);
		if msg.guild_id.is_none() || msg.author.bot {
			return;
		}

		if let Some(responder) =
			get_configuration_lock(&ctx).await.read().await.message_responders.iter().find(
				|&responder| {
					// check if the message was sent in a channel that is included in the responder
					responder.includes.channels.iter().any(|&channel_id| channel_id == msg.channel_id.0)
					// check if the message was sent by a user that is not excluded from the responder
					&& !responder.excludes.roles.iter().any(|&role_id| role_id == msg.author.id.0)
					// check if the message does not match any of the excludes
					&& !contains_match(&responder.excludes.match_field, &msg.content)
					// check if the message matches any of the includes
					&& contains_match(&responder.includes.match_field, &msg.content)
				},
			) {
			let min_age = responder.condition.user.server_age;

			if min_age != 0 {
				let joined_at = ctx
					.http
					.get_member(msg.guild_id.unwrap().0, msg.author.id.0)
					.await
					.unwrap()
					.joined_at
					.unwrap()
					.unix_timestamp();

				let must_joined_at =
					DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(joined_at, 0), Utc);
				let but_joined_at = Utc::now() - Duration::days(min_age);

				if must_joined_at <= but_joined_at {
					return;
				}

				msg.channel_id
					.send_message(&ctx.http, |m| {
						m.reference_message(&msg);
						match &responder.response.embed {
							Some(embed) => m.embed(|e| {
								e.title(&embed.title)
									.description(&embed.description)
									.color(embed.color)
									.fields(embed.fields.iter().map(|field| {
										(field.name.clone(), field.value.clone(), field.inline)
									}))
									.footer(|f| {
										f.text(&embed.footer.text);
										f.icon_url(&embed.footer.icon_url)
									})
									.thumbnail(&embed.thumbnail.url)
									.image(&embed.image.url)
									.author(|a| {
										a.name(&embed.author.name).icon_url(&embed.author.icon_url)
									})
							}),
							None => m.content(responder.response.message.as_ref().unwrap()),
						}
					})
					.await
					.expect("Could not reply to message author.");
			}
		}
	}

	async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
		trace!("Thread created: {}", thread.name);

		let configuration_lock = get_configuration_lock(&ctx).await;
		let configuration = configuration_lock.read().await;

		if let Some(introducer) = &configuration.thread_introductions.iter().find(|introducer| {
			introducer.channels.iter().any(|channel_id| *channel_id == thread.parent_id.unwrap().0)
		}) {
			if let Err(why) =
				thread.say(&ctx.http, &introducer.response.message.as_ref().unwrap()).await
			{
				error!("Error sending message: {:?}", why);
			}
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		info!("Connected as {}", ready.user.name);

		for (cmd, description) in
			[("repload", "Reloads the configuration."), ("stop", "Stop the Discord bot.")]
		{
			Command::create_global_application_command(&ctx.http, |command| {
				command.name(cmd).description(description)
			})
			.await
			.expect("Could not create command.");
		}
	}
}

#[tokio::main]
async fn main() {
	log::set_logger(&LOGGER)
		.map(|()| log::set_max_level(LevelFilter::Warn))
		.expect("Could not set logger.");

	let configuration = load_configuration();

	let mut client = Client::builder(
		&configuration.discord_authorization_token,
		GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT,
	)
	.event_handler(Handler)
	.await
	.expect("Failed to create client");

	client.data.write().await.insert::<BotConfiguration>(Arc::new(RwLock::new(configuration)));

	if let Err(why) = client.start().await {
		error!("{:?}", why);
	} else {
		info!("Client started.");
	}
}
