use serenity::all::{CreateEmbed, CreateInteractionResponseMessage};

use crate::{
    components::{CommandCtx, EventCtx},
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

pub trait IntoEmbedInteractive {
    fn from_command(ctx: &CommandCtx) -> CreateEmbed;
    fn from_event(ctx: &EventCtx) -> CreateEmbed;
}
