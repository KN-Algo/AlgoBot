use serenity::{
    all::{
        CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    async_trait,
};

use crate::traits::bot_command::BotCommand;

pub struct Ping;

#[async_trait]
impl BotCommand for Ping {
    async fn run(
        &self,
        ctx: &Context,
        interaction: CommandInteraction,
    ) -> Result<(), serenity::Error> {
        let msg = CreateInteractionResponseMessage::new().content("pong!");
        let builder = CreateInteractionResponse::Message(msg);
        interaction.create_response(ctx, builder).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Check if the bot is responding")
    }
}
