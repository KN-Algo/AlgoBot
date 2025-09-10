use serenity::{
    all::{Context, InteractionId},
    async_trait,
};

use crate::aliases::TypedResult;

#[async_trait]
pub trait ModalTrait<'ctx>
where
    Self: Sized,
{
    async fn execute(ctx: &Context, id_token: (InteractionId, &str)) -> TypedResult<Self>
    where
        'life0: 'ctx;
}
