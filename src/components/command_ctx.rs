use serenity::all::{
    CacheHttp, CommandInteraction, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use sqlx::SqlitePool;

pub struct CommandCtx<'ctx> {
    pub discord_ctx: &'ctx Context,
    pub interaction: &'ctx CommandInteraction,
    pub db: &'ctx SqlitePool,
}

impl<'ctx> CommandCtx<'ctx> {
    pub async fn simple_response(&self, msg: impl Into<String>) -> Result<(), serenity::Error> {
        let msg = CreateInteractionResponseMessage::new().content(msg);
        let builder = CreateInteractionResponse::Message(msg);
        self.interaction.create_response(self, builder).await
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
