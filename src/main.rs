use handler::Handler;
use serenity::{all::GatewayIntents, Client};

pub mod bot_command;
pub mod commands;
pub mod err;
pub mod handler;
pub mod log;

use crate::commands::*;

#[tokio::main]
async fn main() {
    log!("Starting the bot!");
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut handler = Handler::new();

    handler
        .register_command("ping", Ping)
        .register_command("modal_test", ModalTest);

    let mut client = match Client::builder(token, intents).event_handler(handler).await {
        Ok(c) => {
            log!("Client created!");
            c
        }
        Err(e) => {
            panic!("There was an error starting the client!\n{e}");
        }
    };

    if let Err(e) = client.start().await {
        panic!("Client error: {e}");
    }
}
