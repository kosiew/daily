use crate::db::{print_timesheet, update_entry};
use crate::export::{export_csv, export_json, export_pdf};
use crate::integration::icloud_sync;
use rusqlite::Connection;
use std::path::Path;

pub fn handle_cli(conn: &Connection, args: &[String]) -> rusqlite::Result<bool> {
    if args.len() < 2 {
        return Ok(false);
    }
    match args[1].as_str() {
        "timesheet" => {
            print_timesheet(conn)?;
            Ok(true)
        }
        "edit" => {
            if args.len() >= 4 {
                let id: i64 = args[2].parse().unwrap_or(0);
                let act = &args[3];
                update_entry(conn, id, act)?;
            }
            Ok(true)
        }
        "export" => {
            if args.len() >= 4 {
                let fmt = &args[2];
                let path = Path::new(&args[3]);
                match fmt.as_str() {
                    "csv" => export_csv(conn, path)?,
                    "json" => export_json(conn, path)?,
                    "pdf" => export_pdf(conn, path)?,
                    _ => println!("unknown format"),
                }
            }
            Ok(true)
        }
        "sync" => {
            let db_path = Path::new("daily.db");
            match icloud_sync(db_path) {
                Ok(p) => println!("Synced to {}", p.display()),
                Err(e) => println!("Sync failed: {}", e),
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
