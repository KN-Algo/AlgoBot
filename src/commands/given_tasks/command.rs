use modal_macro::{interactive_msg, modal};
use serenity::{all::CreateCommand, async_trait};

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
    <EditTaskModal title="Add Task" duration=600>
        <row>
            <input id="title" style="short" required=false placeholder="edit">"Title"</input>
        </row>
        <row>
            <input id="description" style="paragraph" required=false placeholder="edit">"Description"</input>
        </row>
        <row>
            <input id="deadline" style="short" required=false placeholder="edit (DD-MM-YY)">"Deadline"</input>
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
    <GivenTasksMsg handler=Handler ephemeral=true>
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

#[async_trait]
impl HandlerTrait for Handler {
    async fn handle_prev(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        if state.page == 0 {
            return ctx.acknowlage().await;
        }

        state.page -= 1;
        ctx.msg.write_state(state).await;
        ctx.update_msg::<GivenTasksMsg<Handler>>().await
    }
    async fn handle_next(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        if state.max_page == 0 || state.page == state.max_page - 1 {
            return ctx.acknowlage().await;
        }

        state.page += 1;
        ctx.msg.write_state(state).await;
        ctx.update_msg::<GivenTasksMsg<Handler>>().await
    }

    async fn handle_add_users(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        let task = match state.tasks.get_mut(state.page) {
            None => return ctx.acknowlage().await,
            Some(t) => t,
        };

        crate::add_users_to_task_from_msg!(ctx, ctx, ctx.interaction.user.id, task);

        ctx.msg.write_state::<State>(state).await;
        Ok(())
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
            task.deadline = match misc::parse_date_dd_mm_yy(&result.deadline) {
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
        let mut msg =
            InteractiveMessage::new_with_state::<GivenTasksMsg<Handler>, State>(ctx).await?;
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Returns tasks you gave to others")
    }
}
