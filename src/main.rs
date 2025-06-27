use chrono::{TimeZone, Utc};
use rusqlite::{Connection, Result, params};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyAccessory, NSMenu, NSMenuItem, NSStatusBar,
    NSVariableStatusItemLength,
};
#[cfg(target_os = "macos")]
use cocoa::base::{YES, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSAutoreleasePool, NSString};

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

#[derive(Debug, Clone)]
struct Entry {
    id: i64,
    activity: String,
    ts: i64,
}

fn fetch_entries(conn: &Connection) -> Result<Vec<Entry>> {
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

fn update_entry(conn: &Connection, id: i64, activity: &str) -> Result<()> {
    conn.execute(
        "UPDATE entries SET activity = ?1 WHERE id = ?2",
        params![activity, id],
    )?;
    Ok(())
}

fn print_timesheet(conn: &Connection) -> Result<()> {
    let entries = fetch_entries(conn)?;
    for e in entries {
        let ts = Utc.timestamp_opt(e.ts, 0).single().unwrap();
        println!("{:<5} {} - {}", e.id, ts, e.activity);
    }
    Ok(())
}

fn export_csv(conn: &Connection, path: &Path) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    let entries = fetch_entries(conn)?;
    let mut f = File::create(path)?;
    writeln!(f, "id,activity,ts")?;
    for e in entries {
        writeln!(f, "{},{},{}", e.id, e.activity.replace(',', " "), e.ts)?;
    }
    Ok(())
}

fn export_json(conn: &Connection, path: &Path) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    let entries = fetch_entries(conn)?;
    let mut f = File::create(path)?;
    write!(f, "[")?;
    for (i, e) in entries.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write!(
            f,
            "{{\"id\":{},\"activity\":\"{}\",\"ts\":{}}}",
            e.id,
            e.activity.replace('"', "\""),
            e.ts
        )?;
    }
    write!(f, "]")?;
    Ok(())
}

fn export_pdf(conn: &Connection, path: &Path) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let entries = fetch_entries(conn)?;
    let mut lines = Vec::new();
    lines.push("Timesheet".to_string());
    for e in entries {
        let ts = Utc.timestamp_opt(e.ts, 0).single().unwrap();
        lines.push(format!("{} - {}", ts.format("%Y-%m-%d %H:%M"), e.activity));
    }

    let mut objects = Vec::new();
    objects.push("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string());
    objects.push("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n".to_string());
    objects.push("3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n".to_string());

    let mut content = String::from("BT /F1 12 Tf\n");
    let mut y = 800;
    for line in lines {
        content.push_str(&format!(
            "1 0 0 1 50 {} Tm ({} ) Tj\n",
            y,
            line.replace('(', "\\(")
                .replace(')', "\\)")
                .replace("\\", "\\\\")
        ));
        y -= 14;
    }
    content.push_str("ET");

    objects.push(format!(
        "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        content.len(),
        content
    ));
    objects.push(
        "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_string(),
    );

    let mut file = File::create(path)?;
    file.write_all(b"%PDF-1.4\n")?;
    let mut offsets = Vec::new();
    offsets.push(0u64); // for object 0
    for obj in &objects {
        let pos = file.stream_position()?;
        offsets.push(pos);
        file.write_all(obj.as_bytes())?;
    }
    let xref = file.stream_position()?;
    writeln!(file, "xref")?;
    writeln!(file, "0 {}", objects.len() + 1)?;
    writeln!(file, "0000000000 65535 f ")?;
    for off in offsets.iter().skip(1) {
        writeln!(file, "{:010} 00000 n ", off)?;
    }
    writeln!(
        file,
        "trailer\n<< /Root 1 0 R /Size {} >>",
        objects.len() + 1
    )?;
    writeln!(file, "startxref")?;
    writeln!(file, "{}", xref)?;
    writeln!(file, "%%EOF")?;
    Ok(())
}

fn prompt_dialog() -> String {
    print!("What are you working on? ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");
    input.trim().to_string()
}

#[derive(Clone)]
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

    fn run(&self, conn: &Connection, running: &AtomicBool) -> Result<()> {
        while running.load(Ordering::SeqCst) {
            thread::sleep(self.interval);
            self.run_once(conn)?;
        }
        Ok(())
    }
}

struct Runner {
    scheduler: Scheduler,
    conn: Arc<Mutex<Connection>>,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Runner {
    fn new(scheduler: Scheduler, conn: Connection) -> Self {
        Self {
            scheduler,
            conn: Arc::new(Mutex::new(conn)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    fn start(&mut self) {
        if self.handle.is_some() {
            return;
        }
        self.running.store(true, Ordering::SeqCst);
        let sched = self.scheduler.clone();
        let conn = self.conn.clone();
        let run_flag = self.running.clone();
        self.handle = Some(thread::spawn(move || {
            let conn = conn.lock().expect("lock");
            sched.run(&conn, &run_flag).unwrap();
        }));
    }

    fn stop(&mut self) {
        if let Some(h) = self.handle.take() {
            self.running.store(false, Ordering::SeqCst);
            let _ = h.join();
        }
    }
}

fn handle_cli(conn: &Connection, args: &[String]) -> Result<bool> {
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
        _ => Ok(false),
    }
}

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
    let conn = init_db()?;
    if handle_cli(&conn, &std::env::args().collect::<Vec<_>>())? {
        return Ok(());
    }
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let mut runner = Runner::new(Scheduler::new(Duration::from_secs(60 * 20), false), conn);

        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);

        let status_item =
            NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSVariableStatusItemLength);
        status_item
            .button()
            .setTitle_(NSString::alloc(nil).init_str("Daily"));

        let menu = NSMenu::new(nil).autorelease();
        let start_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("Start Tracking"),
            sel!(startTracking:),
            NSString::alloc(nil).init_str(""),
        );
        menu.addItem_(start_item);
        let stop_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("Stop Tracking"),
            sel!(stopTracking:),
            NSString::alloc(nil).init_str(""),
        );
        menu.addItem_(stop_item);
        menu.addItem_(NSMenuItem::separatorItem(nil));
        let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("Quit"),
            sel!(terminate:),
            NSString::alloc(nil).init_str("q"),
        );
        menu.addItem_(quit_item);
        status_item.setMenu_(menu);

        // start tracking immediately
        runner.start();
        app.run();
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() -> Result<()> {
    let conn = init_db()?;
    if handle_cli(&conn, &std::env::args().collect::<Vec<_>>())? {
        return Ok(());
    }
    let scheduler = Scheduler::new(Duration::from_secs(60 * 20), false);
    let mut runner = Runner::new(scheduler, conn);
    println!("Starting scheduler without macOS UI. Press Ctrl+C to exit.");
    runner.start();
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
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
        let sched = Scheduler::new(Duration::from_millis(1), true);
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
        insert_entry(&conn, "A").unwrap();
        update_entry(&conn, 1, "B").unwrap();
        let ents = fetch_entries(&conn).unwrap();
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
        insert_entry(&conn, "A").unwrap();
        export_csv(&conn, Path::new("/tmp/test.csv")).unwrap();
        export_json(&conn, Path::new("/tmp/test.json")).unwrap();
        export_pdf(&conn, Path::new("/tmp/test.pdf")).unwrap();
        assert!(std::fs::metadata("/tmp/test.csv").is_ok());
        assert!(std::fs::metadata("/tmp/test.json").is_ok());
        assert!(std::fs::metadata("/tmp/test.pdf").is_ok());
    }
}
