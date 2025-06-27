use crate::db::insert_entry;
use rusqlite::Connection;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Scheduler {
    pub interval: Duration,
    pub silent: bool,
}

impl Scheduler {
    pub fn new(interval: Duration, silent: bool) -> Self {
        Self { interval, silent }
    }

    pub fn run_once(&self, conn: &Connection) -> rusqlite::Result<()> {
        let activity = if self.silent {
            "Working".to_string()
        } else {
            prompt_dialog()
        };
        insert_entry(conn, &activity)
    }

    pub fn run(&self, conn: &Connection, running: &AtomicBool) -> rusqlite::Result<()> {
        while running.load(Ordering::SeqCst) {
            thread::sleep(self.interval);
            self.run_once(conn)?;
        }
        Ok(())
    }
}

pub struct Runner {
    scheduler: Scheduler,
    conn: Arc<Mutex<Connection>>,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Runner {
    pub fn new(scheduler: Scheduler, conn: Connection) -> Self {
        Self {
            scheduler,
            conn: Arc::new(Mutex::new(conn)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn start(&mut self) {
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

    pub fn stop(&mut self) {
        if let Some(h) = self.handle.take() {
            self.running.store(false, Ordering::SeqCst);
            let _ = h.join();
        }
    }
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
