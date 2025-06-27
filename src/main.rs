use cocoa::base::nil;
use rusqlite::{Connection, Result};

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

fn main() -> Result<()> {
    // placeholder for macOS UI initialization
    let _nsapp = nil;
    let _conn = init_db()?;
    println!("Project setup complete.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_memory_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE t(id INTEGER)", []).unwrap();
    }
}
