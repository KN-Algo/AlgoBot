use serenity::{
    all::{CommandInteraction, Context},
    async_trait,
};

#[async_trait]
pub trait Modal
where
    Self: Sized,
{
    async fn execute(
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<Self, serenity::Error>;
}
