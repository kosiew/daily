use chrono::Utc;
use rusqlite::{Connection, Result, params};
use std::io::{self, Write};
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

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let mut runner = Runner::new(
            Scheduler::new(Duration::from_secs(60 * 20), false),
            init_db()?,
        );

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
}
