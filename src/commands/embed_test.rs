use serenity::{
    all::{
        CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponseMessage, Timestamp,
    },
    async_trait,
};

use crate::{components::CommandCtx, traits::BotCommand};

pub struct EmbedTest;

#[async_trait]
impl BotCommand for EmbedTest {
    async fn run(&self, ctx: &CommandCtx) -> Result<(), serenity::Error> {
        ctx.interaction
            .create_response(
                ctx,
                serenity::all::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("embed")
                        .embed(
                            CreateEmbed::new()
                                .title("Test Embed")
                                .description("this a test embed")
                                .color(serenity::model::Colour::FOOYOO)
                                .fields([
                                    ("name", "value", false),
                                    ("inline1", "inline", true),
                                    ("inline2", "inline", true),
                                ])
                                .footer(CreateEmbedFooter::new("a footer"))
                                .timestamp(Timestamp::now())
                                .url("example.org"),
                        ),
                ),
            )
            .await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Command to test embeds")
    }
}
