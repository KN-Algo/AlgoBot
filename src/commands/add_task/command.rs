use chrono::{NaiveDate, TimeZone, Utc};
use modal_macro::modal;
use serenity::{all::CreateCommand, async_trait};

use crate::{
    aliases::Result,
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
            <input id="deadline" style="short">"Deadline in DD-MM-YY format"</input>
        </row>
    </AddTaskModal>
}

pub struct AddTaskCommand;

#[async_trait]
impl BotCommand for AddTaskCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let result = ctx.modal::<AddTaskModal>().await?;
        let naive = match NaiveDate::parse_from_str(&result.deadline, "%d-%m-%y") {
            Ok(d) => d,
            Err(_) => return ctx.respond("invalid deadline date", true).await,
        };
        let naive_date = naive.and_hms_opt(0, 0, 0).unwrap();
        let datetime = Utc.from_utc_datetime(&naive_date);

        ctx.db
            .add_task(&result.title, &result.description, datetime)
            .await?;

        ctx.respond("Task Added!", true).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Assign a new task to users")
    }
}
