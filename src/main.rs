use serenity::{all::GatewayIntents, Client};

use crate::{
    commands::{events::command::EventsCommand, EmbedTest, InterTest, ModalTest, Ping},
    handler::Handler,
};

pub mod aliases;
pub mod calendar;
pub mod commands;
pub mod components;
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

    let db = match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("bot_db.sqlite")
                .create_if_missing(true),
        )
        .await
    {
        Ok(db) => {
            log!("Connected to the database!");
            db
        }
        Err(e) => {
            log_error!("Couldn't connect to the database!: {e}");
            return;
        }
    };

    match sqlx::migrate!("./migrations").run(&db).await {
        Ok(()) => log!("Successfully applied migrations!"),
        Err(e) => {
            log_error!("Migrations failed! {e}");
            return;
        }
    }

    let url = match std::fs::read_to_string("calendar.secret") {
        Ok(u) => u,
        Err(e) => {
            log_error!("Failed to read calendar.secret! {e}");
            return;
        }
    };

    let hub = calendar::CalendarHub::new(url).await;

    let handler = Handler::new(db, hub)
        .register_command("ping", Ping)
        .register_command("modal_test", ModalTest)
        .register_command("inter_test", InterTest)
        .register_command("embed_test", EmbedTest)
        .register_command("events", EventsCommand);

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
