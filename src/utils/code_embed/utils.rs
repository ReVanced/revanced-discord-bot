use poise::serenity_prelude::{CreateAttachment, CreateMessage, EditMessage};
use reqwest::Url;
use tracing::{debug, error, trace};

use super::*;
use crate::utils::code_embed::url_parser::{CodePreview, CodeUrlParser, GitHubCodeUrl};

pub async fn code_preview(ctx: &serenity::Context, new_message: &serenity::Message) {
    let mut urls: Vec<Url> = Vec::new();

    fn get_all_http_urls(string: &str, out: &mut Vec<Url>) {
        fn get_http_url(slice: &str) -> Option<(&str, usize)> {
            if let Some(start) = slice.find("http") {
                debug!("HTTP url start: {}", start);

                let new_slice = &slice[start..];

                if let Some(end) = new_slice
                    .find(' ')
                    .or(Some(new_slice.len()))
                    .map(|slice_end| start + slice_end)
                {
                    debug!("HTTP url end: {}", end);

                    let url = &slice[start..end];
                    return Some((url, end));
                }
            }

            None
        }

        if let Some((url, next_start_index)) = get_http_url(string) {
            if let Ok(url) = Url::parse(url) {
                out.push(url);
            } else {
                error!("Failed to parse url: {}", url);
            }

            get_all_http_urls(&string[next_start_index..], out);
        }
    }
    get_all_http_urls(&new_message.content, &mut urls);

    let mut code_previews: Vec<CodePreview> = Vec::new();

    for url in urls {
        // TODO: Add support for other domains by using the provider pattern
        let code_url = GitHubCodeUrl {
            url: url.clone(),
        };

        match code_url.parse().await {
            Err(e) => trace!("Failed to parse url: {} ({:?})", url, e),
            Ok(code_preview) => {
                if code_preview.preview.is_some() {
                    code_previews.push(code_preview)
                }
            },
        }
    }

    if code_previews.is_empty() {
        return; // Nothing to do
    }

    if let Err(err) = &new_message
        .channel_id
        .send_message(
            &ctx.http,
            {
                let mut message = CreateMessage::new();

                for preview in code_previews.iter() {
                    let language = match preview.code.language.as_ref() {
                        Some(language) => language,
                        None => "txt",
                    };

                    let name = format!("{}.{}", &preview.code.branch_or_sha, language);

                    let content = preview.preview.as_ref().unwrap().as_bytes();

                    message = message.add_file(CreateAttachment::bytes(content, name.as_str()));
                }

                message
            },
        )
        .await
    {
        error!(
            "Failed to reply to the message from {}. Error: {:?}",
            new_message.author.tag(),
            err
        );
        return;
    }

    if let Err(err) = new_message
        .clone()
        .edit(&ctx.http, EditMessage::new().suppress_embeds(true))
        .await
    {
        error!("Failed to remove embeds. Error: {:?}", err);
    }
}
