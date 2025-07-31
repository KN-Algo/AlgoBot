use serenity::{all::GatewayIntents, Client};

use crate::{
    commands::{InterTest, ModalTest, Ping},
    handler::Handler,
};

pub mod commands;
pub mod components;
pub mod handler;
pub mod log;
pub mod traits;

#[tokio::main]
async fn main() {
    log!("Starting the bot!");
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set!");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let handler = Handler::new()
        .register_command("ping", Ping)
        .register_command("modal_test", ModalTest)
        .register_command("inter_test", InterTest);

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
