use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use chrono::format::{Item, StrftimeItems};
use chrono::{Local, TimeZone};

use crate::bridge::native::ffi as native;
use crate::icons;

#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub broken_link: bool,
    pub link_target: String,
    pub size: u64,
    pub mtime: i64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    /// Interned: thousands of entries share a handful of values.
    pub mime: Arc<str>,
    pub type_text: Arc<str>,
    pub hidden: bool,
    pub child_count: Option<u32>,
    pub glyph: &'static str,
    pub category: &'static str,
    /// Only set for items listed from the trash.
    pub trash: Option<Box<TrashInfo>>,
}

#[derive(Clone, Debug, Default)]
pub struct TrashInfo {
    pub orig_path: String,
    pub deleted: i64,
}

/// Shared copy of a short, frequently repeated string.
pub fn intern(s: &str) -> Arc<str> {
    static SET: OnceLock<Mutex<HashSet<Arc<str>>>> = OnceLock::new();
    let mut set = SET.get_or_init(Default::default).lock().unwrap();
    if let Some(a) = set.get(s) {
        return a.clone();
    }
    let a: Arc<str> = Arc::from(s);
    set.insert(a.clone());
    a
}

impl Entry {
    pub fn changed_from(&self, other: &Entry) -> bool {
        self.mtime != other.mtime
            || self.size != other.size
            || self.mode != other.mode
            || self.is_dir != other.is_dir
            || self.link_target != other.link_target
            || self.child_count != other.child_count
            || self.path != other.path
            || self.is_link != other.is_link
            || self.broken_link != other.broken_link
            || self.uid != other.uid
            || self.gid != other.gid
            || self.mime != other.mime
    }

    pub fn is_exec(&self) -> bool {
        !self.is_dir && self.mode & 0o111 != 0
    }

    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }
}

pub fn stat_entry(path: &Path, count_children: bool) -> Option<Entry> {
    let lmeta = fs::symlink_metadata(path).ok()?;
    let is_link = lmeta.file_type().is_symlink();
    let (meta, broken) = if is_link {
        match fs::metadata(path) {
            Ok(m) => (m, false),
            Err(_) => (lmeta.clone(), true),
        }
    } else {
        (lmeta.clone(), false)
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    let path_str = path.to_string_lossy().into_owned();
    let is_dir = meta.is_dir();
    // The icon follows the extension alone (as SPX-FileExplorer does);
    // `mime` may also come from content sniffing.
    let (ext_mime, mime) = if is_dir {
        (String::new(), intern("inode/directory"))
    } else if broken {
        (native::mime_type(&path_str, false), intern("inode/symlink"))
    } else {
        let (by_name, mime) = detect_mime(&path_str, &meta);
        (by_name, intern(&mime))
    };
    let type_text = if is_dir {
        intern("Folder")
    } else if broken {
        intern("Broken link")
    } else {
        mime_comment(&mime)
    };
    let child_count = if is_dir && count_children { child_count(path) } else { None };
    let mut e = Entry {
        hidden: name.starts_with('.') || name.ends_with('~'),
        link_target: if is_link {
            fs::read_link(path)
                .map(|t| t.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else {
            String::new()
        },
        name,
        path: path_str,
        is_dir,
        is_link,
        broken_link: broken,
        size: if is_dir { 0 } else { meta.len() },
        mtime: meta.mtime(),
        mode: meta.mode(),
        uid: meta.uid(),
        gid: meta.gid(),
        mime,
        type_text,
        child_count,
        ..Default::default()
    };
    let (glyph, category) = icons::for_entry(&e, &ext_mime);
    e.glyph = glyph;
    e.category = category;
    if category == "encrypted" {
        e.type_text = intern("Encrypted file");
    }
    Some(e)
}

/// (extension-only mime, effective mime). Extension match first; content
/// sniffing only for regular files the extension can't identify. Opening a
/// FIFO to sniff it would block forever while holding QMimeDatabase's global
/// lock, freezing every other caller.
fn detect_mime(path: &str, meta: &fs::Metadata) -> (String, String) {
    use std::os::unix::fs::FileTypeExt;
    let ft = meta.file_type();
    if !ft.is_file() {
        let mime = if ft.is_fifo() {
            "inode/fifo"
        } else if ft.is_socket() {
            "inode/socket"
        } else if ft.is_char_device() {
            "inode/chardevice"
        } else if ft.is_block_device() {
            "inode/blockdevice"
        } else {
            "application/octet-stream"
        };
        return (String::new(), mime.to_string());
    }
    let by_name = native::mime_type(path, false);
    let mime = if by_name == "application/octet-stream" && meta.len() > 0 {
        native::mime_type(path, true)
    } else {
        by_name.clone()
    };
    (by_name, mime)
}

/// Lists `dir` without child counts (see `child_count`). Gives up early,
/// returning Err, once `stale()` reports the listing is no longer wanted.
pub fn read_dir(dir: &Path, stale: &dyn Fn() -> bool) -> Result<Vec<Entry>, String> {
    let rd = fs::read_dir(dir).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for (i, d) in rd.flatten().enumerate() {
        if i % 128 == 127 && stale() {
            return Err("superseded".into());
        }
        if let Some(e) = stat_entry(&d.path(), false) {
            out.push(e);
        }
    }
    Ok(out)
}

pub fn child_count(dir: &Path) -> Option<u32> {
    fs::read_dir(dir).ok().map(|rd| rd.count() as u32)
}

/// Network and FUSE (gvfs, sshfs) filesystems, where every extra directory
/// read is a round trip.
pub fn is_remote_fs(path: &Path) -> bool {
    const REMOTE: &[i64] = &[
        0x6969,               // NFS
        0x517B,               // SMB
        0xFF534D42u32 as i64, // CIFS
        0xFE534D42u32 as i64, // SMB2
        0x65735546,           // FUSE
        0x564C,               // NCP
        0x73757245,           // Coda
        0x01021997,           // 9p
    ];
    let Ok(c) = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()) else { return false };
    let mut st: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: c is a valid NUL-terminated path and st a valid out pointer.
    if unsafe { libc::statfs(c.as_ptr(), &mut st) } != 0 {
        return false;
    }
    REMOTE.contains(&(st.f_type as i64))
}

fn mime_comment(mime: &Arc<str>) -> Arc<str> {
    static CACHE: OnceLock<Mutex<HashMap<Arc<str>, Arc<str>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(c) = cache.lock().unwrap().get(mime) {
        return c.clone();
    }
    let c = native::mime_comment(mime);
    let c = if c.is_empty() { mime.clone() } else { intern(&c) };
    cache.lock().unwrap().insert(mime.clone(), c.clone());
    c
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SortKey {
    #[default]
    Name,
    Size,
    Type,
    Modified,
}

impl SortKey {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Size,
            2 => Self::Type,
            3 => Self::Modified,
            _ => Self::Name,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SortSpec {
    pub key: SortKey,
    pub descending: bool,
    pub folders_first: bool,
}

/// By name, always: hidden folders, folders, hidden files, files (the groups
/// keep that order when descending; only names within a group flip), names
/// compared case-insensitively. On Size/Type/Modified the column decides, as
/// in SPX-FileExplorer (folders count as size 0), grouped folders-first only
/// when that option is on, with an A-Z name tie-break that does not flip
/// when descending.
pub fn compare(a: &Entry, b: &Entry, spec: SortSpec) -> Ordering {
    let flip = |o: Ordering| if spec.descending { o.reverse() } else { o };
    if spec.key == SortKey::Name {
        if a.is_dir != b.is_dir {
            return if a.is_dir { Ordering::Less } else { Ordering::Greater };
        }
        if a.hidden != b.hidden {
            return if a.hidden { Ordering::Less } else { Ordering::Greater };
        }
        return flip(lower_cmp(&a.name, &b.name).then_with(|| a.name.cmp(&b.name)))
            .then_with(|| a.path.cmp(&b.path));
    }
    if spec.folders_first && a.is_dir != b.is_dir {
        return if a.is_dir { Ordering::Less } else { Ordering::Greater };
    }
    let size = |e: &Entry| if e.is_dir { 0 } else { e.size };
    let primary = match spec.key {
        SortKey::Size => size(a).cmp(&size(b)),
        SortKey::Type => lower_cmp(&a.mime, &b.mime),
        _ => a.mtime.cmp(&b.mtime),
    };
    flip(primary)
        .then_with(|| lower_cmp(&a.name, &b.name))
        .then_with(|| a.name.cmp(&b.name))
        .then_with(|| a.path.cmp(&b.path))
}

/// Case-insensitive comparison without allocating (Python's `str.lower()` order).
fn lower_cmp(a: &str, b: &str) -> Ordering {
    a.chars().flat_map(char::to_lowercase).cmp(b.chars().flat_map(char::to_lowercase))
}

pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let a = a.trim_start_matches('.');
    let b = b.trim_start_matches('.');
    let (mut a, mut b) = (a, b);
    loop {
        let (Some(ca), Some(cb)) = (a.chars().next(), b.chars().next()) else {
            return a.is_empty().cmp(&b.is_empty()).reverse();
        };
        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            let (na, ra) = split_number(a);
            let (nb, rb) = split_number(b);
            let ta = na.trim_start_matches('0');
            let tb = nb.trim_start_matches('0');
            let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
            if ord != Ordering::Equal {
                return ord;
            }
            (a, b) = (ra, rb);
            continue;
        }
        let la = ca.to_lowercase().next().unwrap_or(ca);
        let lb = cb.to_lowercase().next().unwrap_or(cb);
        if la != lb {
            return la.cmp(&lb);
        }
        (a, b) = (&a[ca.len_utf8()..], &b[cb.len_utf8()..]);
    }
}

/// Splits a leading run of ASCII digits off `s`.
fn split_number(s: &str) -> (&str, &str) {
    let n = s.bytes().take_while(u8::is_ascii_digit).count();
    s.split_at(n)
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["bytes", "kB", "MB", "GB", "TB", "PB"];
    if bytes < 1000 {
        return format!("{bytes} {}", if bytes == 1 { "byte" } else { "bytes" });
    }
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1000.0 && u < UNITS.len() - 1 {
        v /= 1000.0;
        u += 1;
    }
    format!("{v:.1} {}", UNITS[u])
}

/// strftime pattern for list dates; "relative" keeps Thunar's Today/Yesterday style.
pub const DEFAULT_DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
pub const RELATIVE_DATE_FORMAT: &str = "relative";

/// The active pattern, pre-parsed once rather than on every formatted cell.
struct DateFormat {
    raw: String,
    items: Option<Vec<Item<'static>>>, // None: relative
}

fn date_format_cell() -> &'static RwLock<Arc<DateFormat>> {
    static F: OnceLock<RwLock<Arc<DateFormat>>> = OnceLock::new();
    F.get_or_init(|| RwLock::new(Arc::new(parse_date_format(DEFAULT_DATE_FORMAT).unwrap())))
}

/// chrono panics when Display-ing a malformed pattern, so patterns are
/// validated (and parsed once) here.
fn parse_date_format(fmt: &str) -> Option<DateFormat> {
    if fmt == RELATIVE_DATE_FORMAT {
        return Some(DateFormat { raw: fmt.into(), items: None });
    }
    if fmt.trim().is_empty() {
        return None;
    }
    let items = StrftimeItems::new(fmt).parse_to_owned().ok()?;
    if items.iter().any(|i| matches!(i, Item::Error)) {
        return None;
    }
    Some(DateFormat { raw: fmt.into(), items: Some(items) })
}

pub fn set_date_format(fmt: &str) -> bool {
    let Some(f) = parse_date_format(fmt) else { return false };
    *date_format_cell().write().unwrap() = Arc::new(f);
    true
}

pub fn format_time(secs: i64) -> String {
    let f = date_format_cell().read().unwrap().clone();
    format_parsed(secs, &f)
}

pub fn format_time_with(secs: i64, fmt: &str) -> String {
    parse_date_format(fmt).map(|f| format_parsed(secs, &f)).unwrap_or_default()
}

fn format_parsed(secs: i64, f: &DateFormat) -> String {
    let Some(t) = Local.timestamp_opt(secs, 0).single() else {
        return String::new();
    };
    if let Some(items) = &f.items {
        return t.format_with_items(items.iter()).to_string();
    }
    let today = Local::now().date_naive();
    let d = t.date_naive();
    if d == today {
        format!("Today at {}", t.format("%H:%M"))
    } else if today.pred_opt() == Some(d) {
        format!("Yesterday at {}", t.format("%H:%M"))
    } else if (today - d).num_days() < 7 && d < today {
        t.format("%A at %H:%M").to_string()
    } else {
        t.format("%Y-%m-%d %H:%M").to_string()
    }
}

/// Long form for Properties/conflicts: weekday + the chosen pattern with seconds.
pub fn format_time_full(secs: i64) -> String {
    let fmt = date_format_cell().read().unwrap().raw.clone();
    let long = if fmt == RELATIVE_DATE_FORMAT {
        "%A, %d %B %Y, %H:%M:%S".to_string()
    } else if fmt.contains("%M") && !fmt.contains("%S") && !fmt.contains("%T") {
        format!("%A, {}", fmt.replacen("%M", "%M:%S", 1))
    } else {
        format!("%A, {fmt}")
    };
    format_time_with(secs, &long)
}

pub fn perm_string(mode: u32, is_dir: bool, is_link: bool) -> String {
    let kind = if is_link {
        'l'
    } else if is_dir {
        'd'
    } else {
        '-'
    };
    let bits = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];
    std::iter::once(kind)
        .chain(bits.iter().map(|(b, c)| if mode & b != 0 { *c } else { '-' }))
        .collect()
}

/// Name for a uid/gid via the reentrant NSS calls (getpwuid/getgrgid share
/// static buffers and race between threads). Cached; the first lookup may
/// block on NSS (LDAP/sssd), so the folder loader warms it off-thread.
fn id_name(id: u32, group: bool) -> String {
    static USERS: OnceLock<Mutex<HashMap<u32, String>>> = OnceLock::new();
    static GROUPS: OnceLock<Mutex<HashMap<u32, String>>> = OnceLock::new();
    let cache = if group { &GROUPS } else { &USERS }.get_or_init(Default::default);
    if let Some(n) = cache.lock().unwrap().get(&id) {
        return n.clone();
    }
    let mut buf = vec![0 as libc::c_char; 1024];
    let name = loop {
        // SAFETY: the out structs and buffer are valid for the call, and the
        // returned name points into `buf`, which outlives its use below.
        let (rc, found, name_ptr) = unsafe {
            if group {
                let mut gr: libc::group = std::mem::zeroed();
                let mut res: *mut libc::group = std::ptr::null_mut();
                let rc = libc::getgrgid_r(id, &mut gr, buf.as_mut_ptr(), buf.len(), &mut res);
                (rc, res.is_null(), gr.gr_name)
            } else {
                let mut pw: libc::passwd = std::mem::zeroed();
                let mut res: *mut libc::passwd = std::ptr::null_mut();
                let rc = libc::getpwuid_r(id, &mut pw, buf.as_mut_ptr(), buf.len(), &mut res);
                (rc, res.is_null(), pw.pw_name)
            }
        };
        if rc == libc::ERANGE && buf.len() < 1 << 20 {
            buf.resize(buf.len() * 2, 0);
            continue;
        }
        if rc != 0 || found || name_ptr.is_null() {
            break id.to_string();
        }
        // SAFETY: on success name_ptr is a NUL-terminated string inside buf.
        break unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy().into_owned();
    };
    cache.lock().unwrap().insert(id, name.clone());
    name
}

pub fn user_name(uid: u32) -> String {
    id_name(uid, false)
}

pub fn group_name(gid: u32) -> String {
    id_name(gid, true)
}

pub fn free_space(path: &Path) -> Option<(u64, u64)> {
    let c = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: c is a valid NUL-terminated path and st is a valid out pointer.
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    let frsize = st.f_frsize as u64;
    Some((st.f_bavail as u64 * frsize, st.f_blocks as u64 * frsize))
}

pub fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o7777))
}

pub fn expand_tilde(p: &str) -> PathBuf {
    let p = p.trim();
    let p = p.strip_prefix("file://").unwrap_or(p);
    if p == "~" {
        return home();
    }
    if let Some(rest) = p.strip_prefix("~/") {
        return home().join(rest);
    }
    PathBuf::from(percent_decode(p))
}

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

pub fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3])
                .ok()
                .and_then(|h| u8::from_str_radix(h, 16).ok())
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Percent-encodes every byte except ASCII alphanumerics and `keep`.
pub fn percent_encode(s: &str, keep: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || keep.contains(&b) {
            out.push(b as char);
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

pub fn path_to_uri(p: &str) -> String {
    format!("file://{}", percent_encode(p, b"/-_.~!$&'()*+,;=:@"))
}

pub fn uri_to_path(u: &str) -> String {
    match u.strip_prefix("file://") {
        Some(rest) => percent_decode(rest),
        None => u.to_string(),
    }
}

/// "name.ext" -> "name (copy 1).ext", Thunar-style, picking the first free slot.
pub fn unique_name(dir: &Path, name: &str, label: &str) -> PathBuf {
    let (stem, ext) = split_ext(name);
    (1..)
        .map(|i| {
            let n = if label.is_empty() {
                format!("{stem} ({i}){ext}")
            } else {
                format!("{stem} ({label} {i}){ext}")
            };
            dir.join(n)
        })
        .find(|p| fs::symlink_metadata(p).is_err())
        .unwrap()
}

pub fn split_ext(name: &str) -> (&str, &str) {
    let multi = [".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst"];
    for m in multi {
        let i = name.len().saturating_sub(m.len());
        if i > 0 && name.get(i..).is_some_and(|t| t.eq_ignore_ascii_case(m)) {
            return (&name[..i], &name[i..]);
        }
    }
    match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_order() {
        let mut v = vec!["file10", "file2", "File1", ".hidden", "file02b", "a"];
        v.sort_by(|a, b| natural_cmp(a, b).then_with(|| a.cmp(b)));
        assert_eq!(v, vec!["a", "File1", "file2", "file02b", "file10", ".hidden"]);
    }

    #[test]
    fn sizes() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(1), "1 byte");
        assert_eq!(format_size(1500), "1.5 kB");
        assert_eq!(format_size(3_200_000_000), "3.2 GB");
    }

    #[test]
    fn date_formats() {
        let valid = |f: &str| parse_date_format(f).is_some();
        assert!(valid("%d/%m/%Y %H:%M"));
        assert!(valid("relative"));
        assert!(!valid("%Q"));
        assert!(!valid("  "));
        let t = chrono::Local.with_ymd_and_hms(2026, 3, 7, 9, 5, 0).unwrap().timestamp();
        assert_eq!(format_time_with(t, "%d/%m/%Y %H:%M"), "07/03/2026 09:05");
        assert_eq!(format_time_with(t, "%Q"), "");
    }

    #[test]
    fn permissions() {
        assert_eq!(perm_string(0o755, true, false), "drwxr-xr-x");
        assert_eq!(perm_string(0o640, false, false), "-rw-r-----");
        assert_eq!(perm_string(0o777, false, true), "lrwxrwxrwx");
    }

    #[test]
    fn uris_round_trip() {
        let p = "/tmp/a dir/ünïcode #1%.txt";
        let u = path_to_uri(p);
        assert!(u.starts_with("file:///tmp/a%20dir/"));
        assert_eq!(uri_to_path(&u), p);
        assert_eq!(percent_decode("%zz%41"), "%zzA");
        assert_eq!(percent_decode("trail%4"), "trail%4");
    }

    #[test]
    fn extensions() {
        assert_eq!(split_ext("photo.jpg"), ("photo", ".jpg"));
        assert_eq!(split_ext("backup.tar.gz"), ("backup", ".tar.gz"));
        assert_eq!(split_ext(".bashrc"), (".bashrc", ""));
        assert_eq!(split_ext("Makefile"), ("Makefile", ""));
    }

    #[test]
    fn unique_names() {
        let dir = std::env::temp_dir().join(format!("tla-fe-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), "").unwrap();
        assert_eq!(unique_name(&dir, "a.txt", "copy"), dir.join("a (copy 1).txt"));
        fs::write(dir.join("a (copy 1).txt"), "").unwrap();
        assert_eq!(unique_name(&dir, "a.txt", "copy"), dir.join("a (copy 2).txt"));
        assert_eq!(unique_name(&dir, "a.txt", ""), dir.join("a (1).txt"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn name_sort_groups_folders_then_hidden() {
        let mk = |name: &str, dir: bool| Entry {
            name: name.into(),
            is_dir: dir,
            hidden: name.starts_with('.'),
            ..Default::default()
        };
        let mut v = vec![
            mk("Music", true), mk(".config", true), mk("notes.txt", false), mk(".zshrc", false),
            mk("Documents", true), mk(".bashrc", false), mk("photo.jpg", false), mk(".cache", true),
        ];
        let names = |v: &[Entry]| v.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
        // Folders First off (as in the saved settings) must not change it.
        let mut spec = SortSpec { key: SortKey::Name, descending: false, folders_first: false };
        v.sort_by(|a, b| compare(a, b, spec));
        assert_eq!(
            names(&v),
            [".cache", ".config", "Documents", "Music", ".bashrc", ".zshrc", "notes.txt", "photo.jpg"]
        );
        spec.descending = true;
        v.sort_by(|a, b| compare(a, b, spec));
        assert_eq!(
            names(&v),
            [".config", ".cache", "Music", "Documents", ".zshrc", ".bashrc", "photo.jpg", "notes.txt"]
        );
    }

    #[test]
    fn metadata_sort_ignores_folders_and_keeps_name_tiebreak() {
        let mk = |name: &str, dir: bool, size: u64| Entry { name: name.into(), is_dir: dir, size, ..Default::default() };
        let mut v = vec![mk("b", false, 5), mk("dir", true, 0), mk("a", false, 5), mk("c", false, 50)];
        let spec = SortSpec { key: SortKey::Size, descending: true, folders_first: false };
        v.sort_by(|a, b| compare(a, b, spec));
        let names: Vec<_> = v.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["c", "a", "b", "dir"]);
    }

    #[test]
    fn sorting_folders_first() {
        let mk = |name: &str, dir: bool, size: u64| Entry {
            name: name.into(),
            is_dir: dir,
            size,
            ..Default::default()
        };
        let mut v = vec![mk("b.txt", false, 5), mk("zdir", true, 0), mk("a.txt", false, 50)];
        let spec = SortSpec { key: SortKey::Size, descending: false, folders_first: true };
        v.sort_by(|a, b| compare(a, b, spec));
        let names: Vec<_> = v.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["zdir", "b.txt", "a.txt"]);
    }
}
