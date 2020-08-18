use chrono::naive::NaiveDate;
use chrono::prelude::*;

pub fn date_to_string(date: Date<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn string_to_date(s: &str) -> Date<Utc> {
    Utc.from_utc_date(&NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("Invalid date string"))
}
