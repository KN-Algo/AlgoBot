use std::collections::HashMap;
//use std::sync::{Arc, Mutex};

use serenity::{
    all::{
        Command, CommandInteraction, Context, CreateInteractionResponse,
        CreateInteractionResponseMessage, EventHandler, Interaction, Ready,
    },
    async_trait,
};

use crate::log_error;
use crate::{bot_command::BotCommand, log};

pub struct Handler {
    registered_commands: HashMap<&'static str, Box<dyn BotCommand + Sync + Send>>,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            registered_commands: HashMap::new(),
        }
    }

    pub fn register_command<C: BotCommand + Sync + Send + 'static>(
        &mut self,
        name: &'static str,
        command: C,
    ) -> &mut Self {
        self.registered_commands.insert(name, Box::new(command));
        self
    }

    async fn handle_command(&self, ctx: &Context, command: &CommandInteraction) {
        let content = match self.registered_commands.get(command.data.name.as_str()) {
            Some(c) => match c.run(ctx, &command).await {
                Ok(s) => s,
                Err(e) => {
                    log_error!("Error running command {}! : {e}", command.data.name);
                    "Error running command".into()
                }
            },
            None => "Command not found".into(),
        };
        let msg = CreateInteractionResponseMessage::new().content(content);
        match command
            .create_response(ctx, CreateInteractionResponse::Message(msg))
            .await
        {
            Ok(()) => (),
            Err(e) => log_error!("Error sending msg: {e}"),
        }
    }

    async fn error(ctx: &Context, interaction: &Interaction, err: &str) {
        let msg = CreateInteractionResponseMessage::new().content(err);
        match interaction {
            Interaction::Command(c) => {
                match c
                    .create_response(ctx, CreateInteractionResponse::Message(msg))
                    .await
                {
                    Ok(()) => (),
                    Err(e) => log_error!("Error sending msg: {e}"),
                }
            }
            Interaction::Ping(_) => (),
            Interaction::Autocomplete(_) => (),
            Interaction::Modal(_) => (),
            Interaction::Component(_) => (),
            &_ => (),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => self.handle_command(&ctx, &command).await,
            //Interaction::Autocomplete(_) => (),
            _ => Self::error(&ctx, &interaction, "Interaction not supported").await,
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        //use crate::comm::*;
        //macro_rules! register_commands (
        //    {$($t:ident),+} => {
        //            $(
        //                match Command::create_global_command(&ctx, $t::register()).await {
        //                    Ok(c) => log!("Command {} registered!", c.name),
        //                    Err(e) => log_error!("Error registering command {e}")
        //                }
        //            )+
        //    };
        //);

        //register_commands!(ping, modal_test);

        for (_, command) in &self.registered_commands {
            match Command::create_global_command(&ctx, command.register()).await {
                Ok(c) => log!("Command \"{}\" registered!", c.name),
                Err(e) => log_error!("Error registering command {e}"),
            }
        }
    }
}
