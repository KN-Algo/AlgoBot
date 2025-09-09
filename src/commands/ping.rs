use serenity::{all::CreateCommand, async_trait};

use crate::{components::CommandCtx, traits::bot_command::BotCommand};

pub struct Ping;

#[async_trait]
impl BotCommand for Ping {
    async fn run(&self, ctx: &CommandCtx) -> Result<(), serenity::Error> {
        ctx.simple_response("pong!").await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Check if the bot is responding")
    }
}
