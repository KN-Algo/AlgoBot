use std::str::FromStr;

use chrono::NaiveDateTime;
use chrono_tz::Tz;
use ical::parser::ical::component::IcalCalendar;

use crate::{calendar::Event, map_properties};

#[derive(Debug)]
pub struct Calendar {
    pub name: String,
    pub description: String,
    pub events: Vec<Event>,
    pub timezone: Tz,
}

impl TryFrom<IcalCalendar> for Calendar {
    type Error = chrono::ParseError;
    fn try_from(value: IcalCalendar) -> Result<Self, Self::Error> {
        let events = value
            .events
            .iter()
            .cloned()
            .map(|ical_event| Event::try_from(ical_event))
            .collect::<Result<Vec<Event>, chrono::ParseError>>()?;

        let mut s = Self {
            events,
            name: String::default(),
            description: String::default(),
            timezone: Tz::default(),
        };

        map_properties!(value.properties,
            "X-WR-CALNAME" => s.name,
            "X-WR-CALDESC" => s.description,
            "X-WR-TIMEZONE" => s.timezone; with tz { tz.parse().map_err(|_| NaiveDateTime::from_str("invalid timezone").err().unwrap())? },
        );

        Ok(s)
    }
}
