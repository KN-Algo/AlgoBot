use chrono::{DateTime, Utc};
use ical::parser::ical::component::IcalEvent;

use crate::{calendar::CalendarParser, map_properties};

#[derive(Debug)]
pub struct Event {
    uid: String,
    summary: String,
    start: DateTime<Utc>,
}

impl TryFrom<IcalEvent> for Event {
    type Error = chrono::ParseError;
    fn try_from(value: IcalEvent) -> Result<Self, Self::Error> {
        let mut s = Self {
            uid: String::default(),
            summary: String::default(),
            start: DateTime::default(),
        };

        map_properties!(value.properties,
            "SUMMARY" => s.summary,
            "DTSTART" => s.start; with date { CalendarParser::parse_date(&date)? },
            "UID" => s.uid,
        );

        Ok(s)
    }
}
