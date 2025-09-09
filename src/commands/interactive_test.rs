use crate::components::EventCtx;
use crate::traits::BotCommand;
use crate::{
    components::{CommandCtx, InteractiveMessage},
    log,
};
use modal_macro::interactive_msg;
use serenity::{all::CreateCommand, async_trait};

pub struct InterTest;

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
    <AmongMsg handler=AmongMsgHandler>
        <row>
            <button id="among_button" style="primary">"back"</button>
        </row>
    </AmongMsg>
}

struct AmongHandler;

#[async_trait]
impl SusMsgHandlerTrait for SusMsgHandler {
    async fn handle_sus_button(ctx: &mut EventCtx) -> Result<(), serenity::Error> {
        ctx.update_msg::<AmongMsg<AmongHandler>>().await
    }
    async fn handle_sus_button2(ctx: &mut EventCtx) -> Result<(), serenity::Error> {
        log!("pressed sus_button2");
        ctx.acknowlage_interaction().await
    }

    async fn handle_susser(ctx: &mut EventCtx) -> Result<(), serenity::Error> {
        log!("selected susser");
        ctx.acknowlage_interaction().await
    }

    async fn handle_mog(ctx: &mut EventCtx) -> Result<(), serenity::Error> {
        log!("selected mog");
        ctx.acknowlage_interaction().await
    }
}

#[async_trait]
impl AmongMsgHandlerTrait for AmongHandler {
    async fn handle_among_button(ctx: &mut EventCtx) -> Result<(), serenity::Error> {
        ctx.msg
            .update_msg::<SusMsg<SusMsgHandler>>(ctx.discord_ctx, &ctx.interaction)
            .await
    }
}

#[async_trait]
impl BotCommand for InterTest {
    async fn run(&self, ctx: &CommandCtx) -> Result<(), serenity::Error> {
        let mut msg = InteractiveMessage::new::<SusMsg<SusMsgHandler>>(ctx).await?;
        msg.handle_events(&ctx).await?;
        Ok(())
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Command for testing \"UI\"")
    }
}
