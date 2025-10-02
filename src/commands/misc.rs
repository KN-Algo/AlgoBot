use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use crate::aliases::TypedResult;

pub fn parse_date_dd_mm_yy(date: &str, format: &str) -> TypedResult<DateTime<Utc>> {
    let naive = NaiveDate::parse_from_str(date, format)?;
    let naive_date = naive.and_hms_opt(0, 0, 0).unwrap();
    Ok(Utc.from_utc_datetime(&naive_date))
}

pub fn verify_email(email: &str) -> bool {
    if email.is_empty() {
        return false;
    }

    let (user, domain) = match email.split_once('@') {
        Some(p) => p,
        None => return false,
    };

    if user.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || domain.len() > 255
        || domain.contains("..")
        || user.contains("..")
        || user.len() > 64
    {
        return false;
    }

    let bad_prefix_or_suffix = |s: &str| {
        s.starts_with('.')
            || s.ends_with('.')
            || s.starts_with('-')
            || s.ends_with('-')
            || s.starts_with('_')
            || s.ends_with('_')
    };

    let validate_chars = |v: &str, ap: &str| {
        v.chars()
            .all(|c| c.is_ascii_alphanumeric() || ap.contains(c))
    };

    if bad_prefix_or_suffix(user) || bad_prefix_or_suffix(domain) {
        return false;
    }

    if !validate_chars(user, "-._+%") || !validate_chars(domain, "-.") {
        return false;
    }

    return true;
}
