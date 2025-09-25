use modal_macro::modal;
use serenity::{all::CreateCommand, async_trait};

use crate::{
    aliases::Result,
    commands::misc,
    components::CommandCtx,
    traits::{BotCommand, Interactable},
};

modal! {
    <AddTaskModal title="Add Task" duration=600>
        <row>
            <input id="title" style="short">"Title"</input>
        </row>
        <row>
            <input id="description" style="paragraph">"Description"</input>
        </row>
        <row>
            <input id="deadline" style="short" placeholder="DD-MM-YY">"Deadline"</input>
        </row>
    </AddTaskModal>
}

pub struct AddTaskCommand;

#[async_trait]
impl BotCommand for AddTaskCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let result = ctx.modal::<AddTaskModal>().await?;
        let datetime = misc::parse_date_dd_mm_yy(&result.deadline)?;
        let mut task = ctx
            .db
            .add_task(
                &result.title,
                &result.description,
                datetime,
                result.interaction.user.id,
            )
            .await?;

        crate::add_users_to_task_from_msg!(result, ctx, ctx.interaction.user.id, task);
        Ok(())
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Assign a new task to users")
    }
}
