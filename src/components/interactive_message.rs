use serenity::all::{
    ComponentInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Message, MessageFlags,
};
use serenity::futures::StreamExt;
use std::time::Duration;

use crate::aliases::{Result, TypedResult};
use crate::calendar::CalendarHub;
use crate::components::{CommandCtx, EventCtx, State};
use crate::database::Db;
use crate::traits::state::StateTrait;
use crate::traits::InteractiveMessageTrait;

pub struct InteractiveMessage {
    msg: Message,
    state: State,

    //i hate this
    stop: bool,
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
    pub async fn from_event<
        T: InteractiveMessageTrait + 'static,
        S: StateTrait + Send + Sync + 'static,
    >(
        ctx: &mut EventCtx<'_>,
        state: S,
    ) -> TypedResult<Self> {
        let state = State::_new_(state);
        let msg = T::into_msg().embeds(T::with_embeds_event(ctx).await);

        let builder = CreateInteractionResponse::Message(msg);
        ctx.interaction
            .create_response(ctx.discord_ctx, builder)
            .await?;
        let m = ctx.interaction.get_response(ctx.discord_ctx).await?;

        Ok(Self {
            msg: m,
            state: state,
            has_handler_mutated: false,
            handler: Box::new(|c| Box::pin(T::handle_event(c))),
            stop: false,
        })
    }

    pub async fn new<T: InteractiveMessageTrait<State: StateTrait + Send + Sync> + 'static>(
        ctx: &CommandCtx<'_>,
    ) -> TypedResult<Self> {
        let state = State::init::<T::State>(ctx).await?;
        let msg = T::into_msg().embeds(T::with_embeds_command(ctx, &state).await);

        let builder = CreateInteractionResponse::Message(msg);
        ctx.interaction.create_response(ctx, builder).await?;
        let m = ctx.interaction.get_response(ctx.discord_ctx).await?;

        Ok(Self {
            msg: m,
            state: state,
            has_handler_mutated: false,
            handler: Box::new(|c| Box::pin(T::handle_event(c))),
            stop: false,
        })
    }

    async fn _handle_events(&mut self, ctx: &Context, db: &Db, calendars: &CalendarHub) -> Result {
        let mut interaction_stream = self
            .msg
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(600))
            .stream();

        while let Some(int) = interaction_stream.next().await {
            //a band-aid fix for my terrible design
            let handler =
                std::mem::replace(&mut self.handler, Box::new(|_| Box::pin(async { Ok(()) })));

            let mut new_ctx = EventCtx {
                discord_ctx: ctx,
                interaction: &int,
                msg: self,
                db: db,
                calendars: calendars,
            };

            handler(&mut new_ctx).await?;

            if self.stop {
                break;
            }

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

    pub async fn handle_events_from_event(&mut self, ctx: &EventCtx<'_>) -> Result {
        self._handle_events(ctx.discord_ctx, ctx.db, ctx.calendars)
            .await
    }

    pub async fn handle_events(&mut self, ctx: &CommandCtx<'_>) -> Result {
        self._handle_events(ctx.discord_ctx, ctx.db, ctx.calendars)
            .await
    }

    pub fn stop(&mut self) {
        self.stop = true;
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

    pub async fn clone_state<S: StateTrait + Send + Sync + 'static>(&self) -> Option<S> {
        self.state.clone::<S>().await
    }

    pub async fn write_state<S: StateTrait + Send + Sync + 'static>(
        &self,
        new_state: S,
    ) -> Option<()> {
        self.state.write(new_state).await
    }
}
