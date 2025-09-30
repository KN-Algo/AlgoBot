use modal_macro::interactive_msg;
use serenity::{all::CreateCommand, async_trait};

use crate::{
    aliases::{Result, TypedResult},
    commands::my_tasks::embed::Embed,
    components::{CommandCtx, EventCtx, InteractiveMessage},
    database::Task,
    traits::{BotCommand, Interactable, StateTrait},
};

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

#[derive(Clone)]
struct ReminderState {
    pub task_id: i64,
    pub when: chrono::Duration,
}

#[async_trait]
impl StateTrait for ReminderState {
    async fn init(_ctx: &CommandCtx) -> TypedResult<Self> {
        Ok(Self {
            task_id: 0,
            when: chrono::Duration::days(1),
        })
    }
}

pub struct MyTasksCommand;

interactive_msg! {
    <MyTasksMsg handler=Handler ephemeral=true>
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
    <AddReminderMsg handler=ReminderHandler ephemeral=true>
        <row>
            <selection id="reminder_when">
                <option id="1d" default=true>"1 Day Before"</option>
                <option id="2d">"2 Days Before"</option>
                <option id="3d">"3 Days Before"</option>
                <option id="5d">"5 Days Before"</option>
            </selection>
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
    async fn handle_1d(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<ReminderState>().await.unwrap();
        state.when = chrono::Duration::days(1);
        ctx.msg.write_state::<ReminderState>(state).await;
        ctx.acknowlage().await
    }

    async fn handle_2d(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<ReminderState>().await.unwrap();
        state.when = chrono::Duration::days(2);
        ctx.msg.write_state::<ReminderState>(state).await;
        ctx.acknowlage().await
    }

    async fn handle_3d(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<ReminderState>().await.unwrap();
        state.when = chrono::Duration::days(3);
        ctx.msg.write_state::<ReminderState>(state).await;
        ctx.acknowlage().await
    }

    async fn handle_5d(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<ReminderState>().await.unwrap();
        state.when = chrono::Duration::days(5);
        ctx.msg.write_state::<ReminderState>(state).await;
        ctx.acknowlage().await
    }

    async fn handle_submit(ctx: &mut EventCtx) -> Result {
        let state = ctx.msg.clone_state::<ReminderState>().await.unwrap();
        ctx.db
            .add_reminder(state.task_id, ctx.interaction.user.id, state.when)
            .await?;
        ctx.update_msg::<EmptyMsg<EmptyHander>>().await?;
        ctx.msg.stop();
        Ok(())
    }
}

#[async_trait]
impl HandlerTrait for Handler {
    async fn handle_prev(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
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

        let mut msg = InteractiveMessage::from_event_with_state::<
            AddReminderMsg<ReminderHandler>,
            ReminderState,
        >(
            ctx,
            ReminderState {
                task_id: task.id,
                when: chrono::Duration::days(1),
            },
        )
        .await?;
        msg.handle_events_from_event(ctx).await
    }
}

#[async_trait]
impl BotCommand for MyTasksCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = InteractiveMessage::new_with_state::<MyTasksMsg<Handler>, State>(ctx).await?;
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Shows tasks assigned to you")
    }
}
