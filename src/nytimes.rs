use failure::Error;
use reqwest::Request;

pub struct NYTimes {
    session: String
}


impl NYTimes {
    pub fn new(session_: String) -> Result<NYTimes, Error> {
        Ok(NYTimes {
            session: session_
        })
    }

    pub fn get_history(&self, start_date: &str, end_date: &str) -> Result<(), Error> {
        Ok(())
    }

}