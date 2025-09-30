use serenity::{all::CreateEmbed, async_trait};

use crate::{
    commands::remind_events::command::State,
    components::{CommandCtx, EventCtx},
    database::ReminderGroup,
    traits::{into_embed::IntoEmbedInteractive, IntoEmbed},
};

pub struct Embed;

impl IntoEmbed for Embed {
    fn into_embed() -> CreateEmbed {
        CreateEmbed::new().color(serenity::model::Colour::MEIBE_PINK)
    }
}

impl Embed {
    fn create(state: &State) -> CreateEmbed {
        let embed = Self::into_embed();

        let embed = embed.title(ReminderGroup::from_u8(state.page).unwrap().to_string());
        let mut rs = state
            .reminders
            .iter()
            .filter(|r| (r.group as u8) == state.page)
            .map(|r| format!("{}\n", r.way))
            .collect::<String>();

        if rs.is_empty() {
            rs = "No reminders!".to_owned();
        }

        embed.field("Reminders", rs, false)
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
