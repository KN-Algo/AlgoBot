use modal_macro::{interactive_msg, Selection, SelectionState};
use serenity::{all::CreateCommand, async_trait};

use crate::{
    aliases::{Result, TypedResult},
    commands::my_tasks::embed::Embed,
    components::{CommandCtx, EventCtx, InteractiveMessage},
    database::Task,
    traits::{BotCommand, Interactable, StateTrait},
};

#[derive(Selection, Clone, Debug)]
enum ReminderWhen {
    #[select_value("1 Day Before")]
    OneDay,
    #[select_value("2 Days Before")]
    TwoDays,
    #[select_value("3 Days Before")]
    ThreeDays,
    #[select_value("5 Days Before")]
    FiveDays,
}

impl From<&ReminderWhen> for chrono::Duration {
    fn from(value: &ReminderWhen) -> Self {
        match value {
            ReminderWhen::OneDay => chrono::Duration::days(1),
            ReminderWhen::TwoDays => chrono::Duration::days(2),
            ReminderWhen::ThreeDays => chrono::Duration::days(3),
            ReminderWhen::FiveDays => chrono::Duration::days(5),
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub page: usize,
    pub max_page: usize,
    pub tasks: Vec<Task>,
}

#[async_trait]
impl StateTrait for State {
    async fn init(ctx: &CommandCtx) -> TypedResult<Self> {
        let mut tasks = ctx.db.get_user_tasks(ctx.interaction.user.id).await?;
        tasks.sort_unstable_by(|a, b| a.deadline.cmp(&b.deadline));
        Ok(Self {
            page: 0,
            max_page: tasks.len(),
            tasks,
        })
    }
}

#[derive(Clone, SelectionState)]
struct ReminderState {
    pub task_id: i64,
    #[selection_state]
    pub when_selection: Vec<ReminderWhen>,
}

#[async_trait]
impl StateTrait for ReminderState {
    async fn init(_ctx: &CommandCtx) -> TypedResult<Self> {
        Ok(Self {
            task_id: 0,
            when_selection: vec![],
        })
    }
}

pub struct MyTasksCommand;

interactive_msg! {
    <MyTasksMsg handler=Handler state=State ephemeral=true>
        <embed>Embed</embed>
        <row>
            <button id="prev">"<"</button>
            <button id="add_reminder" style="secondary">"🔔"</button>
            <button id="check_completed" style="success">"✅"</button>
            <button id="next">">"</button>
        </row>
    </MyTasksMsg>
}

interactive_msg! {
    <AddReminderMsg handler=ReminderHandler state=ReminderState ephemeral=true>
        <row>
            <selection id="when_selection" style=String options=ReminderWhen max_values=4></selection>
        </row>
        <row>
            <button id="submit">"Ok"</button>
        </row>
    </AddReminderMsg>
}

interactive_msg! {
    <EmptyMsg handler=EmptyHander>
        <text>"Done!"</text>
    </EmptyMsg>
}

impl EmptyHanderTrait for EmptyHander {}

#[async_trait]
impl ReminderHandlerTrait for ReminderHandler {
    async fn handle_submit(ctx: &mut EventCtx) -> Result {
        let state = ctx.msg.clone_state::<ReminderState>().await.unwrap();
        ctx.db
            .add_reminder(
                state.task_id,
                ctx.interaction.user.id,
                state
                    .when_selection
                    .iter()
                    .map(|s| s.into())
                    .collect::<Vec<chrono::Duration>>(),
            )
            .await?;
        crate::log!("{:?}", state.when_selection);
        ctx.update_msg::<EmptyMsg<EmptyHander>>().await?;
        ctx.msg.stop();
        Ok(())
    }
}

#[async_trait]
impl HandlerTrait for Handler {
    async fn handle_prev(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        if state.max_page == 0 {
            return ctx.acknowlage().await;
        }
        if state.page == 0 {
            state.page = state.max_page - 1;
        } else {
            state.page -= 1;
        }

        ctx.msg.write_state(state).await;
        ctx.update_msg::<MyTasksMsg<Handler>>().await
    }
    async fn handle_next(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        if state.max_page == 0 {
            return ctx.acknowlage().await;
        }

        if state.page == state.max_page - 1 {
            state.page = 0;
        } else {
            state.page += 1;
        }

        ctx.msg.write_state(state).await;
        ctx.update_msg::<MyTasksMsg<Handler>>().await
    }

    async fn handle_check_completed(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        let task = match state.tasks.get_mut(state.page) {
            None => return ctx.acknowlage().await,
            Some(t) => t,
        };

        task.completed = !task.completed;
        ctx.db.toggle_task_completion(task.id).await?;
        ctx.msg.write_state(state).await;
        ctx.update_msg::<MyTasksMsg<Handler>>().await
    }

    async fn handle_add_reminder(ctx: &mut EventCtx) -> Result {
        let state = ctx.msg.clone_state::<State>().await.unwrap();
        let task = match state.tasks.get(state.page) {
            None => return ctx.acknowlage().await,
            Some(t) => t,
        };

        let mut msg =
            InteractiveMessage::from_event::<AddReminderMsg<ReminderHandler>, ReminderState>(
                ctx,
                ReminderState {
                    task_id: task.id,
                    when_selection: vec![],
                },
            )
            .await?;
        msg.handle_events_from_event(ctx).await
    }
}

#[async_trait]
impl BotCommand for MyTasksCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = InteractiveMessage::new::<MyTasksMsg<Handler>>(ctx).await?;
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Shows tasks assigned to you")
    }
}
