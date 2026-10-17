#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, home_path, cxx_name = "homePath")]
        #[qproperty(QString, start_path, cxx_name = "startPath")]
        #[qproperty(QString, theme_json, cxx_name = "themeJson")]
        #[qproperty(QString, bookmarks_json, cxx_name = "bookmarksJson")]
        #[qproperty(QString, devices_json, cxx_name = "devicesJson")]
        #[qproperty(QString, places_json, cxx_name = "placesJson")]
        #[qproperty(i32, clipboard_serial, cxx_name = "clipboardSerial")]
        type Sys = super::SysRust;

        #[qsignal]
        #[cxx_name = "sizeComputed"]
        fn size_computed(self: Pin<&mut Sys>, path: QString, text: QString);

        /// An open/launch failed after it was started (e.g. no handler).
        #[qsignal]
        #[cxx_name = "launchFailed"]
        fn launch_failed(self: Pin<&mut Sys>, message: QString);

        #[qsignal]
        #[cxx_name = "infoReady"]
        fn info_ready(self: Pin<&mut Sys>, path: QString, json: QString);

        #[qsignal]
        #[cxx_name = "modeApplied"]
        fn mode_applied(self: Pin<&mut Sys>, errors: QString);

        #[qsignal]
        #[cxx_name = "mountFinished"]
        fn mount_finished(self: Pin<&mut Sys>, device: QString, mount_point: QString, error: QString);

        #[qinvokable]
        fn init(self: Pin<&mut Sys>);

        #[qinvokable]
        #[cxx_name = "openPaths"]
        fn open_paths(self: &Sys, paths: &QStringList) -> QString;

        #[qinvokable]
        #[cxx_name = "openWith"]
        fn open_with(self: &Sys, app_id: &QString, paths: &QStringList) -> QString;

        #[qinvokable]
        #[cxx_name = "openWithCommand"]
        fn open_with_command(self: &Sys, command: &QString, paths: &QStringList) -> QString;

        #[qinvokable]
        #[cxx_name = "appsFor"]
        fn apps_for(self: &Sys, mime: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "allApps"]
        fn all_apps(self: &Sys) -> QString;

        #[qinvokable]
        #[cxx_name = "setDefaultApp"]
        fn set_default_app(self: &Sys, mime: &QString, app_id: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "openTerminal"]
        fn open_terminal(self: &Sys, dir: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "newWindow"]
        fn new_window(self: &Sys, dir: &QString);

        #[qinvokable]
        #[cxx_name = "setClipboard"]
        fn set_clipboard(self: Pin<&mut Sys>, paths: &QStringList, cut: bool);

        #[qinvokable]
        #[cxx_name = "clipboardPaths"]
        fn clipboard_paths(self: &Sys) -> QStringList;

        #[qinvokable]
        #[cxx_name = "clipboardIsCut"]
        fn clipboard_is_cut(self: &Sys) -> bool;

        #[qinvokable]
        #[cxx_name = "addBookmark"]
        fn add_bookmark(self: Pin<&mut Sys>, path: &QString);

        #[qinvokable]
        #[cxx_name = "removeBookmark"]
        fn remove_bookmark(self: Pin<&mut Sys>, path: &QString);

        #[qinvokable]
        #[cxx_name = "renameBookmark"]
        fn rename_bookmark(self: Pin<&mut Sys>, path: &QString, label: &QString);

        #[qinvokable]
        #[cxx_name = "moveBookmark"]
        fn move_bookmark(self: Pin<&mut Sys>, from: i32, to: i32);

        #[qinvokable]
        #[cxx_name = "refreshDevices"]
        fn refresh_devices(self: Pin<&mut Sys>);

        #[qinvokable]
        fn mount(self: Pin<&mut Sys>, device: &QString);

        #[qinvokable]
        fn unmount(self: Pin<&mut Sys>, device: &QString, eject: bool);

        #[qinvokable]
        fn normalize(self: &Sys, path: &QString) -> QString;

        #[qinvokable]
        fn parent(self: &Sys, path: &QString) -> QString;

        #[qinvokable]
        fn basename(self: &Sys, path: &QString) -> QString;

        #[qinvokable]
        fn join(self: &Sys, dir: &QString, name: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "isDir"]
        fn is_dir(self: &Sys, path: &QString) -> bool;

        #[qinvokable]
        fn exists(self: &Sys, path: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "toUri"]
        fn to_uri(self: &Sys, path: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "fromUri"]
        fn from_uri(self: &Sys, uri: &QString) -> QString;

        #[qinvokable]
        fn complete(self: &Sys, partial: &QString) -> QStringList;

        #[qinvokable]
        fn templates(self: &Sys) -> QString;

        /// Gathers details off-thread; answered by infoReady.
        #[qinvokable]
        #[cxx_name = "requestInfo"]
        fn request_info(self: &Sys, path: &QString);

        #[qinvokable]
        #[cxx_name = "computeSize"]
        fn compute_size(self: Pin<&mut Sys>, paths: &QStringList);

        /// Applies permissions off-thread; answered by modeApplied.
        #[qinvokable]
        #[cxx_name = "applyMode"]
        fn apply_mode(self: &Sys, paths: &QStringList, mode: i32, recursive: bool);

        /// Applies a strftime pattern (or "relative"); false if it is malformed.
        #[qinvokable]
        #[cxx_name = "setDateFormat"]
        fn set_date_format(self: &Sys, format: &QString) -> bool;

        /// Formats "now" with the pattern, or "" if the pattern is malformed.
        #[qinvokable]
        #[cxx_name = "previewDate"]
        fn preview_date(self: &Sys, format: &QString) -> QString;

        #[qsignal]
        #[cxx_name = "smbReady"]
        fn smb_ready(self: Pin<&mut Sys>, uri: QString, local_path: QString);

        #[qsignal]
        #[cxx_name = "smbAuthRequired"]
        fn smb_auth_required(self: Pin<&mut Sys>, uri: QString, host: QString, message: QString);

        #[qsignal]
        #[cxx_name = "smbError"]
        fn smb_error(self: Pin<&mut Sys>, uri: QString, message: QString);

        /// Mounts (if needed) the share in an smb:// URI off-thread, then emits
        /// smbReady with the local gvfs path, smbAuthRequired or smbError.
        #[qinvokable]
        #[cxx_name = "smbOpen"]
        fn smb_open(self: Pin<&mut Sys>, uri: &QString);

        #[qinvokable]
        #[cxx_name = "setSmbCredentials"]
        fn set_smb_credentials(
            self: &Sys,
            host: &QString,
            user: &QString,
            domain: &QString,
            password: &QString,
            anonymous: bool,
            remember: bool,
        );

        /// Exit hook: drops memory-only credentials and unmounts the shares
        /// they opened.
        #[qinvokable]
        fn shutdown(self: &Sys);

        /// smb://host/share for a local gvfs path ("" otherwise) plus its root.
        #[qinvokable]
        #[cxx_name = "smbInfo"]
        fn smb_info(self: &Sys, path: &QString) -> QString;

        /// The path as the user should see it (gvfs mounts as smb:// URIs).
        #[qinvokable]
        #[cxx_name = "displayPath"]
        fn display_path(self: &Sys, path: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "smbHasShare"]
        fn smb_has_share(self: &Sys, uri: &QString) -> bool;

        /// Whether `path` lies inside a gvfs mount (SMB, SFTP, ...). Those
        /// have no trash: "moving to trash" would download the file through FUSE.
        #[qinvokable]
        #[cxx_name = "onShare"]
        fn on_share(self: &Sys, path: &QString) -> bool;

        /// Host of an SMB share among `paths`/`dest` the app holds no
        /// credentials for ("" if none): direct transfers need them.
        #[qinvokable]
        #[cxx_name = "smbCredsNeeded"]
        fn smb_creds_needed(self: &Sys, paths: &QStringList, dest: &QString) -> QString;

        /// file:// URL of settings.conf, for the QML Settings element.
        #[qinvokable]
        #[cxx_name = "settingsUrl"]
        fn settings_url(self: &Sys) -> QString;

        /// Saved per-folder view settings (JSON object, "{}" if none).
        #[qinvokable]
        #[cxx_name = "folderView"]
        fn folder_view(self: &Sys, path: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "setFolderView"]
        fn set_folder_view(self: &Sys, path: &QString, json: &QString);

        #[qinvokable]
        #[cxx_name = "sameDevice"]
        fn same_device(self: &Sys, a: &QString, b: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "suggestName"]
        fn suggest_name(self: &Sys, dir: &QString, base: &QString) -> QString;
    }

    impl cxx_qt::Threading for Sys {}
}

use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QString, QStringList};
use notify::{RecursiveMode, Watcher};
use serde_json::json;

use crate::apps;
use crate::bridge::native::ffi as native;
use crate::fsutil;
use crate::icons;
use crate::config;
use crate::smb;

#[derive(Default)]
pub struct SysRust {
    home_path: QString,
    start_path: QString,
    theme_json: QString,
    bookmarks_json: QString,
    devices_json: QString,
    places_json: QString,
    clipboard_serial: i32,

    watchers: Vec<notify::RecommendedWatcher>,
    bookmarks_gen: Arc<AtomicU64>,
    /// When we last wrote the bookmarks file, so our own write isn't re-read.
    bookmarks_written: Option<Instant>,
    size_gen: Arc<AtomicU64>,
}

fn theme_file() -> PathBuf {
    config::config_dir().join("colors.json")
}

fn bookmarks_file() -> PathBuf {
    config::bookmarks_file()
}

fn qs(s: impl AsRef<str>) -> QString {
    QString::from(s.as_ref())
}

fn list(l: &QStringList) -> Vec<String> {
    l.iter().map(|s| fsutil::uri_to_path(&s.to_string())).collect()
}

fn err(r: Result<(), String>) -> QString {
    match r {
        Ok(()) => QString::default(),
        Err(e) => qs(e),
    }
}

struct Bookmark {
    path: String,
    label: String,
}

fn read_bookmarks() -> Vec<Bookmark> {
    let text = fs::read_to_string(bookmarks_file()).unwrap_or_default();
    // Local bookmarks become paths; smb:// (and other remote URIs) stay URIs.
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let (uri, label) = l.split_once(' ').unwrap_or((l, ""));
            let path = if uri.starts_with("file://") {
                fsutil::uri_to_path(uri)
            } else if uri.starts_with("smb://") {
                smb::display_uri(uri)
            } else {
                uri.to_string()
            };
            Bookmark { path, label: label.trim().to_string() }
        })
        .collect()
}

fn write_bookmarks(b: &[Bookmark]) {
    let out: Vec<String> = b
        .iter()
        .map(|b| {
            // The file format is "URI label", so URIs must not contain raw spaces.
            let uri = if b.path.starts_with('/') {
                fsutil::path_to_uri(&b.path)
            } else if b.path.starts_with("smb://") {
                smb::encode_uri(&b.path)
            } else {
                b.path.clone()
            };
            if b.label.is_empty() {
                uri
            } else {
                format!("{uri} {}", b.label)
            }
        })
        .collect();
    let file = bookmarks_file();
    if let Some(p) = file.parent() {
        let _ = fs::create_dir_all(p);
    }
    let tmp = file.with_extension(format!("{}.tmp", std::process::id()));
    if fs::write(&tmp, out.join("\n") + "\n").is_ok() {
        let _ = fs::rename(&tmp, &file);
    }
}

/// With `check` false nothing touches the filesystem, so it is safe on the GUI
/// thread; a stale network mount can take seconds to stat.
fn bookmarks_json(check: bool) -> String {
    let v: Vec<_> = read_bookmarks()
        .into_iter()
        .map(|b| {
            let remote = !b.path.starts_with('/');
            let name = if !b.label.is_empty() {
                b.label.clone()
            } else if let Some(u) = smb::parse(&b.path) {
                match &u.share {
                    Some(share) => format!("{share} on {}", u.host),
                    None => u.host.clone(),
                }
            } else {
                Path::new(&b.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| b.path.clone())
            };
            json!({
                "path": b.path,
                "name": name,
                "glyph": if remote { icons::FOLDER_NETWORK } else { icons::for_folder_path(&b.path) },
                "remote": remote,
                "exists": remote || !check || Path::new(&b.path).exists(),
            })
        })
        .collect();
    serde_json::Value::Array(v).to_string()
}

fn places_json() -> String {
    let mut v = vec![json!({
        "path": fsutil::home().to_string_lossy(),
        "name": "Home",
        "glyph": icons::FOLDER_HOME,
    })];
    let mut add = |p: Option<PathBuf>, name: &str| {
        if let Some(p) = p.filter(|p| p.is_dir() && *p != fsutil::home()) {
            v.push(json!({
                "path": p.to_string_lossy(),
                "name": name,
                "glyph": icons::for_folder_path(&p.to_string_lossy()),
            }));
        }
    };
    add(dirs::desktop_dir(), "Desktop");
    v.push(json!({"path": "trash:///", "name": "Trash", "glyph": "\u{F0A79}", "trash": true}));
    v.push(json!({"path": "/", "name": "File System", "glyph": "\u{F02CA}"}));
    serde_json::Value::Array(v).to_string()
}

#[derive(serde::Deserialize)]
struct Lsblk {
    blockdevices: Vec<BlockDev>,
}

#[derive(serde::Deserialize, Clone)]
struct BlockDev {
    path: String,
    label: Option<String>,
    size: Option<u64>,
    fstype: Option<String>,
    mountpoint: Option<String>,
    #[serde(rename = "type")]
    kind: String,
    rm: Option<bool>,
    hotplug: Option<bool>,
    model: Option<String>,
    #[serde(default)]
    children: Vec<BlockDev>,
}

const SYSTEM_MOUNTS: &[&str] = &["/", "/boot", "/boot/efi", "/efi", "/home", "/var", "/usr", "/nix", "[SWAP]"];

fn devices_json() -> String {
    let out = Command::new("lsblk")
        .args(["-J", "-b", "-o", "NAME,PATH,LABEL,SIZE,FSTYPE,MOUNTPOINT,TYPE,RM,HOTPLUG,MODEL"])
        .output();
    let mut v = Vec::new();
    if let Ok(out) = out {
        if let Ok(parsed) = serde_json::from_slice::<Lsblk>(&out.stdout) {
            for disk in &parsed.blockdevices {
                collect_devices(disk, disk, &mut v);
            }
        }
    }
    // gvfs network/MTP mounts.
    let gvfs = PathBuf::from(format!("/run/user/{}/gvfs", unsafe { libc::getuid() }));
    if let Ok(rd) = fs::read_dir(&gvfs) {
        for d in rd.flatten() {
            let name = d.file_name().to_string_lossy().into_owned();
            v.push(json!({
                "device": "",
                "name": gvfs_label(&name),
                "mountPoint": d.path().to_string_lossy(),
                "mounted": true,
                "removable": false,
                "network": true,
                "glyph": "\u{F048D}",
                "sizeText": "",
            }));
        }
    }
    serde_json::Value::Array(v).to_string()
}

/// "smb-share:server=nas,share=media,user=me" -> "media on nas".
fn gvfs_label(raw: &str) -> String {
    let (scheme, rest) = raw.split_once(':').unwrap_or(("", raw));
    let kv: std::collections::HashMap<&str, &str> = rest
        .split(',')
        .filter_map(|p| p.split_once('='))
        .collect();
    let host = kv.get("server").or_else(|| kv.get("host")).copied().unwrap_or("");
    match (kv.get("share"), host) {
        (Some(share), h) if !h.is_empty() => format!("{share} on {h}"),
        (_, h) if !h.is_empty() => format!("{h} ({})", scheme.trim_end_matches("-share")),
        _ => raw.to_string(),
    }
}

fn collect_devices(dev: &BlockDev, disk: &BlockDev, out: &mut Vec<serde_json::Value>) {
    for c in &dev.children {
        collect_devices(c, disk, out);
    }
    let Some(fstype) = dev.fstype.as_deref().filter(|f| !f.is_empty()) else { return };
    if fstype == "swap" || fstype == "crypto_LUKS" || fstype == "LVM2_member" || fstype.ends_with("_member") {
        return;
    }
    let mp = dev.mountpoint.clone().unwrap_or_default();
    if SYSTEM_MOUNTS.contains(&mp.as_str()) {
        return;
    }
    let removable = disk.rm.unwrap_or(false) || disk.hotplug.unwrap_or(false);
    let size = dev.size.unwrap_or(0);
    let name = dev
        .label
        .clone()
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| format!("{} Volume", fsutil::format_size(size)));
    let glyph = if dev.kind == "rom" {
        "\u{F05EE}"
    } else if disk.rm.unwrap_or(false) || disk.hotplug.unwrap_or(false) {
        "\u{F129E}"
    } else {
        "\u{F02CA}"
    };
    let used = if mp.is_empty() {
        None
    } else {
        fsutil::free_space(Path::new(&mp)).map(|(free, total)| (total.saturating_sub(free), total))
    };
    out.push(json!({
        "device": dev.path,
        "disk": disk.path,
        "name": name,
        "model": disk.model.clone().unwrap_or_default().trim(),
        "fstype": fstype,
        "mountPoint": mp,
        "mounted": !mp.is_empty(),
        "removable": removable,
        "network": false,
        "glyph": glyph,
        "sizeText": fsutil::format_size(size),
        "usage": used.map(|(u, t)| if t > 0 { u as f64 / t as f64 } else { 0.0 }).unwrap_or(-1.0),
    }));
}

fn read_theme() -> String {
    fs::read_to_string(theme_file()).unwrap_or_default()
}

impl qobject::Sys {
    pub fn init(mut self: Pin<&mut Self>) {
        let home = fsutil::home().to_string_lossy().into_owned();
        self.as_mut().set_home_path(qs(&home));
        let start = std::env::args()
            .nth(1)
            .map(|a| {
                let p = fsutil::expand_tilde(&a);
                fs::canonicalize(&p).unwrap_or(p).to_string_lossy().into_owned()
            })
            .unwrap_or(home);
        self.as_mut().set_start_path(qs(start));
        self.as_mut().set_theme_json(qs(read_theme()));
        self.as_mut().refresh_bookmarks();
        self.as_mut().set_places_json(qs(places_json()));
        self.as_mut().refresh_devices();
        std::thread::spawn(apps::warm);
        self.as_mut().watch_file(theme_file(), |m| {
            let t = read_theme();
            if !t.trim().is_empty() {
                m.set_theme_json(qs(t));
            }
        });
        self.as_mut().watch_file(bookmarks_file(), |mut m| {
            let own = m.bookmarks_written.is_some_and(|t| t.elapsed() < Duration::from_secs(1));
            m.as_mut().rust_mut().bookmarks_written = None;
            if !own {
                m.refresh_bookmarks();
            }
        });
    }

    /// Watches the parent directory so atomic replace-by-rename (as matugen
    /// and GTK do) is caught, then filters events to the one file.
    fn watch_file(
        mut self: Pin<&mut Self>,
        file: PathBuf,
        on_change: fn(Pin<&mut qobject::Sys>),
    ) {
        let Some(dir) = file.parent().map(Path::to_path_buf) else { return };
        let _ = fs::create_dir_all(&dir);
        let qt = self.qt_thread();
        let pending = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let target = file.clone();
        let w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(ev) = res else { return };
            if !ev.paths.iter().any(|p| *p == target) || matches!(ev.kind, notify::EventKind::Access(_)) {
                return;
            }
            if pending.swap(true, std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            let qt = qt.clone();
            let pending = pending.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(150));
                pending.store(false, std::sync::atomic::Ordering::SeqCst);
                let _ = qt.queue(move |m| on_change(m));
            });
        });
        if let Ok(mut w) = w {
            if w.watch(&dir, RecursiveMode::NonRecursive).is_ok() {
                self.as_mut().rust_mut().watchers.push(w);
            }
        }
    }

    /// Reaps a launched child off-thread; failures surface as launchFailed.
    fn reap(&self, child: Child) {
        let qt = self.qt_thread();
        apps::reap(child, move |msg| {
            let _ = qt.queue(move |m| m.launch_failed(qs(msg)));
        });
    }

    fn reap_all(&self, res: Result<Vec<Child>, String>) -> QString {
        match res {
            Ok(children) => {
                children.into_iter().for_each(|c| self.reap(c));
                QString::default()
            }
            Err(e) => qs(e),
        }
    }

    pub fn open_paths(&self, paths: &QStringList) -> QString {
        self.reap_all(apps::open_default(&list(paths)).map(|c| vec![c]))
    }

    pub fn open_with(&self, app_id: &QString, paths: &QStringList) -> QString {
        self.reap_all(apps::launch(&app_id.to_string(), &list(paths)).map(|c| vec![c]))
    }

    pub fn open_with_command(&self, command: &QString, paths: &QStringList) -> QString {
        self.reap_all(apps::run_custom(&command.to_string(), &list(paths)))
    }

    pub fn apps_for(&self, mime: &QString) -> QString {
        let (default, ids) = apps::for_mime(&mime.to_string());
        let mut v = Vec::new();
        let mut push = |id: &str, is_default: bool| {
            if let Some(a) = apps::lookup(id) {
                v.push(json!({"id": a.id, "name": a.name, "icon": a.icon, "default": is_default}));
            }
        };
        if let Some(d) = &default {
            push(d, true);
        }
        for id in ids.iter().filter(|i| Some(*i) != default.as_ref()) {
            push(id, false);
        }
        qs(serde_json::Value::Array(v).to_string())
    }

    pub fn all_apps(&self) -> QString {
        qs(serde_json::to_string(&apps::all_apps()).unwrap_or_default())
    }

    pub fn set_default_app(&self, mime: &QString, app_id: &QString) -> QString {
        err(apps::set_default(&mime.to_string(), &app_id.to_string()))
    }

    pub fn open_terminal(&self, dir: &QString) -> QString {
        // $TERMINAL may carry arguments ("foot -a x").
        let term = apps::terminal();
        let mut words = term.split_whitespace();
        let Some(prog) = words.next() else { return qs("No terminal configured") };
        let res = Command::new(prog)
            .args(words)
            .current_dir(dir.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|c| vec![c])
            .map_err(|e| e.to_string());
        self.reap_all(res)
    }

    pub fn new_window(&self, dir: &QString) {
        if let Ok(exe) = std::env::current_exe() {
            if let Ok(c) = Command::new(exe).arg(dir.to_string()).spawn() {
                self.reap(c);
            }
        }
    }

    pub fn set_clipboard(mut self: Pin<&mut Self>, paths: &QStringList, cut: bool) {
        let uris: Vec<String> = list(paths).iter().map(|p| fsutil::path_to_uri(p)).collect();
        native::set_clipboard_uris(&uris, cut);
        let n = self.clipboard_serial + 1;
        self.as_mut().set_clipboard_serial(n);
    }

    pub fn clipboard_paths(&self) -> QStringList {
        native::clipboard_uris()
            .iter()
            .filter(|u| u.starts_with("file://"))
            .map(|u| qs(fsutil::uri_to_path(u)))
            .collect()
    }

    pub fn clipboard_is_cut(&self) -> bool {
        native::clipboard_is_cut()
    }

    /// Publish the list immediately, then fill in reachability off-thread.
    fn refresh_bookmarks(mut self: Pin<&mut Self>) {
        self.as_mut().set_bookmarks_json(qs(bookmarks_json(false)));
        let gen = self.bookmarks_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let gen_ref = self.bookmarks_gen.clone();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let j = bookmarks_json(true);
            if gen_ref.load(Ordering::SeqCst) != gen {
                return;
            }
            let _ = qt.queue(move |mut m| {
                if m.bookmarks_gen.load(Ordering::SeqCst) == gen {
                    m.as_mut().set_bookmarks_json(qs(j));
                }
            });
        });
    }

    fn save_bookmarks(mut self: Pin<&mut Self>, b: &[Bookmark]) {
        self.as_mut().rust_mut().bookmarks_written = Some(Instant::now());
        write_bookmarks(b);
    }

    pub fn add_bookmark(mut self: Pin<&mut Self>, path: &QString) {
        // Folders inside a gvfs SMB mount are stored as smb:// so the
        // bookmark survives the share being remounted elsewhere.
        let p = self.display_path(path).to_string();
        let mut b = read_bookmarks();
        if !b.iter().any(|x| x.path == p) {
            b.push(Bookmark { path: p, label: String::new() });
            self.as_mut().save_bookmarks(&b);
        }
        self.as_mut().refresh_bookmarks();
    }

    pub fn remove_bookmark(mut self: Pin<&mut Self>, path: &QString) {
        let p = path.to_string();
        let b: Vec<Bookmark> = read_bookmarks().into_iter().filter(|x| x.path != p).collect();
        self.as_mut().save_bookmarks(&b);
        self.as_mut().refresh_bookmarks();
    }

    pub fn rename_bookmark(mut self: Pin<&mut Self>, path: &QString, label: &QString) {
        let p = path.to_string();
        let mut b = read_bookmarks();
        if let Some(x) = b.iter_mut().find(|x| x.path == p) {
            x.label = label.to_string();
        }
        self.as_mut().save_bookmarks(&b);
        self.as_mut().refresh_bookmarks();
    }

    pub fn move_bookmark(mut self: Pin<&mut Self>, from: i32, to: i32) {
        let mut b = read_bookmarks();
        let (from, to) = (from as usize, to as usize);
        if from < b.len() && to < b.len() && from != to {
            let item = b.remove(from);
            b.insert(to, item);
            self.as_mut().save_bookmarks(&b);
        }
        self.as_mut().refresh_bookmarks();
    }

    pub fn refresh_devices(self: Pin<&mut Self>) {
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let j = devices_json();
            let _ = qt.queue(move |mut m| {
                if m.devices_json.to_string() != j {
                    m.as_mut().set_devices_json(qs(j));
                }
            });
        });
    }

    pub fn mount(self: Pin<&mut Self>, device: &QString) {
        let dev = device.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let out = Command::new("udisksctl").args(["mount", "-b", &dev]).output();
            let (mp, e) = match out {
                Ok(o) if o.status.success() => {
                    let s = String::from_utf8_lossy(&o.stdout);
                    let mp = s
                        .rsplit_once(" at ")
                        .map(|(_, p)| p.trim().trim_end_matches('.').to_string())
                        .unwrap_or_default();
                    (mp, String::new())
                }
                Ok(o) => (String::new(), String::from_utf8_lossy(&o.stderr).trim().to_string()),
                Err(e) => (String::new(), e.to_string()),
            };
            let j = devices_json();
            let _ = qt.queue(move |mut m| {
                m.as_mut().set_devices_json(qs(j));
                m.mount_finished(qs(&dev), qs(&mp), qs(&e));
            });
        });
    }

    pub fn unmount(self: Pin<&mut Self>, device: &QString, eject: bool) {
        let dev = device.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let mut e = String::new();
            let res = if dev.starts_with("/dev/") {
                Command::new("udisksctl").args(["unmount", "-b", &dev]).output()
            } else {
                Command::new("gio").args(["mount", "-u", &dev]).output()
            };
            match res {
                Ok(o) if !o.status.success() => e = String::from_utf8_lossy(&o.stderr).trim().to_string(),
                Err(x) => e = x.to_string(),
                _ => {
                    if eject && dev.starts_with("/dev/") {
                        let disk = parent_disk(&dev);
                        let _ = Command::new("udisksctl").args(["power-off", "-b", &disk]).output();
                    }
                }
            }
            let j = devices_json();
            let _ = qt.queue(move |mut m| {
                m.as_mut().set_devices_json(qs(j));
                m.mount_finished(qs(&dev), QString::default(), qs(&e));
            });
        });
    }

    pub fn normalize(&self, path: &QString) -> QString {
        let s = path.to_string();
        if s.starts_with("trash:") {
            return qs("trash:///");
        }
        if s.starts_with("network:") {
            return qs(smb::NETWORK_URI);
        }
        if s.starts_with("smb://") {
            return match smb::parse(&s) {
                Some(u) => match u.share {
                    Some(share) if u.rest.is_empty() => qs(format!("smb://{}/{share}", u.host)),
                    Some(share) => qs(format!("smb://{}/{share}/{}", u.host, u.rest)),
                    None => qs(format!("smb://{}/", u.host)),
                },
                None => qs(smb::NETWORK_URI),
            };
        }
        let p = fsutil::expand_tilde(&s);
        let mut out = PathBuf::new();
        for c in p.components() {
            match c {
                std::path::Component::ParentDir => {
                    out.pop();
                }
                std::path::Component::CurDir => {}
                c => out.push(c),
            }
        }
        if out.as_os_str().is_empty() {
            out.push("/");
        }
        qs(out.to_string_lossy())
    }

    pub fn parent(&self, path: &QString) -> QString {
        let s = path.to_string();
        if s == smb::NETWORK_URI {
            return qs(s);
        }
        if let Some(u) = smb::parse(&s) {
            return match (u.share, u.rest.is_empty()) {
                (None, _) => qs(smb::NETWORK_URI),
                (Some(_), true) => qs(format!("smb://{}/", u.host)),
                (Some(share), false) => {
                    let up = Path::new(&u.rest).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
                    qs(if up.is_empty() { format!("smb://{}/{share}", u.host) } else { format!("smb://{}/{share}/{up}", u.host) })
                }
            };
        }
        // The root of a gvfs SMB mount goes up to the server's share list.
        if let Some((uri, root)) = smb::uri_for_local(&s) {
            if s.trim_end_matches('/') == root {
                return qs(smb::parse(&uri).map(|u| format!("smb://{}/", u.host)).unwrap_or(uri));
            }
        }
        Path::new(&s)
            .parent()
            .map(|p| qs(p.to_string_lossy()))
            .unwrap_or_else(|| qs(s))
    }

    pub fn basename(&self, path: &QString) -> QString {
        let s = path.to_string();
        if s == "/" {
            return qs("/");
        }
        if s == smb::NETWORK_URI {
            return qs("Network");
        }
        if let Some(u) = smb::parse(&s) {
            let last = u.rest.rsplit('/').next().filter(|x| !x.is_empty()).map(str::to_string);
            return qs(last.or(u.share).unwrap_or(u.host));
        }
        if let Some((uri, root)) = smb::uri_for_local(&s) {
            if s.trim_end_matches('/') == root {
                return qs(smb::parse(&uri).and_then(|u| u.share).unwrap_or(uri));
            }
        }
        Path::new(&s)
            .file_name()
            .map(|p| qs(p.to_string_lossy()))
            .unwrap_or_else(|| qs(s))
    }

    pub fn join(&self, dir: &QString, name: &QString) -> QString {
        qs(Path::new(&dir.to_string()).join(name.to_string()).to_string_lossy())
    }

    pub fn is_dir(&self, path: &QString) -> bool {
        let s = path.to_string();
        if s.starts_with("smb://") || s.starts_with("network:") {
            return true;
        }
        Path::new(&path.to_string()).is_dir()
    }

    pub fn exists(&self, path: &QString) -> bool {
        fs::symlink_metadata(path.to_string()).is_ok()
    }

    pub fn to_uri(&self, path: &QString) -> QString {
        qs(fsutil::path_to_uri(&path.to_string()))
    }

    pub fn from_uri(&self, uri: &QString) -> QString {
        qs(fsutil::uri_to_path(&uri.to_string()))
    }

    pub fn complete(&self, partial: &QString) -> QStringList {
        let s = partial.to_string();
        let expanded = fsutil::expand_tilde(&s).to_string_lossy().into_owned();
        let (dir, prefix) = match expanded.rfind('/') {
            Some(i) => (&expanded[..=i], &expanded[i + 1..]),
            None => return QStringList::default(),
        };
        let Ok(rd) = fs::read_dir(dir) else { return QStringList::default() };
        let show_hidden = prefix.starts_with('.');
        // Name filter first: only matching symlinks need a stat.
        let mut v: Vec<String> = rd
            .flatten()
            .filter_map(|d| {
                let n = d.file_name().to_string_lossy().into_owned();
                if !n.starts_with(prefix) || (!show_hidden && n.starts_with('.')) {
                    return None;
                }
                let ft = d.file_type().ok()?;
                let is_dir = ft.is_dir() || (ft.is_symlink() && d.path().is_dir());
                is_dir.then(|| format!("{dir}{n}/"))
            })
            .collect();
        v.sort_by(|a, b| fsutil::natural_cmp(a, b));
        v.truncate(50);
        v.iter().map(qs).collect()
    }

    pub fn templates(&self) -> QString {
        let dir = dirs::template_dir().unwrap_or_else(|| fsutil::home().join("Templates"));
        let mut v: Vec<_> = fs::read_dir(&dir)
            .map(|rd| {
                rd.flatten()
                    .filter(|d| d.path().is_file())
                    .filter_map(|d| fsutil::stat_entry(&d.path(), false))
                    .collect()
            })
            .unwrap_or_default();
        v.sort_by(|a, b| fsutil::natural_cmp(&a.name, &b.name));
        let j: Vec<_> = v
            .iter()
            .map(|e| {
                json!({
                    "path": e.path,
                    "name": fsutil::split_ext(&e.name).0,
                    "fileName": e.name,
                    "glyph": e.glyph,
                    "category": e.category,
                })
            })
            .collect();
        qs(serde_json::Value::Array(j).to_string())
    }

    pub fn request_info(&self, path: &QString) {
        let p = path.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let j = info_json(Path::new(&p));
            let _ = qt.queue(move |m| m.info_ready(qs(p), qs(j)));
        });
    }

    pub fn compute_size(self: Pin<&mut Self>, paths: &QStringList) {
        let roots: Vec<PathBuf> = list(paths).into_iter().map(PathBuf::from).collect();
        let key = roots
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("\n");
        // A newer request (or a closed dialog re-opened) supersedes this walk.
        let gen = self.size_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let gen_ref = self.size_gen.clone();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let Some(text) = tree_size(roots, || gen_ref.load(Ordering::SeqCst) != gen) else { return };
            let _ = qt.queue(move |m| m.size_computed(qs(key), qs(text)));
        });
    }

    pub fn apply_mode(&self, paths: &QStringList, mode: i32, recursive: bool) {
        let roots: Vec<PathBuf> = list(paths).into_iter().map(PathBuf::from).collect();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let mut errors = Vec::new();
            for r in &roots {
                chmod_tree(r, mode as u32, true, recursive, &mut errors);
            }
            let text = errors.into_iter().take(5).collect::<Vec<_>>().join("\n");
            let _ = qt.queue(move |m| m.mode_applied(qs(text)));
        });
    }

    pub fn set_date_format(&self, format: &QString) -> bool {
        fsutil::set_date_format(&format.to_string())
    }

    pub fn preview_date(&self, format: &QString) -> QString {
        qs(fsutil::format_time_with(chrono::Local::now().timestamp(), &format.to_string()))
    }

    pub fn smb_open(self: Pin<&mut Self>, uri: &QString) {
        let u = uri.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let res = smb::open(&u);
            let host = smb::parse(&u).map(|p| p.host).unwrap_or_default();
            let _ = qt.queue(move |m| match res {
                smb::MountResult::Ready(local) => m.smb_ready(qs(&u), qs(local)),
                smb::MountResult::AuthRequired(msg) => m.smb_auth_required(qs(&u), qs(host), qs(msg)),
                smb::MountResult::Failed(msg) => m.smb_error(qs(&u), qs(msg)),
            });
        });
    }

    pub fn set_smb_credentials(
        &self,
        host: &QString,
        user: &QString,
        domain: &QString,
        password: &QString,
        anonymous: bool,
        remember: bool,
    ) {
        smb::set_creds(
            &host.to_string(),
            smb::Creds {
                user: user.to_string(),
                domain: domain.to_string(),
                password: password.to_string(),
                anonymous,
                remember,
            },
        );
    }

    pub fn shutdown(&self) {
        smb::forget_session();
    }

    pub fn smb_info(&self, path: &QString) -> QString {
        match smb::uri_for_local(&path.to_string()) {
            Some((uri, root)) => qs(json!({ "uri": uri, "root": root }).to_string()),
            None => qs("{}"),
        }
    }

    pub fn display_path(&self, path: &QString) -> QString {
        let p = path.to_string();
        match smb::uri_for_local(&p) {
            Some((uri, root)) => qs(format!("{uri}{}", p.get(root.len()..).unwrap_or(""))),
            None => qs(p),
        }
    }

    pub fn smb_has_share(&self, uri: &QString) -> bool {
        smb::parse(&uri.to_string()).is_some_and(|u| u.share.is_some())
    }

    pub fn on_share(&self, path: &QString) -> bool {
        smb::on_gvfs(Path::new(&path.to_string()))
    }

    pub fn smb_creds_needed(&self, paths: &QStringList, dest: &QString) -> QString {
        let host = |p: &str| smb::gvfs_location(Path::new(p)).map(|(h, _, _)| h);
        let sources = paths.iter().map(|p| host(&p.to_string())).collect::<Vec<_>>();
        // Only local <-> share transfers go direct; share to share stays on gvfs.
        let hosts = match host(&dest.to_string()) {
            Some(h) if sources.iter().any(Option::is_none) => vec![h],
            Some(_) => Vec::new(),
            None => sources.into_iter().flatten().collect(),
        };
        qs(hosts.into_iter().find(|h| smb::creds_for(h).is_none()).as_deref().unwrap_or(""))
    }

    pub fn settings_url(&self) -> QString {
        qs(fsutil::path_to_uri(&config::settings_file().to_string_lossy()))
    }

    /// Keyed by the user-facing path, so SMB folders survive remounts.
    pub fn folder_view(&self, path: &QString) -> QString {
        qs(config::folder_view(&self.display_path(path).to_string()))
    }

    pub fn set_folder_view(&self, path: &QString, json: &QString) {
        config::set_folder_view(&self.display_path(path).to_string(), &json.to_string());
    }

    pub fn same_device(&self, a: &QString, b: &QString) -> bool {
        let dev = |q: &QString| fs::metadata(fsutil::uri_to_path(&q.to_string())).map(|m| m.dev()).ok();
        matches!((dev(a), dev(b)), (Some(x), Some(y)) if x == y)
    }

    pub fn suggest_name(&self, dir: &QString, base: &QString) -> QString {
        let d = PathBuf::from(dir.to_string());
        let b = base.to_string();
        if fs::symlink_metadata(d.join(&b)).is_err() {
            return qs(b);
        }
        let p = fsutil::unique_name(&d, &b, "");
        qs(p.file_name().unwrap_or_default().to_string_lossy())
    }
}

/// Properties dialog details; touches the disk and runs gio, so never on the GUI thread.
fn info_json(p: &Path) -> String {
    let Some(e) = fsutil::stat_entry(p, true) else {
        return "{}".into();
    };
    let meta = fs::symlink_metadata(p).ok();
    let tmeta = fs::metadata(p).ok();
    let (free, total) = fsutil::free_space(p).unwrap_or((0, 0));
    let (default_app, _) = if e.is_dir { (None, vec![]) } else { apps::for_mime(&e.mime) };
    let default_name = default_app
        .as_deref()
        .and_then(apps::lookup)
        .map(|a| a.name)
        .unwrap_or_default();
    let j = json!({
        "name": e.name,
        "path": e.path,
        "parent": p.parent().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default(),
        "isDir": e.is_dir,
        "isLink": e.is_link,
        "linkTarget": e.link_target,
        "mime": &*e.mime,
        "typeText": &*e.type_text,
        "glyph": e.glyph,
        "category": e.category,
        "size": e.size,
        "sizeText": if e.is_dir {
            format!("{} items", e.child_count.unwrap_or(0))
        } else {
            format!("{} ({} bytes)", fsutil::format_size(e.size), e.size)
        },
        "modified": fsutil::format_time_full(e.mtime),
        "accessed": tmeta.as_ref().map(|m| fsutil::format_time_full(m.atime())).unwrap_or_default(),
        "changed": meta.as_ref().map(|m| fsutil::format_time_full(m.ctime())).unwrap_or_default(),
        "owner": fsutil::user_name(e.uid),
        "group": fsutil::group_name(e.gid),
        "isOwner": e.uid == unsafe { libc::getuid() },
        "mode": e.mode & 0o7777,
        "permissions": fsutil::perm_string(e.mode, e.is_dir, e.is_link),
        "freeSpace": format!("{} of {} free", fsutil::format_size(free), fsutil::format_size(total)),
        "usage": if total > 0 { total.saturating_sub(free) as f64 / total as f64 } else { 0.0 },
        "defaultApp": default_app.unwrap_or_default(),
        "defaultAppName": default_name,
    });
    j.to_string()
}


/// "N bytes, F files, D folders" for the trees under `roots`, like `du -x`:
/// stays on each root's filesystem and counts hard links once. None if
/// `stale()` turns true mid-walk.
fn tree_size(roots: Vec<PathBuf>, stale: impl Fn() -> bool) -> Option<String> {
    let (mut bytes, mut files, mut dirs, mut root_dirs) = (0u64, 0u64, 0u64, 0u64);
    let mut links = HashSet::new();
    let mut stack: Vec<(PathBuf, Option<u64>)> = roots.into_iter().map(|p| (p, None)).collect();
    let mut n = 0u32;
    while let Some((p, dev)) = stack.pop() {
        n = n.wrapping_add(1);
        if n.is_multiple_of(512) && stale() {
            return None;
        }
        let Ok(m) = fs::symlink_metadata(&p) else { continue };
        if dev.is_some_and(|d| d != m.dev()) {
            continue;
        }
        if m.is_dir() {
            dirs += 1;
            if dev.is_none() {
                root_dirs += 1;
            }
            if let Ok(rd) = fs::read_dir(&p) {
                stack.extend(rd.flatten().map(|d| (d.path(), Some(m.dev()))));
            }
        } else if m.nlink() <= 1 || links.insert((m.dev(), m.ino())) {
            files += 1;
            bytes += m.len();
        }
    }
    let folders = dirs - root_dirs;
    let s = |n: u64| if n == 1 { "" } else { "s" };
    Some(format!(
        "{} ({} bytes), {} file{}, {} folder{}",
        fsutil::format_size(bytes),
        bytes,
        files,
        s(files),
        folders,
        s(folders)
    ))
}

/// Applies `mode` to `p` (and its tree when `recursive`). Folders get the mode
/// as-is; files keep their own execute bits so scripts and binaries survive a
/// recursive apply. A folder is chmodded before its contents when the new mode
/// lets us read it, after them when it would lock us out.
fn chmod_tree(p: &Path, mode: u32, is_root: bool, recursive: bool, errors: &mut Vec<String>) {
    let meta = if is_root { fs::metadata(p) } else { fs::symlink_metadata(p) };
    let m = match meta {
        Ok(m) => m,
        Err(e) => return errors.push(format!("{}: {e}", p.display())),
    };
    if m.file_type().is_symlink() {
        return;
    }
    let target = if is_root || m.is_dir() { mode } else { (mode & !0o111) | (m.mode() & 0o111) };
    let descend = recursive && m.is_dir();
    let early = !descend || mode & 0o500 == 0o500;
    let chmod = |errors: &mut Vec<String>| {
        if let Err(e) = fsutil::set_mode(p, target) {
            errors.push(format!("{}: {e}", p.display()));
        }
    };
    if early {
        chmod(errors);
    }
    if descend {
        match fs::read_dir(p) {
            Ok(rd) => {
                for c in rd.flatten() {
                    chmod_tree(&c.path(), mode, false, true, errors);
                }
            }
            Err(e) => errors.push(format!("{}: {e}", p.display())),
        }
    }
    if !early {
        chmod(errors);
    }
}

fn parent_disk(dev: &str) -> String {
    let out = Command::new("lsblk")
        .args(["-no", "PKNAME", dev])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if out.is_empty() {
        dev.to_string()
    } else {
        format!("/dev/{out}")
    }
}

#[cfg(test)]
mod tests {
    use super::gvfs_label;

    #[test]
    fn gvfs_labels() {
        assert_eq!(
            gvfs_label("smb-share:domain=WORKGROUP,server=fileserver,share=storage,user=sispx"),
            "storage on fileserver"
        );
        assert_eq!(gvfs_label("sftp:host=box.lan,user=me"), "box.lan (sftp)");
        assert_eq!(gvfs_label("mtp:host=Pixel"), "Pixel (mtp)");
        assert_eq!(gvfs_label("weird"), "weird");
    }
}

