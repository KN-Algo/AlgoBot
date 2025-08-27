use crate::{components::InteractiveMessage, log};
use modal_macro::interactive_msg;
use serenity::{
    all::{
        CommandInteraction, ComponentInteraction, Context, CreateCommand, CreateInteractionResponse,
    },
    async_trait,
};
use sqlx::SqlitePool;

use crate::traits::BotCommand;

pub struct InterTest;

interactive_msg! {
    <SusMsg>
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
    <AmongMsg>
        <row>
            <button id="among_button" style="primary">"back"</button>
        </row>
    </AmongMsg>
}

struct AmongHandler;

struct SusMsgHandler;

#[async_trait]
impl SusMsgHandlerTrait for SusMsgHandler {
    async fn handle_sus_button(
        ctx: &Context,
        interaction: &ComponentInteraction,
        msg: &mut InteractiveMessage,
        _db: &sqlx::SqlitePool,
    ) -> Result<(), serenity::Error> {
        msg.update_msg::<AmongMsg<AmongHandler>>(ctx, interaction)
            .await
    }
    async fn handle_sus_button2(
        ctx: &Context,
        interaction: &ComponentInteraction,
        _msg: &mut InteractiveMessage,
        _db: &sqlx::SqlitePool,
    ) -> Result<(), serenity::Error> {
        log!("pressed sus_button2");
        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
    }

    async fn handle_susser(
        ctx: &Context,
        interaction: &ComponentInteraction,
        _msg: &mut InteractiveMessage,
        _db: &sqlx::SqlitePool,
    ) -> Result<(), serenity::Error> {
        log!("selected susser");
        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
    }

    async fn handle_mog(
        ctx: &Context,
        interaction: &ComponentInteraction,
        _msg: &mut InteractiveMessage,
        _db: &sqlx::SqlitePool,
    ) -> Result<(), serenity::Error> {
        log!("selected mog");
        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
    }
}

#[async_trait]
impl AmongMsgHandlerTrait for AmongHandler {
    async fn handle_among_button(
        ctx: &Context,
        interaction: &ComponentInteraction,
        msg: &mut InteractiveMessage,
        _db: &sqlx::SqlitePool,
    ) -> Result<(), serenity::Error> {
        msg.update_msg::<SusMsg<SusMsgHandler>>(ctx, interaction)
            .await
    }
}

#[async_trait]
impl BotCommand for InterTest {
    async fn run(
        &self,
        ctx: &Context,
        interaction: CommandInteraction,
        db: &SqlitePool,
    ) -> Result<(), serenity::Error> {
        let mut msg = InteractiveMessage::new::<SusMsg<SusMsgHandler>>(ctx, interaction).await?;
        msg.handle_events(ctx, db).await?;
        Ok(())
    }

    fn register(&self, create: CreateCommand) -> CreateCommand {
        create.description("Command for testing \"UI\"")
    }
}
