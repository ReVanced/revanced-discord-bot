use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, Message};
use poise::CreateReply;

pub fn clone_message(message: &Message) -> CreateReply {
    let mut reply = CreateReply {
        content: Some(message.content.clone()),
        ..Default::default()
    };

    if let Some(embed) = message.embeds.first() {
        let mut new_embed = CreateEmbed::new();

        if let Some(color) = embed.colour {
            new_embed = new_embed.color(color);
        }

        new_embed = new_embed.timestamp(Utc::now());

        if let Some(title) = &embed.title {
            new_embed = new_embed.title(title);
        }

        if let Some(description) = &embed.description {
            new_embed = new_embed.description(description);
        }

        if let Some(footer) = &embed.footer {
            new_embed = new_embed.footer(CreateEmbedFooter::new(&footer.text));
        }

        if let Some(author) = &embed.author {
            new_embed = new_embed.author(CreateEmbedAuthor::new(&author.name));
        }

        if let Some(image) = &embed.image {
            new_embed = new_embed.image(image.url.as_str());
        }

        if let Some(thumbnail) = &embed.thumbnail {
            new_embed = new_embed.thumbnail(thumbnail.url.as_str());
        }

        for field in &embed.fields {
            new_embed = new_embed.field(field.name.as_str(), field.value.as_str(), field.inline);
        }

        reply = reply.embed(new_embed);
    }

    reply
}
