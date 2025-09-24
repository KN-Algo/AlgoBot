use std::time::Duration;

use serenity::{
    all::{
        Builder, Context, CreateInteractionResponse, EditMessage, InteractionId, Message,
        MessageCollector, UserId,
    },
    futures::StreamExt,
};
use tokio::time::sleep;

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

    fn respond_and_get_response(
        &self,
        msg: impl IntoResponse,
        timeout_msg: impl Into<String>,
        from: UserId,
    ) -> impl Future<Output = TypedResult<Option<(Message, Message)>>> {
        async move {
            let discord_ctx = self.discord_ctx();
            let (id, token) = self.id_token();
            CreateInteractionResponse::Message(msg.into_msg())
                .execute(discord_ctx, (id, token))
                .await?;
            let mut iteraction_response = discord_ctx
                .http
                .as_ref()
                .get_original_interaction_response(token)
                .await?;

            let mut collector = MessageCollector::new(&discord_ctx)
                .channel_id(iteraction_response.channel_id)
                .author_id(from)
                .timeout(Duration::from_secs(60))
                .filter(move |msg| {
                    msg.referenced_message
                        .as_ref()
                        .map_or(false, |m| m.id == iteraction_response.id)
                })
                .stream();

            if let Some(response_msg) = collector.next().await {
                return Ok(Some((iteraction_response, response_msg)));
            }

            iteraction_response
                .edit(discord_ctx, EditMessage::new().content(timeout_msg))
                .await?;
            sleep(Duration::from_secs(10)).await;
            iteraction_response.delete(discord_ctx).await?;
            Ok(None)
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
