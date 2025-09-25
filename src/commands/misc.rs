use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use crate::aliases::TypedResult;

pub fn parse_date_dd_mm_yy(date: &str) -> TypedResult<DateTime<Utc>> {
    let naive = NaiveDate::parse_from_str(date, "%d-%m-%y")?;
    let naive_date = naive.and_hms_opt(0, 0, 0).unwrap();
    Ok(Utc.from_utc_datetime(&naive_date))
}

#[macro_export]
macro_rules! add_users_to_task_from_msg {
    ($interactable:ident, $ctx: ident, $user_id:expr, $task:ident) => {{
        let (mut bot_response, user_response) = match $interactable
            .respond_and_get_response(
                "Respond to this message with @Mentions to add users to the task",
                "Timed Out!",
                $user_id,
            )
            .await?
        {
            None => return Ok(()),
            Some(msgs) => msgs,
        };

        if user_response.mentions.len() == 0 {
            bot_response
                .edit(
                    $ctx,
                    ::serenity::all::EditMessage::new().content("No users mentioned!"),
                )
                .await?;
            return Ok(());
        }

        let mentions = user_response
            .mentions
            .into_iter()
            .map(|user| user.id)
            .filter(|id| id.get() != 1319203114917822514)
            .collect();
        $task.assigned_users.extend(&mentions);
        $ctx.db.add_users_to_task($task.id, mentions).await?;
        bot_response
            .edit(
                $ctx.discord_ctx,
                ::serenity::all::EditMessage::new().content("Users added to the task!"),
            )
            .await?;
    }};
}
