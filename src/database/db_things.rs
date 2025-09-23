use chrono::Utc;

pub struct Reminder {
    pub when: chrono::Duration,
}

pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub deadline: chrono::DateTime<Utc>,
    pub reminders: Vec<Reminder>,
}
