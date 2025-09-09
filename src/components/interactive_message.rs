use serenity::all::{ComponentInteraction, Context, CreateInteractionResponse, Message};
use serenity::futures::StreamExt;
use std::time::Duration;

use crate::components::{CommandCtx, EventCtx};
use crate::traits::InteractiveMessageTrait;

pub struct InteractiveMessage {
    msg: Message,

    //i hate this
    has_handler_mutated: bool,
    handler: Box<
        dyn for<'a> Fn(
                &'a mut EventCtx<'a>,
            ) -> std::pin::Pin<
                Box<dyn Future<Output = Result<(), serenity::Error>> + Send + 'a>,
            > + Send
            + Sync,
    >,
}

impl InteractiveMessage {
    pub async fn new<T: InteractiveMessageTrait + 'static>(
        ctx: &CommandCtx<'_>,
    ) -> Result<Self, serenity::Error> {
        let msg = T::into_msg().ephemeral(true);

        let builder = CreateInteractionResponse::Message(msg);
        ctx.interaction.create_response(ctx, builder).await?;
        let m = ctx.interaction.get_response(ctx.discord_ctx).await?;

        Ok(Self {
            msg: m,
            has_handler_mutated: false,
            handler: Box::new(|c| Box::pin(T::handle_event(c))),
        })
    }

    pub async fn handle_events(&mut self, ctx: &CommandCtx<'_>) -> Result<(), serenity::Error> {
        let mut interaction_stream = self
            .msg
            .await_component_interaction(&ctx.discord_ctx.shard)
            .timeout(Duration::from_secs(180))
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
            };

            handler(&mut new_ctx).await?;

            if self.has_handler_mutated {
                self.has_handler_mutated = false;
                continue;
            }

            self.handler = handler;
        }

        self.msg.delete(ctx.discord_ctx).await?;
        Ok(())
    }

    pub async fn update_msg<T: InteractiveMessageTrait>(
        &mut self,
        ctx: &Context,
        interaction: &ComponentInteraction,
    ) -> Result<(), serenity::Error> {
        let msg = T::into_msg().ephemeral(true);
        interaction
            .create_response(ctx, CreateInteractionResponse::UpdateMessage(msg))
            .await?;
        self.handler = Box::new(|c| Box::pin(T::handle_event(c)));
        self.has_handler_mutated = true;
        Ok(())
    }
}
