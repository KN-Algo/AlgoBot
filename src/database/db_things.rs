use std::fmt::Display;

use chrono::Utc;
use modal_macro::Selection;
use serenity::all::{ChannelId, Http, UserId};

use crate::{aliases::Result, traits::IntoMessage};

#[derive(Debug, Clone)]
pub struct Reminder {
    pub when: chrono::Duration,
}

#[derive(Selection, Debug, sqlx::Type, Clone, Copy)]
#[repr(u8)]
#[sqlx(type_name = "INTEGER")]
pub enum ReminderWay {
    #[select_value("Discord Ping")]
    DiscordPing = 0,
    #[select_value("Direct Message")]
    DirectMsg = 1,
    Email = 2,
}

impl Display for ReminderWay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DiscordPing => "Discord Ping",
            Self::DirectMsg => "Direct Message",
            Self::Email => "Email",
        };

        write!(f, "{s}")
    }
}

#[derive(Debug, sqlx::Type, Clone, Copy)]
#[repr(u8)]
#[sqlx(type_name = "INTEGER")]
pub enum ReminderGroup {
    Events = 0,
    Tasks = 1,
    Summaries = 2,
}

impl Display for ReminderGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Events => "Events",
            Self::Tasks => "Tasks",
            Self::Summaries => "Summaries",
        };

        write!(f, "{s}")
    }
}

impl ReminderGroup {
    pub fn from_u8(u: u8) -> Option<Self> {
        match u {
            0 => Some(Self::Events),
            1 => Some(Self::Tasks),
            2 => Some(Self::Summaries),
            _ => None,
        }
    }
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct EventReminder {
    pub user_id: UserId,
    pub way: ReminderWay,
    pub email: Option<String>,
    pub group: ReminderGroup,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub deadline: chrono::DateTime<Utc>,
    pub given_by: UserId,
    pub reminders: Vec<Reminder>,
    pub assigned_users: Vec<UserId>,
}

#[derive(Clone, Debug, sqlx::Type)]
pub struct Summary {
    pub id: i64,
    pub author: String,
    pub content: String,
}
