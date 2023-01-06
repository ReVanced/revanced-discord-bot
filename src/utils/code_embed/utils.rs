use reqwest::Url;
use tracing::{debug, error, trace};

use super::*;
use crate::utils::bot::get_data_lock;
use crate::utils::code_embed::url_parser::{CodePreview, CodeUrlParser, GitHubCodeUrl};

pub async fn code_preview(ctx: &serenity::Context, new_message: &serenity::Message) {
    let data_lock = get_data_lock(ctx).await;
    let configuration = &data_lock.read().await.configuration;

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

    if let Err(err) = new_message
        .channel_id
        .send_message(&ctx.http, |message| {
            let mut message = message.reference_message(new_message);
            let icon_url = &new_message.guild(&ctx.cache).unwrap().icon_url();

            for code_preview in code_previews {
                message = message.add_embed(|embed| {
                    if let Some(url) = icon_url {
                        embed.footer(|f| f.text("ReVanced").icon_url(url))
                    } else {
                        embed
                    }
                    .color(configuration.general.embed_color)
                    .description(code_preview.preview.unwrap())
                });
            }

            message
        })
        .await
    {
        error!(
            "Failed to reply to the message from {}. Error: {:?}",
            new_message.author.tag(),
            err
        );
        return;
    }

    if let Err(err) = new_message.clone().suppress_embeds(&ctx.http).await {
        error!("Failed to remove embeds. Error: {:?}", err);
    }
}
