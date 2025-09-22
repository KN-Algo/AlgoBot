use crate::aliases::Result;
use crate::components::EventCtx;
use crate::components::{CommandCtx, InteractiveMessage};
use crate::traits::into_embed::IntoEmbedInteractive;
use crate::traits::Interactable;
use crate::traits::{BotCommand, IntoEmbed};
use modal_macro::interactive_msg;
use modal_macro::modal;
use serenity::all::{CreateEmbed, CreateEmbedFooter, Timestamp};
use serenity::{all::CreateCommand, async_trait};

pub struct InterTest;

struct EmbedTest;

impl IntoEmbed for EmbedTest {
    fn into_embed() -> serenity::all::CreateEmbed {
        CreateEmbed::new()
            .title("hiiii :3333")
            .description("henlo :333")
            .color(serenity::model::Colour::MEIBE_PINK)
            .footer(CreateEmbedFooter::new("byeee <3"))
            .timestamp(Timestamp::now())
    }
}

#[async_trait]
impl IntoEmbedInteractive for EmbedTest {
    async fn from_event(ctx: &EventCtx) -> CreateEmbed {
        let cal = ctx.calendars.get_calendar("").await;
        Self::into_embed()
    }

    async fn from_command(ctx: &CommandCtx) -> CreateEmbed {
        Self::into_embed()
    }
}

interactive_msg! {
    <SusMsg handler=SusMsgHandler>
        <row>
            <button id="sus_button" style="primary">"pr"</button>
            <button id="sus_button2" style="secondary">"sc"</button>
            <button id="sus_button3" style="success">"success"</button>
            <button id="sus_button4" style="danger">"danger"</button>
            <button id="sus_button5" link="https://youtube.com">"link"</button>
        </row>
        <row>
            <selection id="select">
                <option id="susser">"the susser"</option>
                <option id="mog" description="sus" default=true>"the moger"</option>
            </selection>
        </row>
    </SusMsg>
}

interactive_msg! {
    <AmongMsg handler=AmongHandler>
        <embed>EmbedTest</embed>
        <row>
            <button id="among_button" style="primary">"back"</button>
        </row>
    </AmongMsg>
}

modal! {
<ImposterModal title="Impostor!" duration=30>
    <row>
        <input id="i" style="short">"idk what to put here"</input>
    </row>
</ImposterModal>
}

#[async_trait]
impl SusMsgHandlerTrait for SusMsgHandler {
    async fn handle_sus_button(ctx: &mut EventCtx) -> Result {
        ctx.update_msg::<AmongMsg<AmongHandler>>().await
    }

    async fn handle_sus_button2(ctx: &mut EventCtx) -> Result {
        ctx.respond("I am the sus").await
    }

    async fn handle_sus_button3(ctx: &mut EventCtx) -> Result {
        ctx.acknowlage().await
    }

    async fn handle_susser(ctx: &mut EventCtx) -> Result {
        ctx.acknowlage().await
    }

    async fn handle_mog(ctx: &mut EventCtx) -> Result {
        ctx.acknowlage().await
    }
}

#[async_trait]
impl AmongHandlerTrait for AmongHandler {
    async fn handle_among_button(ctx: &mut EventCtx) -> Result {
        ctx.update_msg::<SusMsg<SusMsgHandler>>().await
    }
}

#[async_trait]
impl BotCommand for InterTest {
    async fn run(&self, ctx: &CommandCtx) -> Result {
        let mut msg = InteractiveMessage::new::<SusMsg<SusMsgHandler>>(ctx).await?;
        msg.handle_events(&ctx).await
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Command for testing \"UI\"")
    }
}
