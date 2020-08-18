use chrono::naive::NaiveDate;
use chrono::prelude::*;

use failure::Error;
use futures::Future;
use futures::stream::{self, StreamExt, TryStreamExt, futures_unordered::FuturesUnordered};
use reqwest::{Client, Method};
use serde::Deserialize;
use thiserror::Error;

use std::fmt;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct XwordDetail {
    calcs: XwordCalc
}

#[derive(Deserialize, Debug)]
struct XwordCalc {
    solved: bool,
    secondsSpentSolving: u16
}

#[derive(Deserialize, Debug)]
pub struct XwordSummaryInternal {
    print_date: String,
    puzzle_id: u16,
    solved: bool,
    star: Option<String>
}

#[derive(Deserialize)]
pub struct XwordList {
    results: Vec<XwordSummaryInternal>
}

impl fmt::Debug for XwordList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.results.len())
    }
}

#[derive(Debug)]
pub enum SolveState {
    Unsolved,
    Solved,
    Gold { time: u16 }
}

#[derive(Debug)]
pub struct XwordSummary {
    print_date: String,
    solve_state: SolveState
}

// impl XwordSummary {
//     fn from(xword: XwordSummaryInternal, solve_time: Option<u16>) -> Self {
//         XwordSummary {
//             print_date: xword.print_date,
//             solved: if xword.solved {
//                         match xword.star {
//                             Some(_) => SolveState::Gold{ time: solve_time },
//                             None => SolveState::Solved
//                         }
//                     } else {
//                         SolveState::Unsolved
//                     } 
//         }
//     }
// }

pub struct NYTimes {
    session: String,
    client: Client
}

#[derive(Error, Debug)]
pub enum NYTimesError {
    #[error("Invalid session token provided")]
    InvalidSessionError,

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error)
}

impl NYTimes {
    pub fn new(session_: String) -> Result<NYTimes, Error> {
        Ok(NYTimes {
            session: session_,
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(5))
                .connection_verbose(true)
                .build()?
        })
    }

    // async fn get_all_times_foo<T: Future>(&self, start_date: String, end_date: String) -> Result<(), NYTimesError> {
    //     let hist = self.get_history(start_date, end_date).await?;
    //     let futs = FuturesUnordered::new();
    //     for xword in hist { 
    //         futs.push(self.get_xword_time(xword.puzzle_id));
    //     }

    //     futs.try_collect().await;

    //     Ok::<(), NYTimesError>(())
    // }

    // pub async fn get_all_times(&self, start_date: Date<Utc>, end_date: Date<Utc>) -> Result<Vec<XwordSummary>, NYTimesError> {
    pub async fn get_all_times(&self, start_date: Date<Utc>, end_date: Date<Utc>) -> Result<Vec<XwordSummary>, NYTimesError> {
        let mut curr = start_date;
        let history_futs = FuturesUnordered::new();

        while curr <= end_date {
            let start = curr.format("%Y-%m-%d").to_string();
            let next = curr + chrono::Duration::days(30);
            let end = next.format("%Y-%m-%d").to_string();
            println!("adding {}", start);

            //futs.push(self.get_all_times_foo(start, end, futs));
            history_futs.push(self.get_history(start, end));

            curr = next;
        }

        let time_futs = FuturesUnordered::new();
        history_futs.try_collect::<Vec<_>>().await?.into_iter().flatten().for_each(|xword| {
            time_futs.push(self.process_xword_summary(xword));
        });
        
        Ok(time_futs.try_collect::<Vec<_>>().await?)

        // Err(NYTimesError::InvalidSessionError)
    }

    pub async fn get_times(&self, start_date: String, end_date: String) -> Result<Vec<XwordSummary>, NYTimesError> {
        let xword_list = self.get_history(start_date, end_date).await?;
        //let stream = stream::iter(xword_list);
        let stream = xword_list.iter().map(|xword| async move {
            println!("starting async fn for {}", xword.puzzle_id);
            let time = self.get_xword_time(xword.puzzle_id).await?;
            let solve_state = if xword.solved {
                match xword.star {
                    Some(_) => SolveState::Gold { time: time.expect(&format!("Missing time for {} xword", xword.print_date)) },
                    None => SolveState::Solved
                }
            } else {
                SolveState::Unsolved
            };

            Ok::<XwordSummary, NYTimesError>(XwordSummary {
                print_date: xword.print_date.clone(),
                solve_state: solve_state
            })
        });
        stream.collect::<FuturesUnordered<_>>().try_collect().await
    }

    pub async fn get_history(&self, start_date: String, end_date: String) -> Result<Vec<XwordSummaryInternal>, NYTimesError> {
    // pub async fn get_history(&self, start_date: String, end_date: String) -> reqwest::Result<Vec<XwordSummaryInternal>> {
        println!("getting history from {}", start_date);
        let url = format!("http://nyt-games-prd.appspot.com/svc/crosswords/v3/36569100/puzzles.json?publish_type=daily&date_start={}&date_end={}", start_date, end_date);
        let response = self.client.get(&url).header("nyt-s", &self.session).send().await?;
        let xword_list = response.json::<XwordList>().await?;
        println!("got history for {}", start_date);
        Ok(xword_list.results)
    }

    async fn process_xword_summary(&self, xword: XwordSummaryInternal) -> Result<XwordSummary, NYTimesError> {
        let time = self.get_xword_time(xword.puzzle_id).await?;
        let solve_state = if xword.solved {
            match xword.star {
                Some(_) => SolveState::Gold { time: time.expect(&format!("Missing time for {} xword", xword.print_date)) },
                None => SolveState::Solved
            }
        } else {
            SolveState::Unsolved
        };

        Ok(XwordSummary {
            print_date: xword.print_date.clone(),
            solve_state: solve_state
        })
    }

    async fn get_xword_time(&self, id: u16) -> Result<Option<u16>, NYTimesError> {
    //async fn get_xword_time(&self, id: u16) -> Result<(), NYTimesError> {
        println!("getting time for {}", id);
        let url = format!("https://nyt-games-prd.appspot.com/svc/crosswords/v6/game/{}.json", id);
        let response = self.client.get(&url).header("nyt-s", &self.session).send().await?;
        println!("got response for {}", id);
        if response.status() == reqwest::StatusCode::OK {
            let json = response.json::<XwordDetail>().await?;
            Ok(Some(json.calcs.secondsSpentSolving))
        } else {
            Err(NYTimesError::InvalidSessionError)
        }
    }

}