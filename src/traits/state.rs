use serenity::async_trait;

use crate::{aliases::TypedResult, components::CommandCtx};

#[async_trait]
pub trait StateTrait: Clone {
    async fn init(ctx: &CommandCtx) -> TypedResult<Self>;
}

#[async_trait]
impl StateTrait for () {
    async fn init(_ctx: &CommandCtx) -> TypedResult<Self> {
        Ok(())
    }
}
