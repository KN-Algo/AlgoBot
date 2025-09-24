use std::{any::Any, sync::Arc};
use tokio::sync::RwLock;

use crate::{components::CommandCtx, traits::state::StateTrait};

pub struct State {
    state: Arc<dyn Any + Send + Sync>,
}

impl State {
    pub async fn init<S: StateTrait + Send + Sync + 'static>(ctx: &CommandCtx<'_>) -> Self {
        Self {
            state: Arc::new(RwLock::new(S::init(ctx).await)),
        }
    }

    pub async fn clone<S: StateTrait + Send + Sync + 'static>(&self) -> Option<S> {
        let arc = self.state.clone();
        let s = arc.downcast::<RwLock<S>>().ok()?;
        Some((s.read().await).clone())
    }

    pub async fn write<S: StateTrait + Send + Sync + 'static>(&self, state: S) -> Option<()> {
        let arc = self.state.clone();
        let s = arc.downcast::<RwLock<S>>().ok()?;
        *(s.write().await) = state;

        Some(())
    }
}
