use serenity::{
    all::{ComponentInteraction, Context, CreateInteractionResponseMessage},
    async_trait,
};

use crate::components::InteractiveMessage;

#[async_trait]
pub trait InteractiveMessageTrait {
    fn into_msg() -> CreateInteractionResponseMessage;

    async fn handle_event(
        ctx: &Context,
        interaction: &ComponentInteraction,
        msg: &mut InteractiveMessage,
        db: &sqlx::SqlitePool,
    ) -> Result<(), serenity::Error>;
}
