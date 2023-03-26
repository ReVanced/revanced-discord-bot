macro_rules! to_user {
    ($user:ident, $ctx:ident) => {{
        $user.to_user(&$ctx.serenity_context().http).await?
    }};
}

pub(crate) use to_user;
