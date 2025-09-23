use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Pool, Sqlite};

use crate::{
    database::{Reminder, Task},
    log, log_error,
};

pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(filename: &str, max_connections: u32) -> Result<Self, sqlx::Error> {
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
                return Err(e);
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
                return Err(e);
            }
        }

        Ok(Self { pool: db })
    }

    async fn insert_user(&self, discord_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT OR IGNORE INTO users (discord_id) VALUES (?)",
            discord_id
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }

    async fn get_user_tasks(&self, discord_id: &str) -> Result<Vec<Task>, sqlx::Error> {
        self.insert_user(discord_id).await?;
        let rows = sqlx::query!(
            r#"
        SELECT 
            t.id AS task_id,
            t.title,
            t.description,
            t.completed,
            t.deadline_unixtimestamp,
            r.id AS reminder_id,
            r.task AS reminder_task,
            r.when_unixtimestamp as "when"
        FROM tasks t
        JOIN task_targets tt ON t.id = tt.task_id
        LEFT JOIN reminders r ON r.task = t.id
        WHERE tt.user_id = ?
        "#,
            discord_id
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
                deadline: Utc.timestamp_opt(row.deadline_unixtimestamp, 0).unwrap(),
                reminders: Vec::new(),
            });

            task.reminders.push(Reminder {
                when: chrono::Duration::new(row.when, 0).unwrap(),
            });
        }

        Ok(group_map.into_values().collect())
    }

    async fn add_reminder(&self, task_id: i64, when: chrono::Duration) -> Result<(), sqlx::Error> {
        let secs = when.num_seconds();
        sqlx::query!(
            r#"INSERT INTO reminders (task, when_unixtimestamp) VALUES (?, ?)"#,
            task_id,
            secs
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }

    async fn add_task(
        &self,
        title: &str,
        description: Option<&str>,
        deadline: DateTime<Utc>,
        targets: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        let timestamp = deadline.timestamp();
        sqlx::query!(
            r#"INSERT INTO tasks (title, description, deadline_unixtimestamp) VALUES (?, ?, ?)"#,
            title,
            description,
            timestamp
        )
        .execute(&mut *transaction)
        .await?;

        let task_id: i64 = sqlx::query_scalar!("SELECT last_insert_rowid()")
            .fetch_one(&mut *transaction)
            .await?;

        for target in targets {
            self.insert_user(&target).await?;
            sqlx::query!(
                r#"
                INSERT INTO task_targets (task_id, user_id)
                VALUES (?, ?)
                "#,
                task_id,
                target
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await
    }

    pub async fn toggle_task_completion(&self, task_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE tasks
            SET completed = NOT completed
            WHERE id = ?
            "#,
            task_id
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }

    pub async fn delete_completed_expired_tasks(&self) -> Result<(), sqlx::Error> {
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
        .await
        .map(|_| ())
    }
}
