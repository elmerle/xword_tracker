use crate::tracker::{SolveState, XwordSummary};
use crate::util::*;

use chrono::prelude::*;
use futures::stream::{self, StreamExt, TryStreamExt};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use std::time::Duration;

#[derive(Deserialize, Debug)]
struct XwordDetail {
    calcs: XwordCalc
}

#[derive(Deserialize, Debug)]
struct XwordCalc {
    solved: bool,
    
    #[serde(rename="secondsSpentSolving")]
    seconds_spent_solving: u32
}

#[derive(Deserialize, Debug)]
struct XwordSummaryInternal {
    print_date: String,
    puzzle_id: u32,
    solved: bool,
    star: Option<String>
}

#[derive(Deserialize, Debug)]
struct XwordList {
    results: Vec<XwordSummaryInternal>
}

pub struct NYTimes {
    session: String,
    client: Client
}

#[derive(Error, Debug)]
pub enum NYTimesError {
    #[error("Invalid session token provided")]
    InvalidSessionError,

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    DateError(#[from] chrono::ParseError)
}

impl NYTimes {
    pub fn new(session_: String) -> Result<Self, NYTimesError> {
        Ok(NYTimes {
            session: session_,
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(5))
                .connection_verbose(true)
                .build()?
        })
    }

    pub async fn get_all_times(&self, start_date: Date<Utc>, end_date: Date<Utc>) -> Result<Vec<XwordSummary>, NYTimesError> {
        let mut curr = start_date;
        let mut history_futs = Vec::new();

        while curr <= end_date {
            let next = curr + chrono::Duration::days(30);
            history_futs.push(self.get_history(date_to_string(&curr), date_to_string(&next)));
            curr = next;
        }

        let mut time_futs = Vec::new();
        stream::iter(history_futs).buffer_unordered(10).try_collect::<Vec<_>>().await?.into_iter().flatten().for_each(|xword| {
            time_futs.push(self.process_xword_summary(xword));
        });
        
        Ok(stream::iter(time_futs).buffer_unordered(10).try_collect::<Vec<_>>().await?)
    }

    async fn get_history(&self, start_date: String, end_date: String) -> Result<Vec<XwordSummaryInternal>, NYTimesError> {
        println!("getting history from {}", start_date);
        let url = format!("http://nyt-games-prd.appspot.com/svc/crosswords/v3/50657393/puzzles.json?publish_type=daily&date_start={}&date_end={}", start_date, end_date);
        let response = self.client.get(&url).header("nyt-s", &self.session).send().await?;
        let xword_list = response.json::<XwordList>().await?;
        println!("got history for {}", start_date);
        Ok(xword_list.results)
    }

    async fn process_xword_summary(&self, xword: XwordSummaryInternal) -> Result<XwordSummary, NYTimesError> {
        println!("getting time for {} on {}", xword.puzzle_id, xword.print_date);
        let solve_state = if xword.solved {
            match xword.star {
                Some(_) => SolveState::Gold { 
                    time: self.get_xword_time(xword.puzzle_id).await?.expect(&format!("Missing time for {} xword", xword.print_date)) 
                },
                None => SolveState::Solved
            }
        } else {
            SolveState::Unsolved
        };

        Ok(XwordSummary {
            print_date: string_to_date(&xword.print_date),
            solve_state: solve_state
        })
    }

    async fn get_xword_time(&self, id: u32) -> Result<Option<u32>, NYTimesError> {
        let url = format!("https://nyt-games-prd.appspot.com/svc/crosswords/v6/game/{}.json", id);
        let response = self.client.get(&url).header("nyt-s", &self.session).send().await?;
        println!("got response for {}", id);
        if response.status() == reqwest::StatusCode::OK {
            let json = response.json::<XwordDetail>().await?;
            Ok(Some(json.calcs.seconds_spent_solving))
        } else {
            Err(NYTimesError::InvalidSessionError)
        }
    }

}