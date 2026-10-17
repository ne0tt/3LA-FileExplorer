use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use chrono::{Local, NaiveDateTime, TimeZone};

use crate::fsutil;

/// Lists trashed items without the `trash` crate's mount scan, which stats
/// every mount point and so triggers autofs/network automounts that can
/// stall for many seconds each. Only the home trash and trash dirs on real
/// block-device filesystems are considered.
pub fn list_items() -> Vec<trash::TrashItem> {
    let mut out = Vec::new();
    for (dir, top) in trash_dirs() {
        let Ok(rd) = fs::read_dir(dir.join("info")) else { continue };
        for d in rd.flatten() {
            let info = d.path();
            if info.extension().is_none_or(|e| e != "trashinfo") {
                continue;
            }
            // Orphaned info file: nothing left to restore.
            let Some(stem) = info.file_stem() else { continue };
            if fs::symlink_metadata(dir.join("files").join(stem)).is_err() {
                continue;
            }
            if let Some(item) = parse_info(&info, top.as_deref()) {
                out.push(item);
            }
        }
    }
    out
}

fn trash_dirs() -> Vec<(PathBuf, Option<PathBuf>)> {
    let home_trash = dirs::data_local_dir()
        .unwrap_or_else(|| fsutil::home().join(".local/share"))
        .join("Trash");
    let mut seen = HashSet::new();
    if let Ok(m) = fs::metadata(home_trash.join("info")) {
        seen.insert((m.dev(), m.ino()));
    }
    let mut v = vec![(home_trash, None)];
    // SAFETY: getuid has no failure modes.
    let uid = unsafe { libc::getuid() };
    let mounts = fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    for line in mounts.lines() {
        let mut f = line.split_whitespace();
        let (Some(dev), Some(mp)) = (f.next(), f.next()) else { continue };
        if !dev.starts_with("/dev/") {
            continue;
        }
        let top = PathBuf::from(unescape_mount(mp));
        for cand in volume_trash_dirs(&top, uid) {
            let Ok(m) = fs::symlink_metadata(cand.join("info")) else { continue };
            // Bind mounts expose the same trash dir more than once.
            if m.is_dir() && seen.insert((m.dev(), m.ino())) {
                v.push((cand, Some(top.clone())));
            }
        }
    }
    v
}

/// Volume trash dirs that pass the freedesktop spec's safety checks: a shared
/// `.Trash` must be a sticky, non-symlink directory, and the per-user dir must
/// be a real directory owned by `uid`.
fn volume_trash_dirs(top: &Path, uid: u32) -> Vec<PathBuf> {
    let owned = |p: &Path| {
        fs::symlink_metadata(p).is_ok_and(|m| m.is_dir() && m.uid() == uid)
    };
    let mut out = Vec::new();
    let shared = top.join(".Trash");
    if fs::symlink_metadata(&shared).is_ok_and(|m| m.is_dir() && m.mode() & 0o1000 != 0) {
        let user = shared.join(uid.to_string());
        if owned(&user) {
            out.push(user);
        }
    }
    let user = top.join(format!(".Trash-{uid}"));
    if owned(&user) {
        out.push(user);
    }
    out
}

/// /proc/mounts encodes spaces etc. as octal escapes ("\040").
fn unescape_mount(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 3 < b.len() {
            let oct = std::str::from_utf8(&b[i + 1..i + 4]).ok();
            if let Some(v) = oct.and_then(|o| u8::from_str_radix(o, 8).ok()) {
                out.push(v);
                i += 4;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn parse_info(info: &Path, top: Option<&Path>) -> Option<trash::TrashItem> {
    let text = fs::read_to_string(info).ok()?;
    let mut path = None;
    let mut date = 0i64;
    for line in text.lines() {
        if let Some(p) = line.strip_prefix("Path=") {
            path = Some(fsutil::percent_decode(p.trim()));
        } else if let Some(d) = line.strip_prefix("DeletionDate=") {
            date = NaiveDateTime::parse_from_str(d.trim(), "%Y-%m-%dT%H:%M:%S")
                .ok()
                .and_then(|n| Local.from_local_datetime(&n).single())
                .map(|t| t.timestamp())
                .unwrap_or(0);
        }
    }
    let path = PathBuf::from(path?);
    if path.components().any(|c| c == Component::ParentDir) {
        return None;
    }
    // Volume trash stores paths relative to the volume's top directory, and
    // must never restore outside it.
    let orig = match (path.is_absolute(), top) {
        (false, Some(top)) => top.join(path),
        (true, Some(top)) if !path.starts_with(top) => return None,
        (false, None) => return None,
        _ => path,
    };
    Some(trash::TrashItem {
        id: info.as_os_str().to_owned(),
        name: orig.file_name()?.to_owned(),
        original_parent: orig.parent()?.to_path_buf(),
        time_deleted: date,
    })
}

#[cfg(test)]
mod tests {
    use super::unescape_mount;

    #[test]
    fn mount_escapes() {
        assert_eq!(unescape_mount("/run/media/me/My\\040Disk"), "/run/media/me/My Disk");
        assert_eq!(unescape_mount("/mnt/plain"), "/mnt/plain");
        assert_eq!(unescape_mount("/x\\04"), "/x\\04");
    }
}

