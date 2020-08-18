use crate::database::{Database, DbError};
use crate::nytimes::{NYTimes, XwordSummary, NYTimesError, SolveState};
use crate::util::*;

use chrono::prelude::*;
use thiserror::Error;

static EARLIEST_SOLVE: &str = "2015-06-01";

#[derive(Error, Debug)]
pub enum TrackerError {
    // #[error("Invalid session token provided")]
    // InvalidSessionError,

    #[error(transparent)]
    NYTimesError(#[from] NYTimesError),

    #[error(transparent)]
    DbError(#[from] DbError),

    #[error(transparent)]
    DateError(#[from] chrono::ParseError)
}

pub struct Tracker {
    db: Database,
    nytimes: NYTimes
}

impl Tracker {
    pub fn new(session: String) -> Result<Tracker, TrackerError> {
        Ok(Tracker{
            db: Database::new("xword.db")?,
            nytimes: NYTimes::new(session)?
        })
    }

    pub async fn update_times(&mut self) -> Result<(), TrackerError> {
        let xwords = self.get_all_xwords().await?;
        self.db.save_xwords(&xwords)?;
        Ok(())
    }

    async fn get_all_xwords(&self) -> Result<Vec<XwordSummary>, TrackerError> {
        let start = self.get_last_solve()?;
        let today = Utc::now().date();

        let xwords = self.nytimes.get_all_times(start, today).await?;
        let latest_solve = xwords.iter().max_by_key(|x| {
            match x.solve_state {
                SolveState::Unsolved => string_to_date(EARLIEST_SOLVE),
                SolveState::Solved | SolveState::Gold { .. } => x.print_date
            }
        });
        if let Some(latest_solve) = latest_solve { 
            self.db.set_last_solve(latest_solve.print_date)?;
        }
        Ok(xwords)
    }

    fn get_last_solve(&self) -> Result<Date<Utc>, TrackerError> {
        let last_solve = self.db.get_last_solve()?;
        match last_solve {
            Some(time) => Ok(time),
            None => Ok(string_to_date(EARLIEST_SOLVE))
        }
    }

    pub fn calculate_stats(&self) {

    }
}