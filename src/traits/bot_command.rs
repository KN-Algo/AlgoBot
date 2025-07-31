use serenity::{
    all::{CommandInteraction, Context, CreateCommand},
    async_trait,
};

#[async_trait]
pub trait BotCommand {
    fn register(&self, command: CreateCommand) -> CreateCommand;
    async fn run(
        &self,
        ctx: &Context,
        interaction: CommandInteraction,
    ) -> Result<(), serenity::Error>;
}
