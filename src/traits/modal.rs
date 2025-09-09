use serenity::{
    all::{Context, InteractionId},
    async_trait,
};

#[async_trait]
pub trait ModalTrait
where
    Self: Sized,
{
    async fn execute(
        ctx: &Context,
        id: &InteractionId,
        token: &str,
    ) -> Result<Self, serenity::Error>;
}
