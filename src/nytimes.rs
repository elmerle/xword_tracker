use failure::Error;
use futures::stream::{self, StreamExt, TryStreamExt};
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
struct XwordSummaryInternal {
    print_date: String,
    puzzle_id: u16,
    solved: bool,
    star: Option<String>
}

#[derive(Deserialize)]
struct XwordList {
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
    solved: SolveState
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

    pub async fn get_times(&self, start_date: String, end_date: String) -> Result<Vec<XwordSummary>, NYTimesError> {
        let xword_list = self.get_history(start_date, end_date).await?.results;
        let stream = stream::iter(xword_list);
        let stream = stream.then(|xword| async move {
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
                print_date: xword.print_date,
                solved: solve_state
            })
        });
        stream.try_collect().await
    }

    async fn get_history(&self, start_date: String, end_date: String) -> reqwest::Result<XwordList> {
        println!("getting history from {}", start_date);
        let url = format!("http://nyt-games-prd.appspot.com/svc/crosswords/v3/36569100/puzzles.json?publish_type=daily&date_start={}&date_end={}", start_date, end_date);
        let response = self.client.get(&url).header("nyt-s", &self.session).send().await?;
        let xword_list = response.json::<XwordList>().await?;
        Ok(xword_list)
    }

    async fn get_xword_time(&self, id: u16) -> Result<Option<u16>, NYTimesError> {
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