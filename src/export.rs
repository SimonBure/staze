//! CSV export of session data.
//!
//! Pure, DB-agnostic: callers fetch `SessionRecord`s through `App`/`db.rs`,
//! then hand the already-loaded rows here. This module only formats + writes,
//! and opens the containing directory in the OS file manager.

use std::error::Error;
use std::fs::File;
use std::path::Path;

use csv;

use crate::db::SessionRecord;


/// Write every session to `path` as CSV.
///
/// Suggested columns (header first): `started_at,duration_sec,label`.
/// `label` is `Option<String>` — decide how `None` serializes (empty field).
/// Use `csv::Writer::from_path(path)`; let the `csv` crate handle quoting.
pub fn write_sessions_csv(
    path: &Path,
    sessions: &[SessionRecord],
) -> Result<(), Box<dyn Error>> {
    let _ = (path, sessions);
    let f = File::create(path)?;
    let mut wtr = csv::Writer::from_writer(f);
    
    // Header
    wtr.write_record(["started", "duration (s)", "label"])?;

    for s in sessions {
        let label = s.label.clone().unwrap_or("-".to_string());
        let record = [s.started_at.to_string(), s.duration_sec.to_string(), label]; 
        wtr.write_record(record)?;
    }
    wtr.flush()?;
    
    Ok(())
}

pub fn open_dir(dir: &Path) -> std::io::Result<()> {
    let _ = dir;
    let command = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    
    std::process::Command::new(command).arg(dir).spawn()?;
    return Ok(())
}
