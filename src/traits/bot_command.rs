use serenity::{all::CreateCommand, async_trait};

use crate::components::CommandCtx;

#[async_trait]
pub trait BotCommand {
    fn register(&self, command: CreateCommand) -> CreateCommand;
    async fn run(&self, ctx: &CommandCtx) -> Result<(), serenity::Error>;
}
