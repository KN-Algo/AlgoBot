use serenity::all::{Context, InteractionId};

use crate::{
    components::{CommandCtx, EventCtx},
    traits::modal::ModalTrait,
};

pub struct Modal<'modal, M: ModalTrait> {
    phantom: std::marker::PhantomData<M>,
    discord_ctx: &'modal Context,
    token: &'modal str,
    id: &'modal InteractionId,
}

impl<'a, M: ModalTrait> Modal<'a, M> {
    pub fn from_command(ctx: &CommandCtx<'a>) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            discord_ctx: ctx.discord_ctx,
            token: &ctx.interaction.token,
            id: &ctx.interaction.id,
        }
    }

    pub fn from_event(ctx: &EventCtx<'a>) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            discord_ctx: ctx.discord_ctx,
            token: &ctx.interaction.token,
            id: &ctx.interaction.id,
        }
    }

    pub async fn execute(self) -> Result<M, serenity::Error> {
        M::execute(self.discord_ctx, self.id, self.token).await
    }
}
