use serenity::{all::CreateCommand, async_trait};

use crate::response::{Interaction, Response};

#[async_trait]
pub trait BotCommand {
    fn register(&self) -> CreateCommand;
    async fn run(&self, interaction: Interaction<'async_trait>) -> serenity::Result<Response>;
}
