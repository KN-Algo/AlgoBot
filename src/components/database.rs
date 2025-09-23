use sqlx::{Pool, Sqlite};

use crate::{log, log_error};

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
}
