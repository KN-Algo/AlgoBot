use chrono::{DateTime, TimeZone, Utc};
use serenity::all::UserId;
use sqlx::Row;
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

use crate::calendar::Event;
use crate::database::{ReminderGroup, Summary};
use crate::{
    aliases::{Result, TypedResult},
    database::{EventReminder, Reminder, ReminderWay, Task},
    log, log_error,
};

pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(filename: &str, max_connections: u32) -> TypedResult<Self> {
        let db = match sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(filename)
                    .create_if_missing(true),
            )
            .await
        {
            Ok(db) => {
                log!("Connected to the database!");
                db
            }
            Err(e) => {
                log_error!("Couldn't connect to the database!: {e}");
                return Err(e.into());
            }
        };

        match sqlx::migrate!("./migrations").run(&db).await {
            Ok(()) => log!("Successfully applied migrations!"),
            Err(e) => {
                log_error!("Migrations failed! {e}");
                return Err(e.into());
            }
        }

        match sqlx::query("PRAGMA foreign_keys = ON").execute(&db).await {
            Ok(_) => (),
            Err(e) => {
                log!("Failed to turn on foreign_keys support!");
                return Err(e.into());
            }
        }

        Ok(Self { pool: db })
    }

    async fn insert_user(&self, discord_id: UserId) -> Result {
        let id: i64 = discord_id.into();
        sqlx::query!("INSERT OR IGNORE INTO users (discord_id) VALUES (?)", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_user_tasks(&self, discord_id: UserId) -> TypedResult<Vec<Task>> {
        let id: i64 = discord_id.into();
        self.insert_user(discord_id).await?;
        let rows = sqlx::query!(
            r#"
        SELECT 
            t.id AS task_id,
            t.title,
            t.description,
            t.completed,
            t.deadline_unixtimestamp,
            t.given_by,
            r.id AS reminder_id,
            r.task AS reminder_task,
            r.when_unixtimestamp as "when: Option<i64>"
        FROM tasks t
        JOIN task_targets tt ON t.id = tt.task_id
        LEFT JOIN reminders r ON r.task = t.id AND r.user_id = tt.user_id
        WHERE tt.user_id = ?
        "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut group_map: HashMap<i64, Task> = HashMap::new();

        for row in rows {
            let task = group_map.entry(row.task_id).or_insert_with(|| Task {
                id: row.task_id,
                title: row.title,
                completed: row.completed,
                description: row.description,
                given_by: UserId::new(row.given_by.try_into().unwrap()),
                deadline: Utc.timestamp_opt(row.deadline_unixtimestamp, 0).unwrap(),
                reminders: Vec::new(),
                assigned_users: Vec::new(),
            });
            if let Some(when) = row.when {
                task.reminders.push(Reminder {
                    when: chrono::Duration::new(when, 0).unwrap(),
                });
            }
        }

        Ok(group_map.into_values().collect())
    }

    pub async fn add_reminder(
        &self,
        task_id: i64,
        user_id: UserId,
        when: Vec<chrono::Duration>,
    ) -> Result {
        let id: i64 = user_id.into();
        let mut trans = self.pool.begin().await?;
        for w in when {
            let secs = w.num_seconds();
            sqlx::query!(
                r#"INSERT INTO reminders (task, when_unixtimestamp, user_id) VALUES (?, ?, ?)"#,
                task_id,
                secs,
                id
            )
            .execute(&mut *trans)
            .await?;
        }

        trans.commit().await?;
        Ok(())
    }

    pub async fn get_given_tasks(&self, discord_id: UserId) -> TypedResult<Vec<Task>> {
        self.insert_user(discord_id).await?;
        let id: i64 = discord_id.into();

        // 1. Fetch tasks given by this user
        let task_rows = sqlx::query!(
            r#"
        SELECT * FROM tasks WHERE given_by = ?
        "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        let task_ids: Vec<i64> = task_rows.iter().map(|row| row.id).collect();

        // 2. If no tasks, return early
        if task_ids.is_empty() {
            return Ok(vec![]);
        }

        // 3. Dynamically build the IN clause and bind parameters
        let placeholders = task_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "SELECT task_id, user_id FROM task_targets WHERE task_id IN ({})",
            placeholders
        );

        // Build query with args
        let mut query = sqlx::query(&sql);
        for id in &task_ids {
            query = query.bind(id);
        }

        let target_rows = query.fetch_all(&self.pool).await?;

        // 4. Map task_id => Vec<UserId>
        use std::collections::HashMap;
        let mut assignments: HashMap<i64, Vec<UserId>> = HashMap::new();
        for row in target_rows {
            // Since we used sqlx::query(), the fields need to be accessed by name
            let task_id: i64 = row.try_get("task_id")?;
            let user_id: i64 = row.try_get("user_id")?;
            assignments
                .entry(task_id)
                .or_default()
                .push(UserId::new(user_id as u64));
        }

        // 5. Assemble Task list
        let tasks = task_rows
            .into_iter()
            .map(|row| Task {
                id: row.id,
                title: row.title,
                description: row.description,
                completed: row.completed,
                deadline: Utc.timestamp_opt(row.deadline_unixtimestamp, 0).unwrap(),
                given_by: UserId::new(row.given_by as u64),
                reminders: vec![],
                assigned_users: assignments.remove(&row.id).unwrap_or_default(),
            })
            .collect();

        Ok(tasks)
    }

    pub async fn get_assigned_users(&self, task_id: i64) -> TypedResult<Vec<UserId>> {
        Ok(sqlx::query!(
            r#"SELECT user_id FROM task_targets where task_id = ?"#,
            task_id
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| UserId::new(row.user_id.try_into().unwrap()))
                .collect()
        })?)
    }

    pub async fn add_users_to_task(&self, task_id: i64, users: Vec<UserId>) -> Result {
        let mut transaction = self.pool.begin().await?;
        for target in users {
            let id: i64 = target.into();
            sqlx::query!("INSERT OR IGNORE INTO users (discord_id) VALUES (?)", id)
                .execute(&mut *transaction)
                .await?;
            sqlx::query!(
                r#"
                INSERT OR IGNORE INTO task_targets (task_id, user_id)
                VALUES (?, ?)
                "#,
                task_id,
                id
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn add_task(
        &self,
        title: &str,
        description: &str,
        deadline: DateTime<Utc>,
        given_by: UserId,
    ) -> TypedResult<Task> {
        let timestamp = deadline.timestamp();
        let id: i64 = given_by.into();
        let mut trans = self.pool.begin().await?;
        let last_id = sqlx::query!(
            r#"INSERT INTO tasks (title, description, deadline_unixtimestamp, given_by) VALUES (?, ?, ?, ?)"#,
            title,
            description,
            timestamp,
            id
        )
        .execute(&mut *trans)
        .await?.last_insert_rowid();

        let row = sqlx::query!(
            r#"
        SELECT * from tasks WHERE id = ?
        "#,
            last_id
        )
        .fetch_one(&mut *trans)
        .await?;

        trans.commit().await?;

        Ok(Task {
            id: row.id,
            title: row.title,
            description: row.description,
            completed: row.completed,
            deadline: Utc.timestamp_opt(row.deadline_unixtimestamp, 0).unwrap(),
            given_by: UserId::new(row.given_by.try_into().unwrap()),
            reminders: vec![],
            assigned_users: vec![],
        })
    }

    pub async fn edit_task(&self, new_task: &Task) -> Result {
        let timestamp = new_task.deadline.timestamp();
        sqlx::query!(r#"UPDATE tasks SET title = ?, description = ?, deadline_unixtimestamp = ? WHERE id = ?"#, new_task.title, new_task.description, timestamp, new_task.id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn toggle_task_completion(&self, task_id: i64) -> Result {
        sqlx::query!(
            r#"
            UPDATE tasks
            SET completed = NOT completed
            WHERE id = ?
            "#,
            task_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_task(&self, task_id: i64) -> Result {
        sqlx::query!(r#"DELETE FROM tasks WHERE id = ?"#, task_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_completed_expired_tasks(&self) -> Result {
        let now = Utc::now().timestamp();
        sqlx::query!(
            r#"
            DELETE FROM tasks
            WHERE completed = 1
              AND deadline_unixtimestamp < ?
            "#,
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_expired_custom_events(&self) -> Result {
        let now = Utc::now().timestamp();
        sqlx::query!(
            r#"
            DELETE FROM custom_events
            WHERE start < ?
            "#,
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_event_reminder(
        &self,
        discord_id: UserId,
        way: ReminderWay,
        group: ReminderGroup,
        email: Option<String>,
    ) -> Result {
        let id: i64 = discord_id.into();
        self.insert_user(discord_id).await?;
        sqlx::query!(
            r#"
                INSERT INTO event_reminders (user_id, way, email, reminder_group)
                VALUES (?, ?, ?, ?)
                "#,
            id,
            way,
            email,
            group
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_event_reminder(
        &self,
        discord_id: UserId,
        group: ReminderGroup,
        way: ReminderWay,
    ) -> Result {
        let id: i64 = discord_id.into();
        sqlx::query!(
            r#"
                DELETE FROM event_reminders WHERE user_id = ? AND reminder_group = ? AND way = ?
            "#,
            id,
            group,
            way
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_event_reminders(
        &self,
        user_id: UserId,
    ) -> TypedResult<Vec<EventReminder>> {
        let id: i64 = user_id.into();
        Ok(sqlx::query!(r#"SELECT id, user_id, way as "way: ReminderWay", reminder_group as "rgroup: ReminderGroup", email FROM event_reminders WHERE user_id = ?"#, id)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|record| EventReminder {
                    id: record.id,
                    user_id: UserId::new(record.user_id.try_into().unwrap()),
                    way: record.way,
                    email: record.email,
                    group: record.rgroup
                }).collect())
    }

    pub async fn fetch_event_reminders(&self) -> TypedResult<Vec<EventReminder>> {
        Ok(sqlx::query!(
            r#"SELECT id, user_id, way as "way: ReminderWay", email, reminder_group as "rgroup: ReminderGroup" FROM event_reminders"#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|record| EventReminder {
            id: record.id,
            user_id: UserId::new(record.user_id.try_into().unwrap()),
            way: record.way,
            email: record.email,
            group: record.rgroup
        })
        .collect())
    }

    pub async fn add_custom_event(&self, summary: &str, start: DateTime<Utc>) -> Result {
        let stamp = start.timestamp();
        sqlx::query!(
            r#"INSERT INTO custom_events (summary, start) VALUES (?, ?)"#,
            summary,
            stamp
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_custom_events(&self) -> TypedResult<Vec<Event>> {
        Ok(sqlx::query!(r#"SELECT * FROM custom_events"#)
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|row| Event {
                        uid: 0.to_string(),
                        summary: row.summary,
                        start: Utc.timestamp_opt(row.start, 0).unwrap(),
                    })
                    .collect()
            })?)
    }

    pub async fn get_summaries(&self) -> TypedResult<Vec<Summary>> {
        Ok(sqlx::query_as!(Summary, r#"SELECT * FROM summaries"#)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn add_summary(&self, content: &str, author: &str) -> Result {
        sqlx::query!(
            r#"INSERT INTO summaries (content, author) VALUES (?, ?)"#,
            content,
            author
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_summary(&self, summary: Summary) -> Result {
        sqlx::query!(r#"DELETE FROM summaries WHERE id = ?"#, summary.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn clear_summaries(&self) -> Result {
        sqlx::query!(r#"DELETE FROM summaries"#)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
