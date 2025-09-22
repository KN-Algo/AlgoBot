use crate::calendar::CalendarHub;
use crate::calendar::Event;
use crate::commands::events::command::State;
use crate::components::CommandCtx;
use crate::components::EventCtx;
use crate::get_state;
use crate::write_state;
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
            .fields(vec![("When", event.start.to_string(), false)])
            .title(event.summary.clone())
    }

    async fn create(calendars: &CalendarHub, state: &mut State) -> CreateEmbed {
        let calendar = calendars.get_calendar("KN ALGO").await.unwrap();
        let embed = Self::into_embed();
        let embed = match calendar.events.get(state.page) {
            None => embed.field("Event", "No events", false),
            Some(event) => Self::format_event(embed, event),
        };

        let size = calendar.events.len();
        if state.max != size {
            state.max = size;
        }
        embed.footer(CreateEmbedFooter::new(format!(
            "{}/{}",
            state.page + 1,
            size
        )))
    }
}

#[async_trait]
impl IntoEmbedInteractive for Embed {
    async fn from_command(ctx: &CommandCtx) -> CreateEmbed {
        Self::create(&ctx.calendars, &mut State::default()).await
    }

    async fn from_event(ctx: &mut EventCtx) -> CreateEmbed {
        get_state!(ctx, State, state);
        let e = Self::create(ctx.calendars, &mut state).await;
        write_state!(ctx, State, state);
        e
    }
}
