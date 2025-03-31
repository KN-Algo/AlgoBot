use crate::{bot_command::BotCommand, err::BotError};
use serenity::{
    all::{CommandInteraction, Context, CreateCommand, CreateQuickModal},
    async_trait,
};
use std::time::Duration;

pub struct ModalTest;

#[async_trait]
impl BotCommand for ModalTest {
    async fn run(&self, ctx: &Context, command: &CommandInteraction) -> Result<String, BotError> {
        let modal = CreateQuickModal::new("Test Modal")
            .timeout(Duration::from_secs(600))
            .short_field("The Mog")
            .short_field("Imposter")
            .paragraph_field("things and stuffs");

        let response = command.quick_modal(ctx, modal).await.unwrap();
        let inputs = response.unwrap().inputs;
        let mut s = String::new();
        for input in inputs {
            s.push_str(&input);
        }

        Ok(s)
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new("modal_test").description("Modal Test")
    }
}
