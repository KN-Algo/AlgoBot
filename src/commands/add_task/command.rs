use chrono::{NaiveDate, TimeZone, Utc};
use modal_macro::modal;
use serenity::{
    all::{CreateCommand, EditMessage},
    async_trait,
};

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

        let task = ctx
            .db
            .add_task(
                &result.title,
                &result.description,
                datetime,
                result.interaction.user.id,
            )
            .await?;

        let (mut bot_response, user_response) = match result
            .respond_and_get_response(
                "Respond to this message with @Mentions to add users to the task",
                "Timed Out!",
                ctx.interaction.user.id,
            )
            .await?
        {
            None => return Ok(()),
            Some(msgs) => msgs,
        };

        if user_response.mentions.len() == 0 {
            bot_response
                .edit(ctx, EditMessage::new().content("No users mentioned!"))
                .await?;
            return Ok(());
        }

        let mentions = user_response
            .mentions
            .into_iter()
            .map(|user| user.id)
            .collect();
        ctx.db.add_users_to_task(task.id, mentions).await?;
        bot_response
            .edit(ctx, EditMessage::new().content("Users added to the task!"))
            .await?;
        Ok(())
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Assign a new task to users")
    }
}
