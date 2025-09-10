use crate::traits::Interactable;
use crate::{components::CommandCtx, traits::bot_command::BotCommand};
use serenity::{all::CreateCommand, async_trait};

pub struct Ping;

#[async_trait]
impl BotCommand for Ping {
    async fn run(&self, ctx: &CommandCtx) -> Result<(), serenity::Error> {
        ctx.respond("pong!").await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Check if the bot is responding")
    }
}
