use crate::{
    aliases::{Result, TypedResult},
    commands::summaries::embed::Embed,
    components::{CommandCtx, EventCtx, InteractiveMessage},
    database::Summary,
    traits::{BotCommand, Interactable, StateTrait},
};
use modal_macro::interactive_msg;
use serenity::{all::CreateCommand, async_trait};

#[derive(Clone)]
pub struct State {
    pub summaries: Vec<Summary>,
    pub page: usize,
    pub max_page: usize,
}

#[async_trait]
impl StateTrait for State {
    async fn init(ctx: &CommandCtx) -> TypedResult<Self> {
        let summaries = ctx.db.get_summaries().await?;
        Ok(Self {
            max_page: summaries.len(),
            summaries,
            page: 0,
        })
    }
}

interactive_msg! {
    <SummariesMsg handler=Handler state=State ephemeral=true>
        <embed>Embed</embed>
        <row>
            <button id="prev">"<"</button>
            <button id="add" style="secondary">"+"</button>
            <button id="delete" style="danger">"🗑️"</button>
            <button id="send" style="success">"✉️"</button>
            <button id="next">">"</button>
        </row>
    </SummariesMsg>
}

interactive_msg! {
    <ConfirmMsg handler=ConfirmHandler ephemeral=true>
        <text>"Are you sure?"</text>
        <row>
            <button id="confirm">"Confirm"</button>
        </row>
    </ConfirmMsg>
}

interactive_msg! {
    <EmptyMsg handler=EmptyHandler>
        <text>"Done!"</text>
    </EmptyMsg>
}

modal_macro::modal! {
    <AddSummaryModal title="New Summary" duration=600>
        <row>
            <input id="content" style="paragraph">"Description"</input>
        </row>
    </AddSummaryModal>
}

impl EmptyHandlerTrait for EmptyHandler {}

#[async_trait]
impl ConfirmHandlerTrait for ConfirmHandler {
    async fn handle_confirm(ctx: &mut EventCtx) -> Result {
        ctx.db.clear_summaries().await?;
        ctx.msg.stop();
        ctx.update_msg::<EmptyMsg<EmptyHandler>>().await
    }
}

#[async_trait]
impl HandlerTrait for Handler {
    async fn handle_prev(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        if state.max_page == 0 {
            return ctx.acknowlage().await;
        }

        if state.page == 0 {
            state.page = state.max_page - 1;
        } else {
            state.page -= 1;
        }

        ctx.msg.write_state(state).await;
        ctx.update_msg::<SummariesMsg<Self>>().await
    }

    async fn handle_next(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        if state.max_page == 0 {
            return ctx.acknowlage().await;
        }

        if state.page == state.max_page - 1 {
            state.page = 0;
        } else {
            state.page += 1;
        }

        ctx.msg.write_state(state).await;
        ctx.update_msg::<SummariesMsg<Self>>().await
    }

    async fn handle_add(ctx: &mut EventCtx) -> Result {
        let result = ctx.modal::<AddSummaryModal>().await?;
        ctx.db
            .add_summary(
                &result.content,
                &result
                    .interaction
                    .user
                    .nick_in(
                        ctx.discord_ctx,
                        ctx.interaction.guild_id.unwrap_or_default(),
                    )
                    .await
                    .unwrap_or(ctx.interaction.user.name.clone()),
            )
            .await?;

        result.respond("Done!", true).await
    }

    async fn handle_delete(ctx: &mut EventCtx) -> Result {
        let mut state = ctx.msg.clone_state::<State>().await.unwrap();
        if state.max_page == 0 {
            return ctx.acknowlage().await;
        }
        let summary = state.summaries.remove(state.page);
        ctx.db.delete_summary(summary).await?;
        state.max_page -= 1;
        ctx.msg.write_state::<State>(state).await;
        ctx.respond("Done!", true).await
    }

    async fn handle_send(ctx: &mut EventCtx) -> Result {
        let mut msg =
            InteractiveMessage::from_event::<ConfirmMsg<ConfirmHandler>, ()>(ctx, ()).await?;
        msg.handle_events_from_event(ctx).await
    }
}

pub struct SummariesCommand;

#[async_trait]
impl BotCommand for SummariesCommand {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = InteractiveMessage::new::<SummariesMsg<Handler>>(ctx).await?;
        msg.handle_events(ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Add, remove and send email summaries")
    }
}
