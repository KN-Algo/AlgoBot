use serenity::all::{
    CommandInteraction, ComponentInteraction, Context, CreateInteractionResponse, Message,
};
use serenity::futures::StreamExt;
use sqlx::SqlitePool;
use std::time::Duration;

use crate::traits::InteractiveMessageTrait;

pub struct InteractiveMessage {
    msg: Message,

    //i hate this
    has_handler_mutated: bool,
    handler: Box<
        dyn for<'a> Fn(
                &'a Context,
                &'a ComponentInteraction,
                &'a mut InteractiveMessage,
                &'a sqlx::SqlitePool,
            ) -> std::pin::Pin<
                Box<dyn Future<Output = Result<(), serenity::Error>> + Send + 'a>,
            > + Send
            + Sync,
    >,
}

impl InteractiveMessage {
    pub async fn new<T: InteractiveMessageTrait + 'static>(
        ctx: &Context,
        interaction: CommandInteraction,
    ) -> Result<Self, serenity::Error> {
        let msg = T::into_msg().ephemeral(true);

        let builder = CreateInteractionResponse::Message(msg);
        interaction.create_response(ctx, builder).await?;
        let m = interaction.get_response(&ctx.http).await?;

        Ok(Self {
            msg: m,
            has_handler_mutated: false,
            handler: Box::new(|c, i, m, db| Box::pin(T::handle_event(c, i, m, db))),
        })
    }
    pub async fn handle_events(
        &mut self,
        ctx: &Context,
        db: &SqlitePool,
    ) -> Result<(), serenity::Error> {
        let mut interaction_stream = self
            .msg
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(180))
            .stream();

        while let Some(int) = interaction_stream.next().await {
            //a band-aid fix for my terrible design
            let handler = std::mem::replace(
                &mut self.handler,
                Box::new(|_, _, _, _| Box::pin(async { Ok(()) })),
            );

            handler(ctx, &int, self, db).await?;

            if self.has_handler_mutated {
                self.has_handler_mutated = false;
                continue;
            }

            self.handler = handler;
        }

        self.msg.delete(ctx).await?;
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
        self.handler = Box::new(|c, i, m, db| Box::pin(T::handle_event(c, i, m, db)));
        self.has_handler_mutated = true;
        Ok(())
    }
}
