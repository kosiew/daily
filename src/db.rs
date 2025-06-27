use chrono::{TimeZone, Utc};
use rusqlite::{Connection, Result, params};
use std::path::Path;

use crate::security::{decrypt_file, encrypt_file};

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i64,
    pub activity: String,
    pub ts: i64,
}

pub fn init_db() -> Result<Connection> {
    if Path::new("daily.db.enc").exists() {
        let _ = decrypt_file(Path::new("daily.db.enc"), Path::new("daily.db"));
    }
    let conn = Connection::open("daily.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY,
            activity TEXT NOT NULL,
            ts INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn insert_entry(conn: &Connection, activity: &str) -> Result<()> {
    let ts = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO entries (activity, ts) VALUES (?1, ?2)",
        params![activity, ts],
    )?;
    Ok(())
}

pub fn fetch_entries(conn: &Connection) -> Result<Vec<Entry>> {
    let mut stmt = conn.prepare("SELECT id, activity, ts FROM entries ORDER BY ts")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Entry {
                id: row.get(0)?,
                activity: row.get(1)?,
                ts: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn update_entry(conn: &Connection, id: i64, activity: &str) -> Result<()> {
    conn.execute(
        "UPDATE entries SET activity = ?1 WHERE id = ?2",
        params![activity, id],
    )?;
    Ok(())
}

pub fn print_timesheet(conn: &Connection) -> Result<()> {
    let entries = fetch_entries(conn)?;
    for e in entries {
        let ts = Utc.timestamp_opt(e.ts, 0).single().unwrap();
        println!("{:<5} {} - {}", e.id, ts, e.activity);
    }
    Ok(())
}

pub fn encrypt_db() -> std::io::Result<()> {
    if Path::new("daily.db").exists() {
        encrypt_file(Path::new("daily.db"), Path::new("daily.db.enc"))?;
        std::fs::remove_file("daily.db")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, activity TEXT NOT NULL, ts INTEGER NOT NULL)",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn insert_and_fetch() {
        let conn = setup();
        insert_entry(&conn, "Test").unwrap();
        let entries = fetch_entries(&conn).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].activity, "Test");
    }

    #[test]
    fn update_changes_activity() {
        let conn = setup();
        insert_entry(&conn, "A").unwrap();
        update_entry(&conn, 1, "B").unwrap();
        let entries = fetch_entries(&conn).unwrap();
        assert_eq!(entries[0].activity, "B");
    }
}
