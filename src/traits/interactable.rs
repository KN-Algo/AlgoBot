use serenity::all::{Builder, Context, CreateInteractionResponse, InteractionId};

use crate::{
    aliases::{Result, TypedResult},
    traits::{IntoResponse, ModalTrait},
};

pub trait Interactable<'ctx> {
    fn discord_ctx(&self) -> &Context;
    fn id_token(&self) -> (InteractionId, &str);

    fn acknowlage(&self) -> impl Future<Output = Result> {
        async {
            CreateInteractionResponse::Acknowledge
                .execute(self.discord_ctx(), self.id_token())
                .await?;
            Ok(())
        }
    }

    fn respond(&self, msg: impl IntoResponse, ephemeral: bool) -> impl Future<Output = Result> {
        async move {
            CreateInteractionResponse::Message(msg.into_msg().ephemeral(ephemeral))
                .execute(self.discord_ctx(), self.id_token())
                .await?;
            Ok(())
        }
    }

    fn edit(&self, msg: impl IntoResponse) -> impl Future<Output = Result> {
        async move {
            CreateInteractionResponse::UpdateMessage(msg.into_msg())
                .execute(self.discord_ctx(), self.id_token())
                .await?;
            Ok(())
        }
    }

    fn modal<Modal: ModalTrait<'ctx> + 'ctx>(
        &'ctx self,
    ) -> impl Future<Output = TypedResult<Modal>> {
        Modal::execute(self.discord_ctx(), self.id_token())
    }
}
