use std::collections::HashMap;

use serenity::builder::Builder;
use serenity::{
    all::{
        Command, CommandInteraction, Context, CreateInteractionResponse,
        CreateInteractionResponseMessage, EventHandler, Ready,
    },
    async_trait,
};

use crate::log_error;
use crate::response::Interaction;
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
        mut self,
        name: &'static str,
        command: C,
    ) -> Self {
        self.registered_commands.insert(name, Box::new(command));
        self
    }

    async fn handle_command(&self, ctx: &Context, command: CommandInteraction) {
        let inter = Interaction::from_command(ctx, &command);
        let response = match self.registered_commands.get(inter.name.as_str()) {
            Some(c) => match c.run(inter).await {
                Ok(s) => s,
                Err(e) => {
                    log_error!("Error running command {}! : {e}", command.data.name);
                    return;
                }
            },
            None => {
                log_error!("User {} run a non existing command!", command.user.name);
                return;
            }
        };

        let msg = CreateInteractionResponseMessage::new().content(response.msg);
        match CreateInteractionResponse::Message(msg)
            .execute(ctx, (response.id, &response.token))
            .await
        {
            Ok(()) => (),
            Err(e) => log_error!("Error sending msg: {e}"),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: serenity::all::Interaction) {
        match interaction {
            serenity::all::Interaction::Command(command) => {
                self.handle_command(&ctx, command).await
            }
            serenity::all::Interaction::Modal(_) => (),
            //Interaction::Autocomplete(_) => (),
            _ => (), //Self::error(&ctx, &interaction, "Interaction not supported").await,
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        for command in self.registered_commands.values() {
            match Command::create_global_command(&ctx, command.register()).await {
                Ok(c) => log!("Command \"{}\" registered!", c.name),
                Err(e) => log_error!("Error registering command {e}"),
            }
        }
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}
