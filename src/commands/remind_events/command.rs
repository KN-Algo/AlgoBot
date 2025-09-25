use modal_macro::interactive_msg;
use serenity::{
    all::{CreateCommand, CreateEmbed},
    async_trait,
};

use crate::{
    aliases::{Result, TypedResult},
    commands::misc,
    components::{CommandCtx, EventCtx, InteractiveMessage},
    database::ReminderWay,
    traits::{into_embed::IntoEmbedInteractive, BotCommand, Interactable, StateTrait},
};

pub struct RemindEventsCommand;

#[derive(Clone)]
struct State {
    way: ReminderWay,
    email: Option<String>,
}

#[async_trait]
impl StateTrait for State {
    async fn init(ctx: &CommandCtx) -> TypedResult<Self> {
        let (way, email) = match ctx
            .db
            .get_user_event_reminder(ctx.interaction.user.id)
            .await?
        {
            Some(r) => (r.way, r.email),
            None => (ReminderWay::DiscordPing, None),
        };
        Ok(Self { way, email })
    }
}

struct Embed;

#[async_trait]
impl IntoEmbedInteractive for Embed {
    async fn from_command(
        _ctx: &CommandCtx,
        state: Option<&crate::components::State>,
    ) -> CreateEmbed {
        let state = state.unwrap().clone::<State>().await.unwrap();
        let str = match state.way {
            ReminderWay::DiscordPing => "Discord Ping",
            ReminderWay::DirectMsg => "Direct Message",
            ReminderWay::Email => &format!("Email: {}", state.email.unwrap()),
        };

        CreateEmbed::new()
            .color(serenity::model::Colour::MEIBE_PINK)
            .field("You will be reminded via:", str, false)
    }
    async fn from_event(_ctx: &EventCtx) -> CreateEmbed {
        CreateEmbed::new()
    }
}

interactive_msg! {
    <RemindEventsMsg handler=Handler ephemeral=true>
        <text>"Select reminder type:"</text>
        <row>
            <selection id="selection">
                <option id="ping" default=true>"Discord Ping"</option>
                <option id="dm">"Direct Message"</option>
                <option id="email">"Email"</option>
            </selection>
        </row>
        <row>
            <button id="submit">"Ok"</button>
        </row>
    </RemindEventsMsg>
}

interactive_msg! {
    <DeleteRemindMsg handler=DeleteHandler ephemeral=true>
        <embed>Embed</embed>
        <row>
            <button id="delete">"Unsubscribe"</button>
        </row>
    </DeleteRemindMsg>
}

interactive_msg! {
    <EmptyMsg handler=EmptyHandler ephemeral=true>
        <text>"Done!"</text>
    </EmptyMsg>
}

modal_macro::modal! {
    <EmailModal title="Enter your Email" duration=300>
        <row>
            <input id="email" placeholder="email@example.org" style="short">"Email"</input>
        </row>
    </EmailModal>
}

#[async_trait]
impl DeleteHandlerTrait for DeleteHandler {
    async fn handle_delete(ctx: &mut EventCtx) -> Result {
        ctx.db
            .delete_event_reminder(ctx.interaction.user.id)
            .await?;
        ctx.update_msg::<EmptyMsg<EmptyHandler>>().await
    }
}

impl EmptyHandlerTrait for EmptyHandler {}

#[async_trait]
impl HandlerTrait for Handler {
    async fn handle_ping(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        state.way = ReminderWay::DiscordPing;
        ctx.msg.write_state(state).await;
        ctx.acknowlage().await
    }

    async fn handle_dm(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        state.way = ReminderWay::DirectMsg;
        ctx.msg.write_state(state).await;
        ctx.acknowlage().await
    }
    async fn handle_email(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        state.way = ReminderWay::Email;
        ctx.msg.write_state(state).await;
        ctx.acknowlage().await
    }
    async fn handle_submit(ctx: &mut EventCtx) -> Result {
        let state = ctx.msg.clone_state::<State>().await.unwrap();

        match state.way {
            ReminderWay::Email => {
                let result = ctx.modal::<EmailModal>().await?;
                if misc::verify_email(&result.email) {
                    result.respond("Done!", true).await?;
                    ctx.db
                        .add_event_reminder(ctx.interaction.user.id, state.way, Some(result.email))
                        .await?;
                } else {
                    result.respond("Invalid Email!", true).await?;
                }
            }
            _ => {
                ctx.db
                    .add_event_reminder(ctx.interaction.user.id, state.way, None)
                    .await?;
                ctx.update_msg::<EmptyMsg<EmptyHandler>>().await?;
            }
        };
        ctx.msg.stop();
        Ok(())
    }
}

#[async_trait]
impl BotCommand for RemindEventsCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = match ctx
            .db
            .get_user_event_reminder(ctx.interaction.user.id)
            .await?
        {
            None => {
                InteractiveMessage::new_with_state::<RemindEventsMsg<Handler>, State>(ctx).await?
            }
            Some(_) => {
                InteractiveMessage::new_with_state::<DeleteRemindMsg<DeleteHandler>, State>(ctx)
                    .await?
            }
        };
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Subscribe to reminders about events")
    }
}
