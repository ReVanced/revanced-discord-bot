use poise::serenity_prelude::CreateEmbed;

use crate::model::application::Configuration;

pub(crate) fn load_configuration() -> Configuration {
    Configuration::load().expect("Failed to load configuration")
}

trait PoiseEmbed {
    fn create_embed(self, embed: &mut CreateEmbed) -> &mut CreateEmbed;
}

impl PoiseEmbed for crate::model::application::Embed {
    fn create_embed(self, embed: &mut CreateEmbed) -> &mut CreateEmbed {
        embed
            .title(self.title)
            .description(self.description)
            .color(self.color)
            .fields(
                self.fields
                    .iter()
                    .map(|field| (field.name.clone(), field.value.clone(), field.inline)),
            )
            .footer(|f| {
                f.text(self.footer.text);
                f.icon_url(self.footer.icon_url)
            })
            .thumbnail(self.thumbnail.url)
            .image(self.image.url)
            .author(|a| a.name(self.author.name).icon_url(self.author.icon_url))
    }
}