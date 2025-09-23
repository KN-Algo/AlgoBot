use serenity::{
    all::{Command, CommandInteraction, Context, CreateCommand, EventHandler, Interaction, Ready},
    async_trait,
};

use crate::{
    calendar::CalendarHub,
    components::{CommandCtx, Db},
    log, log_error, log_warn,
};

use crate::traits::bot_command::BotCommand;
use std::collections::{HashMap, HashSet};

async fn add_remove_edit_commands(
    ctx: &Context,
    registered: &HashMap<&'static str, Box<dyn BotCommand + Sync + Send>>,
    global: &[Command],
) {
    let global_map: HashMap<&str, &Command> = global.iter().map(|c| (c.name.as_str(), c)).collect();

    let global_set: HashSet<&str> = global_map.keys().copied().collect();
    let registered_set: HashSet<&str> = registered.keys().copied().collect();

    for key in registered_set.intersection(&global_set) {
        match Command::edit_global_command(
            ctx,
            global_map[key].id,
            registered[key].register(CreateCommand::new(*key)),
        )
        .await
        {
            Ok(c) => log!("Command {} updated!", c.name),
            Err(e) => log_error!("Error updating command {}!", e),
        }
    }

    for key in registered_set.difference(&global_set) {
        match Command::create_global_command(
            ctx,
            registered[key].register(CreateCommand::new(*key)),
        )
        .await
        {
            Ok(c) => log!("Command {} created!", c.name),
            Err(e) => log_error!("Error creating command {}!", e),
        }
    }

    for key in global_set.difference(&registered_set) {
        match Command::delete_global_command(ctx, global_map[key].id).await {
            Ok(()) => log!("Command {} removed!", key),
            Err(e) => log_error!("Error removing command {}!", e),
        }
    }
}

pub struct Handler {
    registered_commands: HashMap<&'static str, Box<dyn BotCommand + Sync + Send>>,
    db: Db,
    calendar: CalendarHub,
}

impl Handler {
    pub fn new(db: Db, calendar: CalendarHub) -> Self {
        Self {
            registered_commands: HashMap::new(),
            db,
            calendar,
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

        let new_ctx = CommandCtx {
            discord_ctx: ctx,
            interaction: &command,
            db: &self.db,
            calendars: &self.calendar,
        };

        match comm.run(&new_ctx).await {
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
            _ => log_warn!("Unsupported interaction: {:?}", interaction),
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        let all_global_commands = match Command::get_global_commands(&ctx).await {
            Ok(commands) => commands,
            Err(e) => {
                log_error!("Error fetching global commands: {e}! Skipping command registration");
                return;
            }
        };

        add_remove_edit_commands(&ctx, &self.registered_commands, &all_global_commands).await;
    }
}
