use poise::serenity_prelude::{self as serenity, Mutex, RwLock, ShardManager, UserId};
use std::sync::Arc;

use crate::{model::application::Configuration, Error};

mod message_create;
mod thread_create;

// Share the lock reference between the threads in serenity framework
async fn get_configuration_lock(ctx: &serenity::Context) -> Arc<RwLock<Configuration>> {
    ctx.data
        .read()
        .await
        .get::<Configuration>()
        .expect("Expected Configuration in TypeMap.")
        .clone()
}

pub struct Handler<T> {
    options: poise::FrameworkOptions<T, Error>,
    data: T,
    bot_id: RwLock<Option<UserId>>,
    shard_manager: RwLock<Option<Arc<Mutex<ShardManager>>>>,
}

// Custom handler to dispatch poise events
impl<T: Send + Sync> Handler<T> {
    pub fn new(options: poise::FrameworkOptions<T, Error>, data: T) -> Self {
        Self {
            options,
            data,
            shard_manager: RwLock::new(None),
            bot_id: RwLock::new(None),
        }
    }

    pub async fn set_shard_manager(&self, shard_manager: Arc<Mutex<serenity::ShardManager>>) {
        *self.shard_manager.write().await = Some(shard_manager);
    }

    async fn dispatch_poise_event(&self, ctx: &serenity::Context, event: &poise::Event<'_>) {
        let framework_data = poise::FrameworkContext {
            bot_id: self.bot_id.read().await.unwrap(),
            options: &self.options,
            user_data: &self.data,
            shard_manager: &(*self.shard_manager.read().await).clone().unwrap(), // Shard manager can be read between all poise events without locks
        };
        poise::dispatch_event(framework_data, ctx, event).await;
    }
}

// Manually dispatch events from serenity to poise
#[serenity::async_trait]
impl serenity::EventHandler for Handler<Arc<RwLock<Configuration>>> {
    async fn ready(&self, _ctx: serenity::Context, ready: serenity::Ready) {
        *self.bot_id.write().await = Some(ready.user.id);
    }

    async fn message(&self, ctx: serenity::Context, new_message: serenity::Message) {
        message_create::message_create(&ctx, &new_message).await;

        self.dispatch_poise_event(&ctx, &poise::Event::Message { new_message })
            .await;
    }

    async fn interaction_create(&self, ctx: serenity::Context, interaction: serenity::Interaction) {
        self.dispatch_poise_event(&ctx, &poise::Event::InteractionCreate { interaction })
            .await;
    }

    async fn message_update(
        &self,
        ctx: serenity::Context,
        old_if_available: Option<serenity::Message>,
        new: Option<serenity::Message>,
        event: serenity::MessageUpdateEvent,
    ) {
        self.dispatch_poise_event(
            &ctx,
            &poise::Event::MessageUpdate {
                old_if_available,
                new,
                event,
            },
        )
        .await;
    }

    async fn thread_create(&self, ctx: serenity::Context, thread: serenity::GuildChannel) {
        thread_create::thread_create(&ctx, &thread).await;
        self.dispatch_poise_event(&ctx, &poise::Event::ThreadCreate { thread })
            .await;
    }
}
