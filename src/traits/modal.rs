use serenity::{
    all::{Context, InteractionId},
    async_trait,
};

#[async_trait]
pub trait ModalTrait<'ctx>
where
    Self: Sized,
{
    async fn execute(
        ctx: &Context,
        id_token: (InteractionId, &str),
    ) -> Result<Self, serenity::Error>
    where
        'life0: 'ctx;
}
