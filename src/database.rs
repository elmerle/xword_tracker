use rusqlite::{Connection, params, Result};

pub struct Database {
    conn: Connection
}

impl Database {

    pub fn new(filename: &str) -> Result<Database> {
        Ok(Database {
            conn: Connection::open(filename)?
        })
    }

    pub fn get_last_solve(&self) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT * FROM last_solve")?;
        match stmt.query(params![])?.next()? {
            Some(row) => {
                let date = row.get::<usize, String>(0)?;
                return Ok(Some(date))
            },
            None => return Ok(None)
        };
    }

}