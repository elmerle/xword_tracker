use failure::Error;
use reqwest::Client;
use serde::Deserialize;

use std::time::Duration;

pub struct NYTimes {
    session: String,
    client: Client
}

#[derive(Deserialize)]
struct XwordSummary {
    print_date: String,
    puzzle_id: u16,
    star: Option<String>
}

#[derive(Deserialize)]
pub struct XwordList {
    results: Vec<XwordSummary>
}

impl NYTimes {
    pub fn new(session_: String) -> Result<NYTimes, Error> {
        Ok(NYTimes {
            session: session_,
            client: Client::builder().timeout(Duration::from_secs(5)).connect_timeout(Duration::from_secs(5)).build()?
        })
    }

    pub async fn get_history(&self, start_date: String, end_date: String) -> reqwest::Result<XwordList> {
        println!("{} start", start_date);
        let url = format!("http://nyt-games-prd.appspot.com/svc/crosswords/v3/36569100/puzzles.json?publish_type=daily&date_start={}&date_end={}", start_date, end_date);
        let response = self.client.get(&url).send().await?;
        println!("{} got {}", start_date, response.status());
        let xword_list = response.json::<XwordList>().await?;
        println!("{} got {} results", start_date, xword_list.results.len());
        Ok(xword_list)
    }

}