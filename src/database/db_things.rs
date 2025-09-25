use chrono::Utc;
use serenity::all::UserId;

#[derive(Clone)]
pub struct Reminder {
    pub when: chrono::Duration,
}

#[derive(Debug, sqlx::Type)]
#[repr(u8)]
#[sqlx(type_name = "INTEGER")]
pub enum ReminderWay {
    DiscordPing = 0,
    DirectMsg = 1,
    Email = 2,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EventReminder {
    pub id: i64,
    pub user_id: UserId,
    pub way: ReminderWay,
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub deadline: chrono::DateTime<Utc>,
    pub given_by: UserId,
    pub reminders: Vec<Reminder>,
}
