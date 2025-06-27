use std::io;
use std::path::{Path, PathBuf};

use crate::security::encrypt_file;

pub fn icloud_sync(db: &Path) -> io::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let dest = Path::new(&home)
            .join("Library")
            .join("Mobile Documents")
            .join("daily.db.enc");
        std::fs::create_dir_all(dest.parent().unwrap())?;
        encrypt_file(db, &dest)?;
        Ok(dest)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let dest = db.with_extension("icloud.enc");
        encrypt_file(db, &dest)?;
        Ok(dest)
    }
}

pub fn run_applescript(script: &str) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()?;
    }
    Ok(())
}
