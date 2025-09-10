use serenity::all::{Builder, Context, CreateInteractionResponse, InteractionId};

use crate::{
    aliases::{Result, TypedResult},
    traits::{IntoResponse, ModalTrait},
};

pub trait Interactable<'ctx> {
    fn discord_ctx(&self) -> &Context;
    fn id_token(&self) -> (InteractionId, &str);

    fn acknowlage_interaction(&self) -> impl Future<Output = Result> {
        CreateInteractionResponse::Acknowledge.execute(self.discord_ctx(), self.id_token())
    }

    fn respond(&self, msg: impl IntoResponse) -> impl Future<Output = Result> {
        CreateInteractionResponse::Message(msg.into_msg())
            .execute(self.discord_ctx(), self.id_token())
    }

    fn modal<Modal: ModalTrait<'ctx> + 'ctx>(
        &'ctx self,
    ) -> impl Future<Output = TypedResult<Modal>> {
        Modal::execute(self.discord_ctx(), self.id_token())
    }
}
