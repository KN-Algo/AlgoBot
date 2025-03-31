use crate::log;
use crate::{bot_command::BotCommand, err::BotError};
use serenity::all::{CommandInteraction, Context, CreateCommand};
use serenity::async_trait;

pub struct Ping;

#[async_trait]
impl BotCommand for Ping {
    async fn run(&self, _: &Context, command: &CommandInteraction) -> Result<String, BotError> {
        log!("{} pinged", command.user.tag());
        Ok("pong".to_string())
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("ping").description("Send ping")
    }
}
