use crate::calendar::CalendarHub;
use crate::calendar::Event;
use crate::commands::events::command::State;
use crate::components::CommandCtx;
use crate::components::EventCtx;
use serenity::all::CreateEmbedFooter;
use serenity::{all::CreateEmbed, async_trait};

use crate::traits::{into_embed::IntoEmbedInteractive, IntoEmbed};

pub struct Embed;

impl IntoEmbed for Embed {
    fn into_embed() -> CreateEmbed {
        CreateEmbed::new().color(serenity::model::Color::MEIBE_PINK)
    }
}

impl Embed {
    fn format_event(embed: CreateEmbed, event: &Event) -> CreateEmbed {
        embed
            .fields(vec![(
                "When",
                format!("<t:{}:F>", event.start.timestamp()),
                false,
            )])
            .title(event.summary.clone())
    }

    async fn create(calendars: &CalendarHub, state: &State) -> CreateEmbed {
        let calendar = calendars.get_calendar("KN ALGO").await.unwrap();
        let embed = Self::into_embed();
        let embed = match calendar.events.get(state.page) {
            None => embed.field("Event", "No events", false),
            Some(event) => Self::format_event(embed, event),
        };

        embed.footer(CreateEmbedFooter::new(format!(
            "{}/{}",
            state.page + 1,
            state.max
        )))
    }
}

#[async_trait]
impl IntoEmbedInteractive for Embed {
    async fn from_command(
        ctx: &CommandCtx,
        state: Option<&crate::components::State>,
    ) -> CreateEmbed {
        let state = state.unwrap().clone::<State>().await;
        Self::create(&ctx.calendars, &state.unwrap()).await
    }

    async fn from_event(ctx: &mut EventCtx) -> CreateEmbed {
        let state = ctx.msg.clone_state().await.unwrap();
        let e = Self::create(ctx.calendars, &state).await;
        e
    }
}
