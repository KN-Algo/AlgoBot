use serenity::{
    all::{CreateEmbed, CreateInteractionResponseMessage},
    async_trait,
};

use crate::{
    components::{CommandCtx, EventCtx, State},
    traits::IntoResponse,
};

pub trait IntoEmbed {
    fn into_embed() -> CreateEmbed;
}

impl<T: IntoEmbed> IntoResponse for T {
    fn into_msg(&self) -> serenity::all::CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new().embed(T::into_embed())
    }
}

#[async_trait]
pub trait IntoEmbedInteractive {
    async fn from_command(ctx: &CommandCtx, state: Option<&mut State>) -> CreateEmbed;
    async fn from_event(ctx: &mut EventCtx) -> CreateEmbed;
}
