use rusqlite::{Connection, Result};

pub struct Db {
    conn: Connection
}


impl Db {    
    pub fn open() -> Result<Self> {
        let conn = Connection::open("staze.db")?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at INTEGER NOT NULL, -- unix timestamp
                duration_sec INTEGER NOT NULL
            );
        ")?;
        Ok( Self { conn } )
    }
    
    pub fn save_session(&self, started_at: u64, duration_sec: u64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (started_at, duration_sec) VALUES (?1, ?2)",
            (started_at as i64, duration_sec as i64),
        )?;
        Ok(())
    }
}
