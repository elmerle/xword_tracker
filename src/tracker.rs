use crate::database::Database;
use crate::nytimes::NYTimes;
use crate::nytimes::XwordList;

use chrono::Duration;
use chrono::naive::NaiveDate;
use chrono::prelude::*;
use failure::Error;
use futures::executor::block_on;
use futures::future::join_all;
use futures::{stream, StreamExt, TryStreamExt};

static EARLIEST_SOLVE: &str = "2015-06-01";
//static EARLIEST_SOLVE: &str = "2019-07-01";

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

    pub async fn foo(&self) -> Result<(), Error> {
        //block_on(self.nytimes.get_history("2020-07-01", "2020-08-01")).expect("blah");
        let x = self.nytimes.get_all_times(
            Utc.from_utc_date(&NaiveDate::parse_from_str("2020-07-01", "%Y-%m-%d")?), 
            Utc.from_utc_date(&NaiveDate::parse_from_str("2020-08-01", "%Y-%m-%d")?)).await;
        println!("{:?}", x);
        Ok(())
    }

    // pub async fn update_times(&mut self) -> Result<(), Error> {
    //     let xwords = self.get_xwords();
    //     Ok(())
    // }

    // pub async fn get_xwords(&mut self) -> Result<(), Error> {
    //     let mut curr = self.get_last_solve()?;
    //     let today = Utc::now().date();
    //     let mut futs = vec![];

    //     while curr <= today {
    //         let start = curr.format("%Y-%m-%d").to_string();
    //         let next = curr + Duration::days(30);
    //         let end = next.format("%Y-%m-%d").to_string();
    //         futs.push(self.nytimes.get_history(start, end));

    //         curr = next;
    //     }

    //     //block_on(join_all(futs));
    //     let xwords = join_all(futs).await;
    //     //let st = stream::iter(futs);
    //     //st.try_collect().await;
        
    //     Ok(())
    // }

    fn get_last_solve(&mut self) -> Result<Date<Utc>, Error> {
        let last_solve = self.db.get_last_solve()?;
        let last_solve = match last_solve {
            Some(time) => NaiveDate::parse_from_str(&time, "%Y-%m-%d")?,
            None => NaiveDate::parse_from_str(EARLIEST_SOLVE, "%Y-%m-%d")?
        };
        Ok(Utc.from_utc_date(&last_solve))
    }
}