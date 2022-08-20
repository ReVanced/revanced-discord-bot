use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use regex::Regex;
use tracing::debug;

use super::*;
use crate::utils::bot::get_data_lock;

fn contains_match(regex: &[Regex], text: &str) -> bool {
    regex.iter().any(|r| r.is_match(text))
}

pub async fn message_create(ctx: &serenity::Context, new_message: &serenity::Message) {
    debug!("Received message: {}", new_message.content);
    if new_message.guild_id.is_none() || new_message.author.bot {
        return;
    }

    if let Some(message_response) = get_data_lock(ctx)
		.await
		.read()
		.await
		.configuration
		.read()
		.await
		.message_responses
		.iter()
		.find(|&response| {
			// check if the message was sent in a channel that is included in the responder
			response.includes.channels.iter().any(|&channel_id| channel_id == new_message.channel_id.0)
				// check if the message was sent by a user that is not excluded from the responder
				&& !response.excludes.roles.iter().any(|&role_id| role_id == new_message.author.id.0)
				// check if the message does not match any of the excludes
				&& !contains_match(&response.excludes.match_field, &new_message.content)
				// check if the message matches any of the includes
				&& contains_match(&response.includes.match_field, &new_message.content)
		})
	{
		let min_age = message_response.condition.user.server_age;

		if min_age != 0 {
			let joined_at = ctx
				.http
				.get_member(new_message.guild_id.unwrap().0, new_message.author.id.0)
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

			new_message.channel_id
				.send_message(&ctx.http, |m| {
					m.reference_message(new_message);
					match &message_response.response.embed {
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
						None => m.content(message_response.response.message.as_ref().unwrap()),
					}
				})
				.await
				.expect("Could not reply to message author.");
		}
	}
}
