use rusqlite::{Connection, Result};

pub struct SessionFilter {
    pub since: Option<i64>,
    pub tag: Option<String>,
}

pub struct SessionRecord {
    pub started_at: i64,
    pub duration_sec: i64,
    pub label: String,
}

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
                duration_sec INTEGER NOT NULL,
                label TEXT NOT NULL
            );
        ")?;
        Ok( Self { conn } )
    }
    
    pub fn save_session(&self, started_at: u64, duration_sec: u64, label: String) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (started_at, duration_sec, label) VALUES (?1, ?2, ?3)",
            (started_at as i64, duration_sec as i64, label),
        )?;
        Ok(())
    }

    pub fn get_labels(&self, prefix: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT label FROM sessions WHERE label LIKE ?1 ORDER BY label"
        )?;
        let rows = stmt.query_map([format!("{}%", prefix)], |row| row.get(0))?;
        rows.collect()
    }

    pub fn get_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionRecord>> {
        let mut query = "SELECT started_at, duration_sec, label FROM sessions WHERE 1=1".to_string();
        if filter.since.is_some() { query.push_str(" AND started_at >= :since"); }
        if filter.tag.is_some()   { query.push_str(" AND label = :tag"); }

        let mut stmt = self.conn.prepare(&query)?;

        let since_val = filter.since.unwrap_or(0);
        let tag_val = filter.tag.clone().unwrap_or_default();
        let mut params: Vec<(&str, &dyn rusqlite::types::ToSql)> = Vec::new();
        if filter.since.is_some() { params.push((":since", &since_val)); }
        if filter.tag.is_some()   { params.push((":tag",   &tag_val)); }

        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok(SessionRecord {
                started_at:   row.get(0)?,
                duration_sec: row.get(1)?,
                label:        row.get(2)?,
            })
        })?;
        rows.collect()
    }
}
