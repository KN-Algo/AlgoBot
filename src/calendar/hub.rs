use crate::{
    calendar::{Calendar, CalendarParser},
    log_error,
};

#[derive(Debug)]
pub struct CalendarHub {
    url: String,
    calendars: Vec<Calendar>,
}

impl CalendarHub {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            calendars: vec![],
        }
    }

    pub async fn update(&mut self) {
        self.calendars = match CalendarParser::fetch_and_parse(&self.url).await {
            Err(e) => {
                log_error!("Failed to update calendar! {e}");
                return;
            }
            Ok(v) => match v {
                Ok(r) => match r {
                    Ok(c) => c,
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
        }
    }
}
