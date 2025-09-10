use serenity::all::{
    CacheHttp, CommandInteraction, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use sqlx::SqlitePool;

use crate::traits::ModalTrait;

pub struct CommandCtx<'ctx> {
    pub discord_ctx: &'ctx Context,
    pub interaction: &'ctx CommandInteraction,
    pub db: &'ctx SqlitePool,
}

impl<'ctx> CommandCtx<'ctx> {
    pub fn simple_response(
        &self,
        msg: impl Into<String>,
    ) -> impl Future<Output = Result<(), serenity::Error>> {
        let msg = CreateInteractionResponseMessage::new().content(msg);
        let builder = CreateInteractionResponse::Message(msg);
        self.interaction.create_response(self, builder)
    }

    pub fn modal<Modal: ModalTrait + 'ctx>(
        &self,
    ) -> impl Future<Output = Result<Modal, serenity::Error>> {
        Modal::execute(
            self.discord_ctx,
            &self.interaction.id,
            &self.interaction.token,
        )
    }

    pub fn acknowlage_interaction(&self) -> impl Future<Output = Result<(), serenity::Error>> {
        self.interaction.create_response(
            self.discord_ctx,
            serenity::all::CreateInteractionResponse::Acknowledge,
        )
    }
}

impl CacheHttp for CommandCtx<'_> {
    fn http(&self) -> &serenity::all::Http {
        &self.discord_ctx.http
    }
}

impl<'a> From<&CommandCtx<'a>> for &'a Context {
    fn from(value: &CommandCtx<'a>) -> Self {
        value.discord_ctx
    }
}

impl<'a> From<&CommandCtx<'a>> for &'a CommandInteraction {
    fn from(value: &CommandCtx<'a>) -> Self {
        value.interaction
    }
}

impl<'a> From<&CommandCtx<'a>> for &'a SqlitePool {
    fn from(value: &CommandCtx<'a>) -> Self {
        value.db
    }
}
