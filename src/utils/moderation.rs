use tokio::task::JoinHandle;

use super::*;
use crate::db::database::Database;
use crate::db::model::Muted;
use crate::{Context, Error};

pub enum ModerationKind {
    Mute(String, String, Option<Error>), // Reason, Expires, Error
    Unmute(Option<Error>),               // Error
}

pub fn queue_unmute_member(
    ctx: &serenity::Context,
    database: &Database,
    member: &Member,
    mute_role_id: u64,
    mute_duration: u64,
) -> JoinHandle<Option<Error>> {
    let ctx = ctx.clone();
    let database = database.clone();
    let mut member = member.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(mute_duration)).await;

        let delete_result = database
            .find_and_delete::<Muted>(
                "muted",
                Muted {
                    user_id: Some(member.user.id.0.to_string()),
                    ..Default::default()
                }
                .into(),
                None,
            )
            .await;

        if let Err(database_remove_result) = delete_result {
            Some(database_remove_result)
        } else if let Some(find_result) = delete_result.unwrap() {
            let taken_roles = find_result
                .taken_roles
                .unwrap()
                .into_iter()
                .map(|r| RoleId::from(r.parse::<u64>().unwrap()))
                .collect::<Vec<_>>();

            if let Err(add_role_result) = member.add_roles(&ctx.http, &taken_roles).await {
                Some(Error::from(add_role_result))
            } else if let Err(remove_result) = member.remove_role(ctx.http, mute_role_id).await {
                Some(Error::from(remove_result))
            } else {
                None
            }
        } else {
            None
        }
    })
}

pub async fn respond_mute_command(
    ctx: &Context<'_>,
    moderation: ModerationKind,
    user: &serenity::User,
    embed_color: i32,
) -> Result<(), Error> {
    let tag = user.tag();
    let image = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    ctx.send(|f| {
        f.embed(|f| {
            match moderation {
                ModerationKind::Mute(reason, expires, error) => match error {
                    Some(err) => f.title(format!("Failed to mute {}", tag)).field(
                        "Exception",
                        err.to_string(),
                        false,
                    ),
                    None => f.title(format!("Muted {}", tag)),
                }
                .field("Reason", reason, false)
                .field("Expires", expires, false),
                ModerationKind::Unmute(error) => match error {
                    Some(err) => f.title(format!("Failed to unmute {}", tag)).field(
                        "Exception",
                        err.to_string(),
                        false,
                    ),
                    None => f.title(format!("Unmuted {}", tag)),
                },
            }
            .color(embed_color)
            .thumbnail(image)
        })
    })
    .await?;

    Ok(())
}
