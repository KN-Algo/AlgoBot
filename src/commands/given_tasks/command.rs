use modal_macro::{interactive_msg, modal, SelectionState};
use serenity::{
    all::{CreateCommand, UserId},
    async_trait,
};

use crate::{
    aliases::{Result, TypedResult},
    commands::{given_tasks::embed::Embed, misc},
    components::{CommandCtx, EventCtx, InteractiveMessage},
    database::Task,
    error::BotError,
    traits::{BotCommand, Interactable, StateTrait},
};

pub struct GivenTasksCommand;

modal! {
    <EditTaskModal title="Edit Task" duration=600>
        <row>
            <input id="title" style="short" required=false placeholder="edit">"Title"</input>
        </row>
        <row>
            <input id="description" style="paragraph" required=false placeholder="edit">"Description"</input>
        </row>
        <row>
            <input id="deadline" style="short" required=false placeholder="edit (DD-MM-YY)" min_len=8 max_len=8>"Deadline"</input>
        </row>
    </EditTaskModal>
}

#[derive(Clone, Debug)]
pub struct State {
    pub page: usize,
    pub max_page: usize,
    pub tasks: Vec<Task>,
}

#[async_trait]
impl StateTrait for State {
    async fn init(ctx: &CommandCtx) -> TypedResult<Self> {
        let mut tasks = ctx.db.get_given_tasks(ctx.interaction.user.id).await?;

        tasks.sort_unstable_by(|a, b| a.deadline.cmp(&b.deadline));
        Ok(Self {
            page: 0,
            max_page: tasks.len(),
            tasks,
        })
    }
}

interactive_msg! {
    <GivenTasksMsg handler=Handler state=State ephemeral=true>
        <embed>Embed</embed>
        <row>
            <button id="prev">"<"</button>
            <button id="add_users" style="secondary">"+ Add Users"</button>
            <button id="edit" style="success">"✏️"</button>
            <button id="delete" style="danger">"🗑️"</button>
            <button id="next">">"</button>
        </row>
    </GivenTasksMsg>
}

#[derive(SelectionState, Clone)]
struct AddUserState {
    pub task_id: i64,
    #[selection_state]
    pub users: Vec<UserId>,
}

#[async_trait]
impl StateTrait for AddUserState {
    async fn init(_ctx: &CommandCtx) -> TypedResult<Self> {
        Ok(Self {
            task_id: 0,
            users: vec![],
        })
    }
}

interactive_msg! {
    <AddUsers handler=AddUserHandler state=AddUserState ephemeral=true>
        <row>
            <selection id="users" style=User max_values=25></selection>
        </row>
        <row>
            <button id="submit">"Ok"</button>
        </row>
    </AddUsers>
}

interactive_msg! {
    <EmptyMsg handler=EmptyHandler ephemeral=true>
        <text>"Done!"</text>
    </EmptyMsg>
}

impl EmptyHandlerTrait for EmptyHandler {}

#[async_trait]
impl AddUserHandlerTrait for AddUserHandler {
    async fn handle_submit(ctx: &mut EventCtx) -> Result {
        let state = ctx.msg.clone_state::<AddUserState>().await.unwrap();
        ctx.db.add_users_to_task(state.task_id, state.users).await?;
        ctx.msg.stop();
        ctx.update_msg::<EmptyMsg<EmptyHandler>>().await
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
        ctx.update_msg::<GivenTasksMsg<Handler>>().await
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
        ctx.update_msg::<GivenTasksMsg<Handler>>().await
    }

    async fn handle_add_users(ctx: &mut EventCtx) -> Result {
        let state = ctx.msg.clone_state::<State>().await.unwrap();
        let task = match state.tasks.get(state.page) {
            None => return ctx.acknowlage().await,
            Some(t) => t,
        };

        let mut msg = InteractiveMessage::from_event::<AddUsers<AddUserHandler>, AddUserState>(
            ctx,
            AddUserState {
                users: vec![],
                task_id: task.id,
            },
        )
        .await?;
        msg.handle_events_from_event(ctx).await
    }

    async fn handle_edit(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        let task = match state.tasks.get_mut(state.page) {
            None => return ctx.acknowlage().await,
            Some(t) => t,
        };

        let result = ctx.modal::<EditTaskModal>().await?;

        if !result.title.trim().is_empty() {
            task.title = result.title.clone();
        }

        if !result.description.trim().is_empty() {
            task.description = result.description.clone();
        }

        if !result.deadline.trim().is_empty() {
            task.deadline = match misc::parse_date_dd_mm_yy(&result.deadline, "%d-%m-%y") {
                Ok(d) => d,
                Err(e) => match e {
                    BotError::ChronoParse(_) => return result.respond("invalid date!", true).await,
                    _ => unreachable!(),
                },
            }
        }

        ctx.db.edit_task(task).await?;

        ctx.msg.write_state(state).await;

        result.respond("Edit Successful!", true).await
    }

    async fn handle_delete(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        let task = match state.tasks.get(state.page) {
            None => return ctx.acknowlage().await,
            Some(t) => t,
        };

        ctx.db.delete_task(task.id).await?;
        state.tasks.remove(state.page);
        state.max_page -= 1;
        if state.page == state.max_page && state.page != 0 {
            state.page -= 1;
        }
        ctx.msg.write_state::<State>(state).await;
        ctx.update_msg::<GivenTasksMsg<Handler>>().await
    }
}

#[async_trait]
impl BotCommand for GivenTasksCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = InteractiveMessage::new::<GivenTasksMsg<Handler>>(ctx).await?;
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Shows tasks you gave to others")
    }
}
