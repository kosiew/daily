mod cli;
mod db;
mod export;
mod idle;
mod integration;
mod scheduler;
mod security;

use crate::cli::handle_cli;
use crate::db::init_db;
use crate::scheduler::{Runner, Scheduler};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

fn create_tray() -> SystemTray {
    let start = CustomMenuItem::new("start".to_string(), "Start Tracking");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop Tracking");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let menu = SystemTrayMenu::new()
        .add_item(start)
        .add_item(stop)
        .add_item(quit);
    SystemTray::new().with_menu(menu)
}

fn main() -> tauri::Result<()> {
    let conn = init_db().expect("db");
    let args = std::env::args().collect::<Vec<_>>();
    if handle_cli(&conn, &args)? {
        drop(conn);
        db::encrypt_db().ok();
        return Ok(());
    }
    let runner = Runner::new(Scheduler::new(Duration::from_secs(60 * 20), false), conn);
    let runner_state = tauri::State::new(Mutex::new(runner));
    tauri::Builder::default()
        .manage(runner_state)
        .system_tray(create_tray())
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                let mut runner = app.state::<Mutex<Runner>>().lock().unwrap();
                match id.as_str() {
                    "start" => runner.start(),
                    "stop" => runner.stop(),
                    "quit" => std::process::exit(0),
                    _ => {}
                }
            }
        })
        .run(tauri::generate_context!())?;
    db::encrypt_db().ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::path::Path;

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
        db::insert_entry(&conn, "Test").unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .unwrap();
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
        let sched = Scheduler::new(std::time::Duration::from_millis(1), true);
        sched.run_once(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn update_and_fetch() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, activity TEXT NOT NULL, ts INTEGER NOT NULL)",
            [],
        )
        .unwrap();
        db::insert_entry(&conn, "A").unwrap();
        db::update_entry(&conn, 1, "B").unwrap();
        let ents = db::fetch_entries(&conn).unwrap();
        assert_eq!(ents.len(), 1);
        assert_eq!(ents[0].activity, "B");
    }

    #[test]
    fn export_creates_files() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, activity TEXT NOT NULL, ts INTEGER NOT NULL)",
            [],
        )
        .unwrap();
        db::insert_entry(&conn, "A").unwrap();
        export::export_csv(&conn, Path::new("/tmp/test.csv")).unwrap();
        export::export_json(&conn, Path::new("/tmp/test.json")).unwrap();
        export::export_pdf(&conn, Path::new("/tmp/test.pdf")).unwrap();
        assert!(std::fs::metadata("/tmp/test.csv").is_ok());
        assert!(std::fs::metadata("/tmp/test.json").is_ok());
        assert!(std::fs::metadata("/tmp/test.pdf").is_ok());
    }
}
