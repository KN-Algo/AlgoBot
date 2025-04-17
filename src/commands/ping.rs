use crate::bot_command::BotCommand;
use crate::log;
use crate::response::Respond;
use crate::response::{Interaction, Response};
use serenity::all::CreateCommand;
use serenity::async_trait;

pub struct Ping;

#[async_trait]
impl BotCommand for Ping {
    async fn run(&self, interaction: Interaction<'async_trait>) -> serenity::Result<Response> {
        log!("{} pinged", interaction.user.tag());
        interaction.respond("pong!")
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("ping").description("Send ping")
    }
}
