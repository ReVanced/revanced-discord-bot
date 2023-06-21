use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use regex::Regex;
use tracing::log::error;

use super::*;
use crate::utils::bot::get_data_lock;

pub fn contains_match(regex: &[Regex], text: &str) -> bool {
    regex.iter().any(|r| r.is_match(text))
}

pub async fn handle_message_response(ctx: &serenity::Context, new_message: &serenity::Message) {
    if new_message.guild_id.is_none() || new_message.author.bot {
        return;
    }

    let data_lock = get_data_lock(ctx).await;
    let responses = &data_lock.read().await.configuration.message_responses;
    let message = &new_message.content;

    for response in responses {
        if let Some(includes) = &response.includes {
            if let Some(channels) = &includes.channels {
                // check if the channel is whitelisted, if not, check if the channel is a thread, if it is check if the parent id is whitelisted
                if !channels.contains(&new_message.channel_id.0) {
                    if response.thread_options.is_some() {
                        let Some(parent_id) = new_message.channel(&ctx.http).await.unwrap().guild().unwrap().parent_id else { continue; };
                        if !channels.contains(&parent_id.0) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            }

            // check if message matches regex
            if !contains_match(&includes.match_field, message) {
                tracing::log::trace!("Message does not match regex");
                continue;
            }
        }

        if let Some(excludes) = &response.excludes {
            // check if the role is blacklisted
            if let Some(roles) = &excludes.roles {
                let member_roles = &new_message.member.as_ref().unwrap().roles;
                if roles.iter().any(|&role_id| {
                    member_roles
                        .iter()
                        .any(|&member_role| role_id == member_role.0)
                }) {
                    continue;
                }
            }
        }

        if let Some(condition) = &response.condition {
            let min_age = condition.user.server_age;

            if min_age != 0 {
                let joined_at = ctx
                    .http
                    .get_member(new_message.guild_id.unwrap().0, new_message.author.id.0)
                    .await
                    .unwrap()
                    .joined_at
                    .unwrap()
                    .unix_timestamp();

                let must_joined_at = DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(joined_at, 0).unwrap(),
                    Utc,
                );
                let but_joined_at = Utc::now() - Duration::days(min_age);

                if must_joined_at <= but_joined_at {
                    continue;
                }
            }
        }

        let channel_id = new_message.channel_id;

        if let Err(err) = channel_id
            .send_message(&ctx.http, |m| {
                m.reference_message(new_message);
                match &response.response.embed {
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
                            .author(|a| a.name(&embed.author.name).icon_url(&embed.author.icon_url))
                    }),
                    None => m.content(response.response.message.as_ref().unwrap()),
                }
            })
            .await
        {
            error!(
                "Failed to reply to the message from {}. Error: {:?}",
                new_message.author.tag(),
                err
            );
        } else if let Some(thread_options) = &response.thread_options {
            let channel = channel_id
                .to_channel(&ctx.http)
                .await
                .unwrap()
                .guild()
                .unwrap();

            // only apply this thread if the channel is a thread
            if channel.thread_metadata.is_none() {
                return;
            }

            // only edit this thread if the message is the first one
            if !channel_id
                .messages(&ctx.http, |b| b.limit(1).before(new_message))
                .await
                .unwrap().is_empty()
            {
                return;
            }

            if let Err(err) = channel
                .edit_thread(&ctx.http, |e| {
                    e.locked(thread_options.lock_on_response)
                        .archived(thread_options.close_on_response)
                })
                .await
            {
                error!(
                    "Failed to edit the thread from {}. Error: {:?}",
                    new_message.author.tag(),
                    err
                );
            }
        }
    }
}
