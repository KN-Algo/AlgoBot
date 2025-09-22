use serenity::{all::CreateCommand, async_trait};

use crate::{
    aliases::Result,
    components::{CommandCtx, InteractiveMessage},
    traits::BotCommand,
    write_state,
};

use crate::commands::events::embed::Embed;
use crate::get_state;
use crate::traits::into_embed::IntoEmbedInteractive;
use crate::{components::EventCtx, traits::Interactable};
use modal_macro::interactive_msg;

#[derive(Clone)]
pub struct State {
    pub page: usize,
    pub max: usize,
}

impl Default for State {
    fn default() -> Self {
        State { page: 0, max: 99 }
    }
}

interactive_msg! {
    <AllEvents handler=Handler ephemeral=true>
        <embed>Embed</embed>
        <row>
            <button id="prev">"<"</button>
            <button id="next">">"</button>
        </row>
    </AllEvents>
}

#[async_trait]
impl HandlerTrait for Handler {
    async fn handle_prev(ctx: &mut EventCtx) -> Result {
        get_state!(ctx, State, state);
        if state.page == 0 {
            return ctx.acknowlage().await;
        }

        state.page -= 1;
        write_state!(ctx, State, state);
        ctx.update_msg::<AllEvents<Handler>>().await
    }
    async fn handle_next(ctx: &mut EventCtx) -> Result {
        get_state!(ctx, State, state);
        if state.page == state.max {
            return ctx.acknowlage().await;
        }

        state.page += 1;
        write_state!(ctx, State, state);
        ctx.update_msg::<AllEvents<Handler>>().await
    }
}

pub struct EventsCommand;

#[async_trait]
impl BotCommand for EventsCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = InteractiveMessage::new_with_state::<AllEvents<Handler>, State>(ctx).await?;
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("List all events in the google calendar")
    }
}
