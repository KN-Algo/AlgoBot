use serenity::{
    all::{Context, CreateInteractionResponseMessage, InteractionId, ModalInteraction},
    async_trait,
};

#[async_trait]
pub trait ModalTrait<'ctx>
where
    Self: Sized,
{
    async fn execute(
        ctx: &Context,
        id: &InteractionId,
        token: &str,
    ) -> Result<Self, serenity::Error>
    where
        'life0: 'ctx;

    fn discord_ctx(&self) -> &Context;
    fn interaction(&self) -> &ModalInteraction;

    fn acknowlage_interaction(&self) -> impl Future<Output = Result<(), serenity::Error>> {
        self.interaction().create_response(
            self.discord_ctx(),
            serenity::all::CreateInteractionResponse::Acknowledge,
        )
    }

    fn simple_response(
        &self,
        msg: impl Into<String>,
    ) -> impl Future<Output = Result<(), serenity::Error>> {
        self.interaction().create_response(
            self.discord_ctx(),
            serenity::all::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(msg)
                    .ephemeral(true),
            ),
        )
    }
}
