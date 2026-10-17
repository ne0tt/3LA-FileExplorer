use std::collections::HashMap;
use std::fs;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

use crate::fsutil;

/// Everything the app persists lives here.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| fsutil::home().join(".config"))
        .join("3la-fileexplorer")
}

pub fn bookmarks_file() -> PathBuf {
    config_dir().join("bookmarks")
}

pub fn settings_file() -> PathBuf {
    config_dir().join("settings.conf")
}

fn folders_file() -> PathBuf {
    config_dir().join("folders.json")
}

/// One-time move from the old locations (GTK bookmarks shared with Thunar,
/// Qt's default ~/.config/3LA/FileExplorer.conf). Originals are left alone.
pub fn migrate() {
    let _ = fs::create_dir_all(config_dir());
    let old_config = dirs::config_dir().unwrap_or_else(|| fsutil::home().join(".config"));
    let moves = [
        (old_config.join("gtk-3.0/bookmarks"), bookmarks_file()),
        (old_config.join("3LA/FileExplorer.conf"), settings_file()),
    ];
    for (from, to) in moves {
        if !to.exists() && from.exists() {
            let _ = fs::copy(&from, &to);
        }
    }
}

/// Reads folders.json; a corrupt file is set aside as .bak, not overwritten.
fn load_folders() -> HashMap<String, Value> {
    let file = folders_file();
    let Ok(text) = fs::read_to_string(&file) else { return HashMap::new() };
    serde_json::from_str(&text).unwrap_or_else(|_| {
        let _ = fs::rename(&file, file.with_extension("json.bak"));
        HashMap::new()
    })
}

fn folders() -> &'static Mutex<HashMap<String, Value>> {
    static F: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();
    F.get_or_init(|| Mutex::new(load_folders()))
}

/// Saved view settings for a folder as a JSON object string ("{}" if none).
pub fn folder_view(key: &str) -> String {
    folders()
        .lock()
        .unwrap()
        .get(key)
        .map(Value::to_string)
        .unwrap_or_else(|| "{}".into())
}

/// Every window is its own process, so the file is re-read under an flock
/// and merged before writing; otherwise the last writer drops the others' views.
pub fn set_folder_view(key: &str, json: &str) {
    let Ok(v) = serde_json::from_str::<Value>(json) else { return };
    let mut map = folders().lock().unwrap();
    let lock = fs::File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(config_dir().join("folders.lock"));
    if let Ok(f) = &lock {
        // SAFETY: f is an open file descriptor; the lock is released on close.
        unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX) };
    }
    *map = load_folders();
    map.insert(key.to_string(), v);
    let sorted: std::collections::BTreeMap<_, _> = map.iter().collect();
    if let Ok(text) = serde_json::to_string(&sorted) {
        let tmp = folders_file().with_extension(format!("json.{}.tmp", std::process::id()));
        if fs::write(&tmp, text).is_ok() {
            let _ = fs::rename(tmp, folders_file());
        }
    }
}
