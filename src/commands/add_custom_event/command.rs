use serenity::{all::CreateCommand, async_trait};

use crate::{
    aliases::Result,
    commands::misc,
    components::CommandCtx,
    traits::{BotCommand, Interactable},
};

modal_macro::modal! {
    <AddEventModal title="Add Event" duration=600>
        <row>
            <input id="summary" style="paragraph">"Event Summary"</input>
        </row>
        <row>
            <input id="when" style="short" min_len=14 max_len=14 placeholder="HH:MM DD-MM-YY">"Event start time"</input>
        </row>
    </AddEventModal>
}

pub struct AddEventCommand;

#[async_trait]
impl BotCommand for AddEventCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let result = ctx.modal::<AddEventModal>().await?;

        let datetime = match misc::parse_date_dd_mm_yy(&result.when, "%H:%M %d-%m-%y") {
            Ok(d) => d,
            Err(_) => return result.respond("Invalid Date!", true).await,
        };

        ctx.db.add_custom_event(&result.summary, datetime).await?;
        result.respond("Done!", true).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Add a custom event")
    }
}
