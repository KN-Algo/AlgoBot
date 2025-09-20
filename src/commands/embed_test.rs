use serenity::{
    all::{
        CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponseMessage, Timestamp,
    },
    async_trait,
};

use crate::{
    aliases::Result,
    components::CommandCtx,
    traits::{BotCommand, Interactable, IntoResponse},
};

struct Embed;

impl IntoResponse for Embed {
    fn into_msg(&self) -> CreateInteractionResponseMessage {
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
                    .url("https://example.org"),
            )
    }
}

pub struct EmbedTest;

#[async_trait]
impl BotCommand for EmbedTest {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        ctx.respond(Embed).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Command to test embeds")
    }
}
