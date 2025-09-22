use serenity::{
    all::{CreateEmbed, CreateInteractionResponseMessage},
    async_trait,
};

use crate::{
    aliases::Result,
    components::{CommandCtx, EventCtx},
};

#[async_trait]
pub trait InteractiveMessageTrait {
    fn into_msg() -> CreateInteractionResponseMessage;
    async fn with_embeds_command(ctx: &CommandCtx) -> Vec<CreateEmbed>;
    async fn with_embeds_event(ctx: &EventCtx) -> Vec<CreateEmbed>;
    async fn handle_event(ctx: &mut EventCtx) -> Result;
}
