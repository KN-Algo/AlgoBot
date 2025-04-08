use crate::bot_command::BotCommand;
use crate::log;
use crate::response::Response;
use serenity::all::{CommandInteraction, Context, CreateCommand};
use serenity::async_trait;

pub struct Ping;

#[async_trait]
impl BotCommand for Ping {
    async fn run(&self, _: &Context, command: &CommandInteraction) -> serenity::Result<Response> {
        log!("{} pinged", command.user.tag());
        Ok(Response::from_command(command, "pong!"))
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("ping").description("Send ping")
    }
}
