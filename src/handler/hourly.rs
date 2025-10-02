use std::sync::Arc;
use std::time::Duration;

use serenity::all::Http;

use crate::{aliases::Result, calendar::CalendarHub, database::Db, log, log_error};

static INTERVAL: u64 = 1 * 60 * 60; // hours * minutes * seconds

async fn notify(http: &Http, calendar: &CalendarHub, db: &Db) -> Result {
    let reminders = db.fetch_event_reminders().await?;

    Ok(())
}

async fn cleanup(db: &Db) -> Result {
    db.delete_completed_expired_tasks().await?;
    db.delete_expired_custom_events().await
}

async fn add_calendar_events_as_discord_events(http: &Http, calendar: &CalendarHub) -> Result {
    Ok(())
}

pub async fn hourly_task(http: Arc<Http>, calendar: Arc<CalendarHub>, db: Arc<Db>) {
    let mut interval = tokio::time::interval(Duration::from_secs(INTERVAL));
    loop {
        interval.tick().await;

        log!("Running hourly task");

        match notify(&http, &calendar, &db).await {
            Ok(()) => log!("Users notified!"),
            Err(e) => {
                log_error!("Error notifying users: {e}");
                return;
            }
        }

        match add_calendar_events_as_discord_events(&http, &calendar).await {
            Ok(()) => log!("Events added as discord events!"),
            Err(e) => {
                log_error!("Error adding discord events: {e}");
                return;
            }
        }

        match cleanup(&db).await {
            Ok(()) => log!("Database cleaned up!"),
            Err(e) => {
                log_error!("Error cleaning up the database: {e}");
                return;
            }
        }
    }
}
