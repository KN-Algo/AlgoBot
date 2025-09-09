use crate::{
    components::interactive_message::InteractiveMessage,
    traits::{InteractiveMessageTrait, ModalTrait},
};
use serenity::all::{CacheHttp, ComponentInteraction, Context};
use sqlx::SqlitePool;

pub struct EventCtx<'ctx> {
    pub discord_ctx: &'ctx Context,
    pub interaction: &'ctx ComponentInteraction,
    pub msg: &'ctx mut InteractiveMessage,
    pub db: &'ctx SqlitePool,
}

impl<'ctx> EventCtx<'ctx> {
    pub async fn update_msg<T: InteractiveMessageTrait>(&mut self) -> Result<(), serenity::Error> {
        self.msg
            .update_msg::<T>(self.discord_ctx, self.interaction)
            .await
    }

    pub async fn acknowlage_interaction(&self) -> Result<(), serenity::Error> {
        self.interaction
            .create_response(
                self.discord_ctx,
                serenity::all::CreateInteractionResponse::Acknowledge,
            )
            .await
    }

    pub async fn modal<Modal: ModalTrait>(&self) -> Result<Modal, serenity::Error> {
        Modal::execute(
            self.discord_ctx,
            &self.interaction.id,
            &self.interaction.token,
        )
        .await
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
