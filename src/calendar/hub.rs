use std::{sync::Arc, time::Duration};

use chrono::Utc;
use tokio::{sync::Mutex, sync::RwLock, time::Instant};

use crate::{
    calendar::{Calendar, CalendarParser},
    log, log_error,
};

#[derive(Debug)]
pub struct CalendarHub {
    url: String,
    calendars: RwLock<Vec<Arc<Calendar>>>,
    last_refresh: Mutex<Instant>,
}

impl CalendarHub {
    pub async fn new(url: impl Into<String>) -> Self {
        let s = Self {
            url: url.into(),
            calendars: RwLock::new(vec![]),
            last_refresh: Mutex::new(Instant::now()),
        };

        s.update().await;

        s
    }

    pub async fn update(&self) {
        *self.calendars.write().await = match CalendarParser::fetch_and_parse(&self.url).await {
            Err(e) => {
                log_error!("Failed to update calendar! {e}");
                return;
            }
            Ok(v) => match v {
                Ok(r) => match r {
                    Ok(mut c) => {
                        let now = Utc::now();
                        c.iter_mut().for_each(|cal| {
                            cal.events.retain(|event| {
                                event.summary.contains("--BOT--") && event.start > now
                            });
                            cal.events.sort_unstable()
                        });
                        c.into_iter().map(Arc::new).collect()
                    }
                    Err(e) => {
                        log_error!("Failed to update calendar! {e}");
                        return;
                    }
                },
                Err(e) => {
                    log_error!("Failed to update calendar! {e}");
                    return;
                }
            },
        };

        *self.last_refresh.lock().await = Instant::now();
    }

    pub async fn get_calendar(&self, name: &str) -> Option<Arc<Calendar>> {
        if self.last_refresh.lock().await.elapsed() > Duration::from_secs(3600) {
            log!("Refreshing calendar cache!");
            self.update().await;
        }

        self.calendars
            .read()
            .await
            .iter()
            .find(|c| c.name == name)
            .cloned()
    }
}
