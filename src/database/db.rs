use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use serenity::all::UserId;
use sqlx::{Pool, Sqlite};

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
            r.when_unixtimestamp as "when"
        FROM tasks t
        JOIN task_targets tt ON t.id = tt.task_id
        LEFT JOIN reminders r ON r.task = t.id
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
                given_by: row.given_by,
                deadline: Utc.timestamp_opt(row.deadline_unixtimestamp, 0).unwrap(),
                reminders: Vec::new(),
            });

            task.reminders.push(Reminder {
                when: chrono::Duration::new(row.when, 0).unwrap(),
            });
        }

        Ok(group_map.into_values().collect())
    }

    pub async fn add_reminder(&self, task_id: i64, when: chrono::Duration) -> Result {
        let secs = when.num_seconds();
        sqlx::query!(
            r#"INSERT INTO reminders (task, when_unixtimestamp) VALUES (?, ?)"#,
            task_id,
            secs
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_given_tasks(&self, discord_id: UserId) -> TypedResult<Vec<Task>> {
        self.insert_user(discord_id).await?;

        let id: i64 = discord_id.into();
        Ok(sqlx::query!(
            r#"
        SELECT * from tasks WHERE given_by = ?
        "#,
            id
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| Task {
                    id: row.id,
                    title: row.title,
                    description: row.description,
                    completed: row.completed,
                    deadline: Utc.timestamp_opt(row.deadline_unixtimestamp, 0).unwrap(),
                    given_by: row.given_by,
                    reminders: vec![],
                })
                .collect()
        })?)
    }

    pub async fn add_users_to_task(&self, task_id: i64, users: Vec<UserId>) -> Result {
        let mut transaction = self.pool.begin().await?;
        for target in users {
            let id: i64 = target.into();
            self.insert_user(target).await?;
            sqlx::query!(
                r#"
                INSERT INTO task_targets (task_id, user_id)
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
    ) -> Result {
        let timestamp = deadline.timestamp();
        sqlx::query!(
            r#"INSERT INTO tasks (title, description, deadline_unixtimestamp) VALUES (?, ?, ?)"#,
            title,
            description,
            timestamp
        )
        .execute(&self.pool)
        .await?;
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

    pub async fn add_event_reminder(
        &self,
        discord_id: UserId,
        way: ReminderWay,
        email: Option<String>,
    ) -> Result {
        let id: i64 = discord_id.into();
        self.insert_user(discord_id).await?;
        sqlx::query!(
            r#"
                INSERT INTO event_reminders (user_id, way, email)
                VALUES (?, ?, ?)
                "#,
            id,
            way,
            email
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_event_reminder(&self, discord_id: UserId) -> Result {
        let id: i64 = discord_id.into();
        sqlx::query!(
            r#"
                DELETE FROM event_reminders WHERE user_id = ?
            "#,
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch_event_reminders(&self) -> TypedResult<Vec<EventReminder>> {
        Ok(sqlx::query_as!(
            EventReminder,
            r#"SELECT id, user_id, way as "way: ReminderWay", email FROM event_reminders"#
        )
        .fetch_all(&self.pool)
        .await?)
    }
}
