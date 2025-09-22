use serenity::all::{
    ComponentInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Message, MessageFlags,
};
use serenity::futures::StreamExt;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::aliases::{Result, TypedResult};
use crate::components::{CommandCtx, EventCtx};
use crate::traits::InteractiveMessageTrait;

#[macro_export]
macro_rules! get_state {
    ($ctx:ident, $type:ident, $var:ident) => {
        let arc = $ctx.msg.state::<$type>().await.unwrap().clone();
        #[allow(unused_mut)]
        let mut $var = arc.lock().await;
    };
}

pub struct InteractiveMessage {
    msg: Message,
    state: Option<Arc<dyn Any + Send + Sync>>,

    //i hate this
    has_handler_mutated: bool,
    handler: Box<
        dyn for<'a> Fn(
                &'a mut EventCtx<'a>,
            ) -> std::pin::Pin<Box<dyn Future<Output = Result> + Send + 'a>>
            + Send
            + Sync,
    >,
}

impl InteractiveMessage {
    pub async fn new_with_state<
        T: InteractiveMessageTrait + 'static,
        State: Default + Send + Sync + 'static,
    >(
        ctx: &CommandCtx<'_>,
    ) -> TypedResult<Self> {
        let mut s = Self::new::<T>(ctx).await?;
        s.state = Some(Arc::new(Mutex::new(State::default())));
        Ok(s)
    }

    async fn new<T: InteractiveMessageTrait + 'static>(ctx: &CommandCtx<'_>) -> TypedResult<Self> {
        let msg = T::into_msg().embeds(T::with_embeds_command(ctx).await);

        let builder = CreateInteractionResponse::Message(msg);
        ctx.interaction.create_response(ctx, builder).await?;
        let m = ctx.interaction.get_response(ctx.discord_ctx).await?;

        Ok(Self {
            msg: m,
            state: None,
            has_handler_mutated: false,
            handler: Box::new(|c| Box::pin(T::handle_event(c))),
        })
    }

    pub async fn handle_events(&mut self, ctx: &CommandCtx<'_>) -> Result {
        let mut interaction_stream = self
            .msg
            .await_component_interaction(&ctx.discord_ctx.shard)
            .timeout(Duration::from_secs(60))
            .stream();

        while let Some(int) = interaction_stream.next().await {
            //a band-aid fix for my terrible design
            let handler =
                std::mem::replace(&mut self.handler, Box::new(|_| Box::pin(async { Ok(()) })));

            let mut new_ctx = EventCtx {
                discord_ctx: ctx.discord_ctx,
                interaction: &int,
                msg: self,
                db: ctx.db,
                calendars: ctx.calendars,
            };

            handler(&mut new_ctx).await?;

            if self.has_handler_mutated {
                self.has_handler_mutated = false;
                continue;
            }

            self.handler = handler;
        }

        match self.msg.flags {
            Some(flags) => match flags {
                MessageFlags::EPHEMERAL => (), //you just can't delete emphemeral messages
                _ => self.msg.delete(ctx).await?,
            },
            None => (),
        }

        Ok(())
    }

    pub async fn update_msg_modify<T: InteractiveMessageTrait>(
        &mut self,
        ctx: &Context,
        interaction: &ComponentInteraction,
        embeds: Vec<CreateEmbed>,
        modifier: fn(CreateInteractionResponseMessage) -> CreateInteractionResponseMessage,
    ) -> Result {
        let msg = modifier(T::into_msg()).embeds(embeds);
        interaction
            .create_response(ctx, CreateInteractionResponse::UpdateMessage(msg))
            .await?;
        self.handler = Box::new(|c| Box::pin(T::handle_event(c)));
        self.has_handler_mutated = true;
        Ok(())
    }

    pub async fn update_msg<T: InteractiveMessageTrait>(
        &mut self,
        ctx: &Context,
        interaction: &ComponentInteraction,
        embeds: Vec<CreateEmbed>,
    ) -> Result {
        self.update_msg_modify::<T>(ctx, interaction, embeds, |m| m)
            .await
    }

    pub async fn state<State: Default + Send + Sync + 'static>(
        &mut self,
    ) -> Option<Arc<Mutex<State>>> {
        let arc = self.state.clone()?;
        arc.downcast::<Mutex<State>>().ok()
    }
}
