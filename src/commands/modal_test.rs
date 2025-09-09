use crate::components::CommandCtx;
use crate::log;
use crate::traits::BotCommand;
use modal_macro::modal;
use serenity::{all::CreateCommand, async_trait};

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
    async fn run(&self, ctx: &CommandCtx) -> Result<(), serenity::Error> {
        let modal = ctx.modal::<CoolModal>().await?;
        log!("Modal results {:?} {:?}", modal.name, modal.email);
        Ok(())
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Command to test Modals")
    }
}
