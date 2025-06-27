use cocoa::base::nil;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn init_db() -> Result<Connection> {
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

fn insert_entry(conn: &Connection, activity: &str) -> Result<()> {
    let ts = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO entries (activity, ts) VALUES (?1, ?2)",
        params![activity, ts],
    )?;
    Ok(())
}

fn prompt_dialog() -> String {
    print!("What are you working on? ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("failed to read line");
    input.trim().to_string()
}

struct Scheduler {
    interval: Duration,
    silent: bool,
}

impl Scheduler {
    fn new(interval: Duration, silent: bool) -> Self {
        Self { interval, silent }
    }

    fn run_once(&self, conn: &Connection) -> Result<()> {
        let activity = if self.silent {
            "Working".to_string()
        } else {
            prompt_dialog()
        };
        insert_entry(conn, &activity)
    }

    fn run(&self, conn: &Connection) -> Result<()> {
        loop {
            thread::sleep(self.interval);
            self.run_once(conn)?;
        }
    }
}

fn main() -> Result<()> {
    // placeholder for macOS UI initialization
    let _nsapp = nil;
    let conn = init_db()?;
    let scheduler = Scheduler::new(Duration::from_secs(60 * 20), false);
    println!("Starting scheduler. Press Ctrl+C to exit.");
    scheduler.run(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_memory_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE t(id INTEGER)", []).unwrap();
    }

    #[test]
    fn inserts_entry() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, activity TEXT NOT NULL, ts INTEGER NOT NULL)",
            [],
        )
        .unwrap();
        insert_entry(&conn, "Test").unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn scheduler_run_once() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, activity TEXT NOT NULL, ts INTEGER NOT NULL)",
            [],
        )
        .unwrap();
        let sched = Scheduler::new(Duration::from_millis(1), true);
        sched.run_once(&conn).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
