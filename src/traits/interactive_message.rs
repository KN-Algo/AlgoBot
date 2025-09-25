use serenity::{
    all::{CreateEmbed, CreateInteractionResponseMessage},
    async_trait,
};

use crate::{
    aliases::Result,
    components::{CommandCtx, EventCtx, State},
};

#[async_trait]
pub trait InteractiveMessageTrait {
    fn into_msg() -> CreateInteractionResponseMessage;
    async fn with_embeds_command(ctx: &CommandCtx, state: Option<&State>) -> Vec<CreateEmbed>;
    async fn with_embeds_event(ctx: &EventCtx) -> Vec<CreateEmbed>;
    async fn handle_event(ctx: &mut EventCtx) -> Result;
}
