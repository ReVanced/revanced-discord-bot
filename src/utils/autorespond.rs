use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use poise::serenity_prelude::Attachment;
use regex::Regex;
use tracing::debug;

use super::*;
use crate::utils::bot::get_data_lock;
use crate::utils::ocr;

pub fn contains_match(regex: &[Regex], text: &str) -> bool {
    regex.iter().any(|r| r.is_match(text))
}

async fn attachments_contains(attachments: &[Attachment], regex: &[Regex]) -> bool {
    for attachment in attachments {
        debug!("Checking attachment {}", &attachment.url);

        if !&attachment.content_type.as_ref().unwrap().contains("image") {
            continue;
        }

        if contains_match(
            regex,
            &ocr::get_text_from_image_url(&attachment.url).await.unwrap(),
        ) {
            return true;
        }
    }
    false
}

pub async fn auto_respond(ctx: &serenity::Context, new_message: &serenity::Message) {
    if new_message.guild_id.is_none() || new_message.author.bot {
        return;
    }

    let data_lock = get_data_lock(ctx).await;
    let responses = &data_lock.read().await.configuration.message_responses;

    for response in responses {
        // check if the message was sent in a channel that is included in the responder
        if !response
            .includes
            .channels
            .iter()
            .any(|&channel_id| channel_id == new_message.channel_id.0)
        {
            continue;
        }

        let excludes = &response.excludes;
        // check if the message was sent by a user that is not excluded from the responder
        if excludes
            .roles
            .iter()
            .any(|&role_id| role_id == new_message.author.id.0)
        {
            continue;
        }

        let message = &new_message.content;
        let contains_attachments = !new_message.attachments.is_empty();

        // check if the message does not match any of the excludes
        if contains_match(&excludes.match_field.text, message) {
            continue;
        }

        if contains_attachments
            && !excludes.match_field.ocr.is_empty()
            && attachments_contains(&new_message.attachments, &excludes.match_field.ocr).await
        {
            continue;
        }

        // check if the message does match any of the includes
        if !(contains_match(&response.includes.match_field.text, message)
            || (contains_attachments
                && !response.includes.match_field.ocr.is_empty()
                && attachments_contains(
                    &new_message.attachments,
                    &response.includes.match_field.ocr,
                )
                .await))
        {
            continue;
        }

        let min_age = response.condition.user.server_age;

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

            new_message
                .channel_id
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
                                .author(|a| {
                                    a.name(&embed.author.name).icon_url(&embed.author.icon_url)
                                })
                        }),
                        None => m.content(response.response.message.as_ref().unwrap()),
                    }
                })
                .await
                .expect("Could not reply to message author.");
        }
    }
}
