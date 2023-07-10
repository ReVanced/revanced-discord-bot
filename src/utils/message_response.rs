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

    let mut guild_message = None;

    let member = &new_message.member.as_ref().unwrap();
    let member_roles = &member.roles;

    let joined_at = member.joined_at.unwrap().unix_timestamp();
    let must_joined_at = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(joined_at, 0).unwrap(),
        Utc,
    );

    for response in responses {
        if let Some(includes) = &response.includes {
            if let Some(roles) = &includes.roles {
                // check if the role is whitelisted
                if !roles.iter().any(|&role_id| {
                    member_roles
                        .iter()
                        .any(|&member_role| role_id == member_role.0)
                }) {
                    continue;
                }
            }

            if let Some(channels) = &includes.channels {
                // check if the channel is whitelisted, if not, check if the channel is a thread, if it is check if the parent id is whitelisted
                if !channels.contains(&new_message.channel_id.0) {
                    if response.thread_options.is_some() {
                        if guild_message.is_none() {
                            guild_message = Some(
                                new_message
                                    .channel(&ctx.http)
                                    .await
                                    .unwrap()
                                    .guild()
                                    .unwrap(),
                            );
                        };

                        let Some(parent_id) = guild_message.as_ref().unwrap().parent_id else { continue; };
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
                let but_joined_at = Utc::now() - Duration::days(min_age);

                if must_joined_at <= but_joined_at {
                    continue;
                }
            }
        }

        let channel_id = new_message.channel_id;

        let mut message_reference: Option<&serenity::Message> = None;

        // If the message has a reference and the response is set to respond to references, respond to the reference
        if let Some(respond_to_reference) = response.respond_to_reference {
            if respond_to_reference {
                if let Some(reference) = &new_message.referenced_message {
                    message_reference = Some(reference.as_ref());
                    if let Err(err) = new_message.delete(&ctx.http).await {
                        error!(
                            "Failed to delete the message from {}. Error: {:?}",
                            new_message.author.tag(),
                            err
                        );
                    }
                }
            }
        }

        if let Err(err) = channel_id
            .send_message(&ctx.http, |m| {
                if let Some(reference) = message_reference {
                    m.reference_message(reference);
                } else {
                    m.reference_message(new_message);
                }

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

            if thread_options.only_on_first_message
                && !channel_id
                    .messages(&ctx.http, |b| b.limit(1).before(new_message))
                    .await
                    .unwrap()
                    .is_empty()
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
