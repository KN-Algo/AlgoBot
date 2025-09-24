use serenity::{all::GatewayIntents, Client};

use crate::{
    commands::{
        add_task::AddTaskCommand, events::command::EventsCommand, EmbedTest, InterTest, ModalTest,
        Ping,
    },
    database::Db,
    handler::Handler,
};

pub mod aliases;
pub mod calendar;
pub mod commands;
pub mod components;
pub mod database;
pub mod error;
pub mod handler;
pub mod log;
pub mod traits;

#[tokio::main]
async fn main() {
    log!("Starting the bot!");
    let token = match std::env::var("DISCORD_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            log_error!("DISCORD_TOKEN is not set!");
            return;
        }
    };

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let url = match std::fs::read_to_string("calendar.secret") {
        Ok(u) => u,
        Err(e) => {
            log_error!("Failed to read calendar.secret! {e}");
            return;
        }
    };

    let hub = calendar::CalendarHub::new(url).await;
    let db = match Db::new("bot_db.sqlite", 5).await {
        Ok(db) => db,
        Err(e) => {
            log_error!("Database Error! {e}");
            return;
        }
    };

    let handler = Handler::new(db, hub)
        .register_command("ping", Ping)
        .register_command("modal_test", ModalTest)
        .register_command("inter_test", InterTest)
        .register_command("embed_test", EmbedTest)
        .register_command("events", EventsCommand)
        .register_command("add_task", AddTaskCommand);

    let mut client = match Client::builder(token, intents).event_handler(handler).await {
        Ok(c) => {
            log!("Client created!");
            c
        }
        Err(e) => {
            log_error!("There was an error starting the client!\n{e}");
            return;
        }
    };

    if let Err(e) = client.start().await {
        log_error!("Client error: {e}")
    }
}
