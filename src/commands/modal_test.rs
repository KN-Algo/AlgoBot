use crate::bot_command::BotCommand;
use crate::modal::Modal;
use crate::response::ModalResponse;
use crate::response::Response;
use serenity::{
    all::{CommandInteraction, Context, CreateCommand},
    async_trait,
};

pub struct ModalTest;

#[derive(Modal)]
#[modal("Test", 600)]
struct TestModal {
    #[short_field("The Mog")]
    the_mog: String,
    #[paragraph_field("Imposter")]
    impostor: String,
}

#[async_trait]
impl BotCommand for ModalTest {
    async fn run(&self, ctx: &Context, command: &CommandInteraction) -> serenity::Result<Response> {
        let (modal, response) = TestModal::execute(ctx, command.id, &command.token).await?;
        let s = format!("{:?} {:?}", modal.the_mog, modal.impostor);

        Ok(Response::from_modal(response, s))
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("modal_test").description("Modal Test")
    }
}
