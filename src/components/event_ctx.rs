use crate::{
    aliases::Result,
    calendar::CalendarHub,
    components::interactive_message::InteractiveMessage,
    traits::{interactable::Interactable, InteractiveMessageTrait},
};
use serenity::all::{CacheHttp, ComponentInteraction, Context};
use sqlx::SqlitePool;

pub struct EventCtx<'ctx> {
    pub discord_ctx: &'ctx Context,
    pub interaction: &'ctx ComponentInteraction,
    pub msg: &'ctx mut InteractiveMessage,
    pub db: &'ctx SqlitePool,
    pub calendars: &'ctx CalendarHub,
}

impl<'ctx> EventCtx<'ctx> {
    pub async fn update_msg<T: InteractiveMessageTrait>(&mut self) -> Result {
        let embeds = T::with_embeds_event(self).await;
        self.msg
            .update_msg::<T>(self.discord_ctx, self.interaction, embeds)
            .await
    }
}

impl<'ctx> Interactable<'ctx> for EventCtx<'ctx> {
    fn discord_ctx(&self) -> &Context {
        self.discord_ctx
    }

    fn id_token(&self) -> (serenity::all::InteractionId, &str) {
        (self.interaction.id, &self.interaction.token)
    }
}

impl CacheHttp for &mut EventCtx<'_> {
    fn http(&self) -> &serenity::all::Http {
        &self.discord_ctx.http
    }
}

impl<'a> From<&EventCtx<'a>> for &'a Context {
    fn from(value: &EventCtx<'a>) -> Self {
        value.discord_ctx
    }
}

impl<'a> From<&EventCtx<'a>> for &'a ComponentInteraction {
    fn from(value: &EventCtx<'a>) -> Self {
        value.interaction
    }
}

impl<'a> From<&EventCtx<'a>> for &'a SqlitePool {
    fn from(value: &EventCtx<'a>) -> Self {
        value.db
    }
}
