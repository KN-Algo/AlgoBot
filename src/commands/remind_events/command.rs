use modal_macro::{interactive_msg, SelectionState};
use serenity::{all::CreateCommand, async_trait};

use crate::{
    aliases::{Result, TypedResult},
    commands::{misc, remind_events::embed::Embed},
    components::{CommandCtx, EventCtx, InteractiveMessage},
    database::{EventReminder, ReminderGroup, ReminderWay},
    traits::{BotCommand, Interactable, StateTrait},
};

pub struct RemindEventsCommand;

#[derive(Clone)]
pub struct State {
    pub reminders: Vec<EventReminder>,
    pub page: u8,
    pub max_page: usize,
}

#[async_trait]
impl StateTrait for State {
    async fn init(ctx: &CommandCtx) -> TypedResult<Self> {
        let reminders = ctx
            .db
            .get_user_event_reminders(ctx.interaction.user.id)
            .await?;

        Ok(Self {
            reminders,
            page: 0,
            max_page: 2,
        })
    }
}

#[derive(Clone, SelectionState)]
struct SelectState {
    #[selection_state]
    pub selection: Vec<ReminderWay>,
    pub group: ReminderGroup,
}

#[async_trait]
impl StateTrait for SelectState {
    async fn init(_ctx: &CommandCtx) -> TypedResult<Self> {
        Ok(Self {
            selection: vec![],
            group: ReminderGroup::Events,
        })
    }
}

interactive_msg! {
    <RemindersMsg handler=RemindersHandler>
        <embed>Embed</embed>
        <row>
            <button id="prev">"<"</button>
            <button id="next">">"</button>
            <button id="add" style="secondary">"+"</button>
            <button id="delete" style="danger">"<"</button>
        </row>
    </RemindersMsg>
}

interactive_msg! {
    <AddRemindEventsMsg handler=AddHandler state=SelectState ephemeral=true>
        <text>"Select reminder type:"</text>
        <row>
            <selection id="selection" options=ReminderWay max_values=2></selection>
        </row>
        <row>
            <button id="submit">"Add"</button>
        </row>
    </AddRemindEventsMsg>
}

interactive_msg! {
    <DeleteRemindMsg handler=DeleteHandler state=SelectState ephemeral=true>
        <text>"Select reminder type:"</text>
        <row>
            <selection id="selection" options=ReminderWay></selection>
        </row>
        <row>
            <button id="delete">"Delete"</button>
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
        let state = ctx.msg.clone_state::<SelectState>().await.unwrap();
        /*
                ctx.db
                    .delete_event_reminder(ctx.interaction.user.id, state.group, state.way)
                    .await?;
        */
        ctx.msg.stop();
        ctx.update_msg::<EmptyMsg<EmptyHandler>>().await
    }
}

impl EmptyHandlerTrait for EmptyHandler {}

#[async_trait]
impl AddHandlerTrait for AddHandler {
    async fn handle_submit(ctx: &mut EventCtx) -> Result {
        let state = ctx.msg.clone_state::<SelectState>().await.unwrap();
        crate::log!("{:?}", state.selection);
        /*
        match state.way {
            ReminderWay::Email => {
                let result = ctx.modal::<EmailModal>().await?;
                if misc::verify_email(&result.email) {
                    result.respond("Done!", true).await?;
                    ctx.db
                        .add_event_reminder(
                            ctx.interaction.user.id,
                            state.way,
                            state.group,
                            Some(result.email),
                        )
                        .await?;
                } else {
                    result.respond("Invalid Email!", true).await?;
                }
            }
            _ => {
                ctx.db
                    .add_event_reminder(ctx.interaction.user.id, state.way, state.group, None)
                    .await?;
                ctx.update_msg::<EmptyMsg<EmptyHandler>>().await?;
            }
        };*/
        ctx.msg.stop();
        Ok(())
    }
}

#[async_trait]
impl BotCommand for RemindEventsCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = InteractiveMessage::new::<AddRemindEventsMsg<AddHandler>>(ctx).await?;
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Subscribe to reminders about events")
    }
}
