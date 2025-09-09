use serenity::{all::CreateInteractionResponseMessage, async_trait};

use crate::components::EventCtx;

#[async_trait]
pub trait InteractiveMessageTrait {
    fn into_msg() -> CreateInteractionResponseMessage;
    async fn handle_event(ctx: &mut EventCtx) -> Result<(), serenity::Error>;
}
