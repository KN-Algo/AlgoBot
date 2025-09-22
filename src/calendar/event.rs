use chrono::{DateTime, Utc};
use ical::parser::ical::component::IcalEvent;

use crate::{calendar::CalendarParser, map_properties};

#[derive(Debug, Clone)]
pub struct Event {
    pub uid: String,
    pub summary: String,
    pub start: DateTime<Utc>,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.start.partial_cmp(&other.start)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
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
