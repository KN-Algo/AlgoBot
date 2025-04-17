use crate::bot_command::BotCommand;
use crate::modal;
use crate::response::Interaction;
use crate::response::Respond;
use crate::response::Response;
use serenity::{all::CreateCommand, async_trait};

pub struct ModalTest;

modal! {
    CoolModal("Test Modal", 1200) {
        testfield => short_field("name"),
        testfield1 => short_field("email"),
    }
}

#[async_trait]
impl BotCommand for ModalTest {
    async fn run(&self, interaction: Interaction<'async_trait>) -> serenity::Result<Response> {
        let modal = interaction.modal::<CoolModal>().await?;
        let s = format!("{:?} {:?}", modal.testfield, modal.testfield1);
        modal.respond(s)
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("modal_test").description("Modal Test")
    }
}
