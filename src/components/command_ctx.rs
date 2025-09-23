use serenity::all::{CacheHttp, CommandInteraction, Context};

use crate::{calendar::CalendarHub, database::Db, traits::interactable::Interactable};

pub struct CommandCtx<'ctx> {
    pub discord_ctx: &'ctx Context,
    pub interaction: &'ctx CommandInteraction,
    pub db: &'ctx Db,
    pub calendars: &'ctx CalendarHub,
}

impl<'ctx> Interactable<'ctx> for CommandCtx<'ctx> {
    fn discord_ctx(&self) -> &Context {
        self.discord_ctx
    }

    fn id_token(&self) -> (serenity::all::InteractionId, &str) {
        (self.interaction.id, &self.interaction.token)
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

impl<'a> From<&CommandCtx<'a>> for &'a Db {
    fn from(value: &CommandCtx<'a>) -> Self {
        value.db
    }
}
