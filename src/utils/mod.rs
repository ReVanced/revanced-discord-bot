use chrono::Duration;
use poise::serenity_prelude::{
    self as serenity, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, Member, RoleId,
};

use crate::model::application::{Configuration, Embed};

pub mod bot;
pub mod code_embed;
pub mod decancer;
pub mod macros;
pub mod message_response;
pub mod moderation;

pub fn parse_duration(duration: String) -> Result<Duration, parse_duration::parse::Error> {
    let d = parse_duration::parse(&duration)?;
    Ok(Duration::nanoseconds(d.as_nanos() as i64))
}

pub fn create_embed(configuration: &Configuration, embed_configuration: &Embed) -> CreateEmbed {
    let mut create_embed = CreateEmbed::new();

    if let Some(title) = embed_configuration
        .title
        .as_ref()
        .or(configuration.default_embed.title.as_ref())
    {
        create_embed = create_embed.title(title);
    }

    if let Some(description) = embed_configuration
        .description
        .as_ref()
        .or(configuration.default_embed.description.as_ref())
    {
        create_embed = create_embed.description(description);
    }

    if let Some(color) = embed_configuration
        .color
        .or(configuration.default_embed.color)
    {
        create_embed = create_embed.color(color);
    }

    if let Some(fields) = embed_configuration
        .fields
        .as_ref()
        .or(configuration.default_embed.fields.as_ref())
    {
        for field in fields {
            create_embed =
                create_embed.field(&field.name, &field.value, field.inline.unwrap_or(false));
        }
    }

    if let Some(footer) = embed_configuration
        .footer
        .as_ref()
        .or(configuration.default_embed.footer.as_ref())
    {
        let mut create_footer = CreateEmbedFooter::new(&footer.text);

        if let Some(icon_url) = &footer.icon_url {
            create_footer = create_footer.icon_url(icon_url);
        }

        create_embed = create_embed.footer(create_footer);
    }

    if let Some(image_url) = embed_configuration
        .image_url
        .as_ref()
        .or(configuration.default_embed.image_url.as_ref())
    {
        create_embed = create_embed.image(image_url);
    }

    if let Some(thumbnail_url) = embed_configuration
        .thumbnail_url
        .as_ref()
        .or(configuration.default_embed.thumbnail_url.as_ref())
    {
        create_embed = create_embed.thumbnail(thumbnail_url);
    }
    if let Some(author) = embed_configuration
        .author
        .as_ref()
        .or(configuration.default_embed.author.as_ref())
    {
        let mut create_author = CreateEmbedAuthor::new(&author.name);

        if let Some(icon_url) = &author.icon_url {
            create_author = create_author.icon_url(icon_url);
        }

        if let Some(url) = &author.url {
            create_author = create_author.url(url);
        }

        create_embed = create_embed.author(create_author);
    }

    create_embed
}

pub fn create_default_embed(configuration: &Configuration) -> CreateEmbed {
    create_embed(configuration, &configuration.default_embed)
}
