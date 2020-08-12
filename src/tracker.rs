use crate::database::Database;
use crate::nytimes::NYTimes;

use chrono::Duration;
use chrono::naive::NaiveDate;
use chrono::prelude::*;
use failure::Error;

static EARLIEST_SOLVE: &str = "2015-06-01";

pub struct Tracker {
    db: Database,
    nytimes: NYTimes
}

impl Tracker {
    pub fn new(session: String) -> Result<Tracker, Error> {
        Ok(Tracker{
            db: Database::new("xword.db")?,
            nytimes: NYTimes::new(session)?
        })
    }

    pub fn update_times(&mut self) -> Result<(), Error> {
        let mut curr = self.get_last_solve()?;
        let today = Utc::now().date();

        while curr <= today {
            let start = curr.format("%Y-%m-%d").to_string();
            let next = curr + Duration::days(30);
            let end = next.format("%Y-%m-%d").to_string();
            self.nytimes.get_history(&start, &end)?;

            curr = next;
        }

        

        Ok(())
    }

    fn get_last_solve(&mut self) -> Result<Date<Utc>, Error> {
        let last_solve = self.db.get_last_solve()?;
        let last_solve = match last_solve {
            Some(time) => NaiveDate::parse_from_str(&time, "%Y-%m-%d")?,
            None => NaiveDate::parse_from_str(EARLIEST_SOLVE, "%Y-%m-%d")?
        };
        Ok(Utc.from_utc_date(&last_solve))
    }
}