use serenity::{
    all::{CommandInteraction, Context, CreateCommand},
    async_trait,
};

use crate::err::BotError;

#[async_trait]
pub trait BotCommand {
    fn register(&self) -> CreateCommand;
    async fn run(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<String, BotError>;
}
