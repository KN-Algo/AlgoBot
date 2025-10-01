use serenity::{
    all::{CreateEmbed, CreateEmbedFooter},
    async_trait,
};

use crate::{
    commands::summaries::command::State,
    components::{CommandCtx, EventCtx},
    traits::{into_embed::IntoEmbedInteractive, IntoEmbed},
};

pub struct Embed;

impl IntoEmbed for Embed {
    fn into_embed() -> serenity::all::CreateEmbed {
        CreateEmbed::new().color(serenity::model::Colour::MEIBE_PINK)
    }
}

impl Embed {
    fn create(state: &State) -> CreateEmbed {
        let embed = Self::into_embed();
        let summary = match state.summaries.get(state.page) {
            Some(s) => s,
            None => return embed.title("No summaries").field("Empty", "", true),
        };

        embed
            .title(&summary.author)
            .field("", &summary.content, true)
            .footer(CreateEmbedFooter::new(format!(
                "{}/{}",
                state.page + 1,
                state.max_page
            )))
    }
}

#[async_trait]
impl IntoEmbedInteractive for Embed {
    async fn from_command(_ctx: &CommandCtx, state: &crate::components::State) -> CreateEmbed {
        let state = state.clone::<State>().await.unwrap();
        Self::create(&state)
    }

    async fn from_event(ctx: &EventCtx) -> CreateEmbed {
        let state = ctx.msg.clone_state::<State>().await.unwrap();
        Self::create(&state)
    }
}
