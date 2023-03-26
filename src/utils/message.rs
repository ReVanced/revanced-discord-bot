use chrono::Utc;
use poise::serenity_prelude::Message;
use poise::CreateReply;

pub fn clone_message<'a, 'b>(
    message: &'a Message,
    to_reply: &'b mut CreateReply<'a>,
) -> &'b mut CreateReply<'a> {
    let mut reply = to_reply.content(message.content.as_str());

    if let Some(embed) = message.embeds.get(0) {
        reply = reply.embed(|e| {
            let mut new_embed = e;

            if let Some(color) = embed.colour {
                new_embed = new_embed.color(color);
            }

            new_embed = new_embed.timestamp(Utc::now().to_rfc3339());

            if let Some(title) = &embed.title {
                new_embed = new_embed.title(title);
            }

            if let Some(description) = &embed.description {
                new_embed = new_embed.description(description);
            }

            if let Some(footer) = &embed.footer {
                new_embed = new_embed.footer(|f| f.text(&footer.text));
            }

            if let Some(author) = &embed.author {
                new_embed = new_embed.author(|a| a.name(&author.name));
            }

            if let Some(image) = &embed.image {
                new_embed = new_embed.image(image.url.as_str());
            }

            if let Some(thumbnail) = &embed.thumbnail {
                new_embed = new_embed.thumbnail(thumbnail.url.as_str());
            }

            for field in &embed.fields {
                new_embed =
                    new_embed.field(field.name.as_str(), field.value.as_str(), field.inline);
            }

            new_embed
        })
    }

    reply
}
