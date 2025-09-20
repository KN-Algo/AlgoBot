use std::io::Cursor;

use chrono::{DateTime, NaiveDateTime, ParseError, TimeZone, Utc};
use ical::{parser::ParserError, IcalParser};

use crate::calendar::Calendar;

pub struct CalendarParser;

impl CalendarParser {
    async fn fetch(url: &str) -> Result<String, reqwest::Error> {
        reqwest::get(url).await?.text().await
    }

    fn parse(data: String) -> Result<Result<Vec<Calendar>, chrono::ParseError>, ParserError> {
        let parser = IcalParser::new(Cursor::new(data));
        parser
            .map(|maybe_ical| maybe_ical.map(|ical| Calendar::try_from(ical)))
            .collect::<Result<Result<Vec<Calendar>, chrono::ParseError>, ParserError>>()
    }

    pub async fn fetch_and_parse(
        url: &str,
    ) -> Result<Result<Result<Vec<Calendar>, chrono::ParseError>, ParserError>, reqwest::Error>
    {
        Ok(Self::parse(Self::fetch(url).await?))
    }

    pub fn parse_date(date: &str) -> Result<DateTime<Utc>, ParseError> {
        let date = date.trim();
        if date.ends_with('Z') {
            let naive = NaiveDateTime::parse_from_str(date, "%Y%m%dT%H%M%SZ")?;
            Ok(Utc.from_utc_datetime(&naive))
        } else if date.len() == 8 {
            let naive =
                match chrono::NaiveDate::parse_from_str(date, "%Y%m%d")?.and_hms_opt(0, 0, 0) {
                    Some(hms) => hms,
                    None => NaiveDateTime::default(),
                };
            Ok(Utc.from_utc_datetime(&naive))
        } else {
            let naive = chrono::NaiveDateTime::parse_from_str(date, "%Y%m%dT%H%M%S")?;
            Ok(Utc.from_utc_datetime(&naive))
        }
    }
}
