use crate::db::fetch_entries;
use rusqlite::Connection;
use std::io::Write;
use std::path::Path;

pub fn export_csv(conn: &Connection, path: &Path) -> rusqlite::Result<()> {
    use std::fs::File;
    let entries = fetch_entries(conn)?;
    let mut f = File::create(path)?;
    writeln!(f, "id,activity,ts")?;
    for e in entries {
        writeln!(f, "{},{},{}", e.id, e.activity.replace(',', " "), e.ts)?;
    }
    Ok(())
}

pub fn export_json(conn: &Connection, path: &Path) -> rusqlite::Result<()> {
    use std::fs::File;
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

pub fn export_pdf(conn: &Connection, path: &Path) -> rusqlite::Result<()> {
    use std::fs::File;
    let entries = fetch_entries(conn)?;
    let mut lines = Vec::new();
    lines.push("Timesheet".to_string());
    for e in entries {
        let ts = chrono::Utc.timestamp_opt(e.ts, 0).single().unwrap();
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
    offsets.push(0u64);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::insert_entry;
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
    fn export_csv_writes_header() {
        let conn = setup();
        insert_entry(&conn, "Task").unwrap();
        let path = std::env::temp_dir().join("test.csv");
        export_csv(&conn, &path).unwrap();
        let data = std::fs::read_to_string(path).unwrap();
        assert!(data.starts_with("id,activity,ts\n"));
    }

    #[test]
    fn export_json_brackets() {
        let conn = setup();
        insert_entry(&conn, "Task").unwrap();
        let path = std::env::temp_dir().join("test.json");
        export_json(&conn, &path).unwrap();
        let data = std::fs::read_to_string(path).unwrap();
        assert!(data.starts_with("["));
        assert!(data.ends_with("]"));
    }
}
