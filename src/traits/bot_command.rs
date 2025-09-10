use serenity::{all::CreateCommand, async_trait};

use crate::{aliases::Result, components::CommandCtx};

#[async_trait]
pub trait BotCommand {
    fn register(&self, command: CreateCommand) -> CreateCommand;
    async fn run(&self, ctx: &CommandCtx) -> Result;
}
