use chrono::Utc;
use reqwest::Url;
use tracing::{debug, error};

use super::*;
use crate::utils::bot::get_data_lock;
use crate::utils::code_embed::url_parser::{CodePreview, CodeUrlParser, GitHubCodeUrl};

pub async fn handle_code_url(ctx: &serenity::Context, new_message: &serenity::Message) {
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
            Err(e) => error!("Failed to parse url: {} ({:?})", url, e),
            Ok(code_preview) => code_previews.push(code_preview),
        }
    }

    if code_previews.is_empty() {
        return; // Nothing to do
    }

    if let Err(err) = new_message
        .channel_id
        .send_message(&ctx.http, |m| {
            let mut message = m;

            for code_preview in code_previews {
                message = message.add_embed(|e| {
                    let mut e = e
                        .title("Source code")
                        .url(code_preview.code.original_code_url)
                        .color(configuration.general.embed_color)
                        .field(
                            "Raw link",
                            format!("[Click here]({})", code_preview.code.raw_code_url),
                            true,
                        )
                        .field("Branch/Sha", code_preview.code.branch_or_sha, true);

                    if let Some(preview) = code_preview.preview {
                        e = e.field("Preview", preview, false)
                    }

                    let guild = &new_message.guild(&ctx.cache).unwrap();
                    if let Some(url) = &guild.icon_url() {
                        e = e.footer(|f| {
                            f.icon_url(url).text(format!(
                                "{} • {}",
                                guild.name,
                                Utc::today().format("%Y/%m/%d")
                            ))
                        })
                    }

                    e.field(
                        format!("Original message by {}", new_message.author.tag()),
                        new_message.content.clone(),
                        false,
                    )
                });
            }

            message.content(
                new_message
                    .mentions
                    .iter()
                    .map(|m| format!("<@{}>", m.id))
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        })
        .await
    {
        error!(
            "Failed to reply to the message from {}. Error: {:?}",
            new_message.author.tag(),
            err
        );
    }

    if let Err(err) = new_message.delete(&ctx.http).await {
        error!("Failed to delete the message. Error: {:?}", err);
    }
}
