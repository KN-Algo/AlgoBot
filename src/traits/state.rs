use serenity::async_trait;

use crate::components::CommandCtx;

#[async_trait]
pub trait StateTrait: Clone {
    async fn init(ctx: &CommandCtx) -> Self;
}
