use poise::serenity_prelude::CreateMessage;

use serenity::Message;
use tracing::log::error;

use super::*;
use crate::{model::application::Trigger, BotData};

impl Trigger {
    fn matches(&self, new_message: &Message, member_roles: &[RoleId]) -> bool {
        if let Some(channels) = &self.channels {
            if !channels.contains(&new_message.channel_id.get()) {
                return false;
            }
        }

        if let Some(roles) = &self.roles {
            if !member_roles
                .iter()
                .any(|&member_role| roles.contains(&member_role.get()))
            {
                return false;
            }
        }

        if !self.regex.is_empty() && self.regex.iter().any(|r| r.is_match(&new_message.content)) {
            return true;
        }

        false
    }
}

pub async fn handle_message_response(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    data: &BotData,
) {
    if new_message.guild_id.is_none() || new_message.author.bot {
        return;
    }

    let configuration = &data.read().await.configuration;

    let member_roles = &new_message.member.as_ref().unwrap().roles;

    for response in &configuration.responses {
        if let Some(whitelist) = &response.whitelist {
            if !whitelist.matches(new_message, member_roles) {
                continue;
            }
        }

        if let Some(blacklist) = &response.blacklist {
            if blacklist.matches(new_message, member_roles) {
                continue;
            }
        }

        if let Err(err) = new_message
            .channel_id
            .send_message(&ctx.http, {
                let mut message = CreateMessage::default();

                message = message.reference_message(
                    if let Some(reference) = &new_message.referenced_message {
                        reference.as_ref()
                    } else {
                        new_message
                    },
                );

                if let Some(embed_configuration) = &response.message.embed {
                    message = message.embed(create_embed(configuration, embed_configuration))
                }

                if let Some(content) = &response.message.content {
                    message = message.content(content)
                }

                message
            })
            .await
        {
            error!(
                "Failed to reply to {}. Error: {:?}",
                new_message.author.tag(),
                err
            );
        }
    }
}
