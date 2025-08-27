use crate::log;
use crate::traits::BotCommand;
use crate::traits::Modal;
use modal_macro::modal;
use serenity::{
    all::{CommandInteraction, Context, CreateCommand},
    async_trait,
};
use sqlx::SqlitePool;

pub struct ModalTest;

modal! {
    <CoolModal title="Title" duration=600>
        <row>
            <input
                id="name"
                style="short"
                placeholder="your name"
                min_len=10
                max_len=125>"name"</input>
        </row>
        <row>
            <input id="email" style="paragraph"
            placeholder="a really long email">"email"</input>
        </row>
    </CoolModal>
}

#[async_trait]
impl BotCommand for ModalTest {
    async fn run(
        &self,
        ctx: &Context,
        interaction: CommandInteraction,
        _db: &SqlitePool,
    ) -> Result<(), serenity::Error> {
        let modal = CoolModal::execute(ctx, &interaction).await?;
        log!("Modal results {:?} {:?}", modal.name, modal.email);
        Ok(())
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Command to test Modals")
    }
}
