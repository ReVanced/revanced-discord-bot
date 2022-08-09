use decancer::Decancer;
use poise::serenity_prelude::{self as serenity, CreateEmbed};
use tracing::{error, info};

use crate::model::application::Configuration;

const DECANCER: Decancer = Decancer::new();

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

pub async fn cure(ctx: &serenity::Context, member: &serenity::Member) {
    let display_name = member.display_name();
    let name = display_name.to_string();

    let cured_user_name = DECANCER.cure(&name);

    if name.to_lowercase() == cured_user_name {
        return; // username is already cured
    }

    match member
        .guild_id
        .edit_member(&ctx.http, member.user.id, |edit_member| {
            edit_member.nickname(cured_user_name)
        })
        .await
    {
        Ok(_) => info!("Cured user {}", member.user.tag()),
        Err(err) => error!("Failed to cure user {}: {}", name, err),
    }
}
