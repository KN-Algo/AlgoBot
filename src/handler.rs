use serenity::{
    all::{Command, CommandInteraction, Context, CreateCommand, EventHandler, Interaction, Ready},
    async_trait,
};

use crate::log;
use crate::log_error;

use crate::traits::bot_command::BotCommand;
use std::collections::HashMap;

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
        let comm = match self.registered_commands.get(command.data.name.as_str()) {
            Some(c) => c,
            None => {
                log_error!("User {} run a non existing command!", command.user.name);
                return;
            }
        };

        match comm.run(ctx, command.clone()).await {
            Ok(_) => (),
            Err(e) => {
                log_error!("Error running command {}!: {e}", command.data.name);
                return;
            }
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => self.handle_command(&ctx, command).await,
            Interaction::Component(_component) => (),
            Interaction::Modal(_) => (),
            _ => log!("Unsupported interaction: {:?}", interaction),
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        for (name, command) in &self.registered_commands {
            match Command::create_global_command(&ctx, command.register(CreateCommand::new(*name)))
                .await
            {
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
