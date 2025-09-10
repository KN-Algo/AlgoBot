use serenity::all::{Builder, Context, CreateInteractionResponse, InteractionId};

use crate::traits::{IntoResponse, ModalTrait};

pub trait Interactable<'ctx> {
    fn discord_ctx(&self) -> &Context;
    fn id_token(&self) -> (InteractionId, &str);

    fn acknowlage_interaction(&self) -> impl Future<Output = Result<(), serenity::Error>> {
        CreateInteractionResponse::Acknowledge.execute(self.discord_ctx(), self.id_token())
    }

    fn respond(&self, msg: impl IntoResponse) -> impl Future<Output = Result<(), serenity::Error>> {
        CreateInteractionResponse::Message(msg.into_msg())
            .execute(self.discord_ctx(), self.id_token())
    }

    fn modal<Modal: ModalTrait<'ctx> + 'ctx>(
        &'ctx self,
    ) -> impl Future<Output = Result<Modal, serenity::Error>> {
        Modal::execute(self.discord_ctx(), self.id_token())
    }
}
