use crate::nytimes::{SolveState, XwordSummary};
use crate::util::*;

use chrono::prelude::*;
use rusqlite::{Connection, params};
use rusqlite::types::Null;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error(transparent)]
    DbError(#[from] rusqlite::Error),

    #[error(transparent)]
    DateError(#[from] chrono::ParseError)
}


pub struct Database {
    conn: Connection
}

impl Database {

    pub fn new(filename: &str) -> Result<Database, DbError> {
        Ok(Database {
            conn: Connection::open(filename)?
        })
    }

    pub fn get_last_solve(&self) -> Result<Option<Date<Utc>>, DbError> {
        let mut stmt = self.conn.prepare("SELECT * FROM last_solve")?;
        match stmt.query(params![])?.next()? {
            Some(row) => {
                let date = row.get::<usize, String>(0)?;
                return Ok(Some(string_to_date(&date)));
            },
            None => return Ok(None)
        };
    }

    pub fn set_last_solve(&self, date: Date<Utc>) -> Result<(), DbError> {
        let mut stmt = self.conn.prepare("REPLACE INTO last_solve VALUES (?)")?;
        let date = date_to_string(date);
        stmt.execute(params![date])?;
        Ok(())
    } 

    pub fn save_xwords(&mut self, xwords: &Vec<XwordSummary>) -> Result<(), DbError> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("REPLACE INTO xwords VALUES (?, ?, ?)")?;
            xwords.iter().for_each(|xword| {
                match xword.solve_state {
                    SolveState::Unsolved => stmt.execute(params![date_to_string(xword.print_date), false, Null]),
                    SolveState::Solved => stmt.execute(params![date_to_string(xword.print_date), true, Null]),
                    SolveState::Gold{ time } => stmt.execute(params![date_to_string(xword.print_date), true, time])
                }.expect("Failed inserting all xword times");
            });
        }
        tx.commit()?;
        Ok(())    
    }
}