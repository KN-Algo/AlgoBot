use handler::Handler;
use serenity::{all::GatewayIntents, Client};

pub mod bot_command;
pub mod commands;
//pub mod err;
pub mod handler;
pub mod log;
pub mod modal;
pub mod response;

use crate::commands::*;

#[tokio::main]
async fn main() {
    log!("Starting the bot!");
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let handler = Handler::new()
        .register_command("ping", Ping)
        .register_command("modal_test", ModalTest);

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
        panic!("Client error: {e}");
    }
}
