use serenity::{
    all::{CreateEmbed, CreateEmbedFooter},
    async_trait,
};

use crate::{
    commands::given_tasks::command::State,
    components::{CommandCtx, EventCtx},
    database::Task,
    traits::{into_embed::IntoEmbedInteractive, IntoEmbed},
};

pub struct Embed;

impl IntoEmbed for Embed {
    fn into_embed() -> serenity::all::CreateEmbed {
        CreateEmbed::new().color(serenity::model::colour::Color::MEIBE_PINK)
    }
}

impl Embed {
    fn format_task(embed: CreateEmbed, task: &Task) -> CreateEmbed {
        let targets_string = if task.assigned_users.is_empty() {
            "No people assigned".to_owned()
        } else {
            task.assigned_users
                .iter()
                .map(|id| format!("<@{}> ", id))
                .collect::<String>()
        };
        embed.title(task.title.clone()).fields(vec![
            ("Description", task.description.clone(), false),
            (
                "Deadline",
                format!("<t:{}:D>", task.deadline.timestamp()),
                true,
            ),
            (
                "Completed",
                if task.completed {
                    "Yes".to_owned()
                } else {
                    "No".to_owned()
                },
                true,
            ),
            ("Assigned People", targets_string, false),
        ])
    }

    fn create(state: &State) -> CreateEmbed {
        let embed = Self::into_embed();

        let embed = match state.tasks.get(state.page) {
            None => return embed.field("Task", "No Tasks", false),
            Some(t) => Self::format_task(embed, t),
        };

        embed.footer(CreateEmbedFooter::new(format!(
            "{}/{}",
            state.page + 1,
            state.max_page
        )))
    }
}

#[async_trait]
impl IntoEmbedInteractive for Embed {
    async fn from_command(
        _ctx: &CommandCtx,
        state: Option<&crate::components::State>,
    ) -> CreateEmbed {
        let state = state.unwrap().clone::<State>().await.unwrap();
        Self::create(&state)
    }

    async fn from_event(ctx: &EventCtx) -> CreateEmbed {
        let state = ctx.msg.clone_state::<State>().await.unwrap();
        Self::create(&state)
    }
}
