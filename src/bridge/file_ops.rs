#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    #[qenum(FileOps)]
    enum Choice {
        Cancel,
        Skip,
        SkipAll,
        Replace,
        ReplaceAll,
        KeepBoth,
        KeepBothAll,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, busy)]
        #[qproperty(f64, progress)]
        #[qproperty(QString, title)]
        #[qproperty(QString, detail)]
        #[qproperty(i32, queued)]
        #[qproperty(bool, can_undo, cxx_name = "canUndo")]
        #[qproperty(QString, undo_text, cxx_name = "undoText")]
        #[qproperty(QString, kind)]
        #[qproperty(f64, done_bytes, cxx_name = "doneBytes")]
        #[qproperty(f64, total_bytes, cxx_name = "totalBytes")]
        #[qproperty(i32, done_items, cxx_name = "doneItems")]
        #[qproperty(i32, total_items, cxx_name = "totalItems")]
        #[qproperty(QString, current_file, cxx_name = "currentFile")]
        #[qproperty(f64, eta_seconds, cxx_name = "etaSeconds")]
        type FileOps = super::FileOpsRust;

        /// Destination can't hold symlinks (e.g. SMB/FAT); answered with
        /// Cancel / Skip / SkipAll via resolveConflict.
        #[qsignal]
        #[cxx_name = "symlinkConflict"]
        fn symlink_conflict(self: Pin<&mut FileOps>, name: QString, destination: QString);

        #[qsignal]
        fn conflict(
            self: Pin<&mut FileOps>,
            source: QString,
            target: QString,
            source_info: QString,
            target_info: QString,
        );

        #[qsignal]
        #[cxx_name = "errorOccurred"]
        fn error_occurred(self: Pin<&mut FileOps>, message: QString);

        #[qsignal]
        fn finished(self: Pin<&mut FileOps>, created: QStringList, cancelled: bool, failed: bool);

        /// A job (including the next queued one) has begun; its kind is already set.
        #[qsignal]
        fn started(self: Pin<&mut FileOps>);

        /// Decryption of these paths failed because the password was wrong.
        #[qsignal]
        #[cxx_name = "passwordRejected"]
        fn password_rejected(self: Pin<&mut FileOps>, paths: QStringList);

        #[qinvokable]
        fn copy(self: Pin<&mut FileOps>, sources: &QStringList, dest: &QString);

        #[qinvokable]
        #[cxx_name = "moveTo"]
        fn move_to(self: Pin<&mut FileOps>, sources: &QStringList, dest: &QString);

        #[qinvokable]
        fn link(self: Pin<&mut FileOps>, sources: &QStringList, dest: &QString);

        #[qinvokable]
        #[cxx_name = "trash"]
        fn trash_files(self: Pin<&mut FileOps>, sources: &QStringList);

        #[qinvokable]
        #[cxx_name = "remove"]
        fn remove_files(self: Pin<&mut FileOps>, sources: &QStringList);

        #[qinvokable]
        #[cxx_name = "emptyTrash"]
        fn empty_trash(self: Pin<&mut FileOps>);

        /// Extracts each archive next to itself: a lone top-level entry as is,
        /// anything else wrapped in a folder named after the archive.
        #[qinvokable]
        #[cxx_name = "extractHere"]
        fn extract_here(self: Pin<&mut FileOps>, archives: &QStringList);

        /// Extracts each archive into a folder named after it.
        #[qinvokable]
        #[cxx_name = "extractToFolder"]
        fn extract_to_folder(self: Pin<&mut FileOps>, archives: &QStringList);

        /// Packs `sources` into `target`; the format follows its suffix.
        #[qinvokable]
        fn compress(self: Pin<&mut FileOps>, sources: &QStringList, target: &QString);

        /// Encrypts files (to `<name>.age`) and folders (contents into a vault
        /// inside the folder) with `password`; the originals are deleted.
        #[qinvokable]
        fn encrypt(self: Pin<&mut FileOps>, sources: &QStringList, password: &QString);

        /// Reverses `encrypt` for `.age` files and locked folders.
        #[qinvokable]
        fn decrypt(self: Pin<&mut FileOps>, sources: &QStringList, password: &QString);

        /// Whether `path` is a folder whose contents are encrypted.
        #[qinvokable]
        #[cxx_name = "isLocked"]
        fn is_locked(self: &FileOps, path: &QString) -> bool;

        /// Name without the archive suffix, or empty if `path` isn't an extractable archive.
        #[qinvokable]
        #[cxx_name = "archiveStem"]
        fn archive_stem(self: &FileOps, path: &QString) -> QString;

        #[qinvokable]
        fn cancel(self: Pin<&mut FileOps>);

        #[qinvokable]
        #[cxx_name = "resolveConflict"]
        fn resolve_conflict(self: Pin<&mut FileOps>, choice: i32);

        #[qinvokable]
        fn rename(self: Pin<&mut FileOps>, path: &QString, new_name: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "createFolder"]
        fn create_folder(self: Pin<&mut FileOps>, parent: &QString, name: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "createFile"]
        fn create_file(
            self: Pin<&mut FileOps>,
            parent: &QString,
            name: &QString,
            template_path: &QString,
        ) -> QString;

        #[qinvokable]
        fn undo(self: Pin<&mut FileOps>) -> QString;
    }

    impl cxx_qt::Threading for FileOps {}
}

use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::fs::{self, File, FileTimes, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{symlink, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};

use cxx_qt::{CxxQtThread, CxxQtType, Threading};
use cxx_qt_lib::{QString, QStringList};

use crate::crypt::{self, CryptError};
use crate::{archive, fsutil, smb, smb_io};
use age::secrecy::SecretString;
use qobject::Choice;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Copy,
    Move,
    Link,
    Trash,
    Delete,
    EmptyTrash,
    ExtractHere,
    ExtractTo,
    Compress,
    Encrypt,
    Decrypt,
}

impl Kind {
    fn id(self) -> &'static str {
        match self {
            Kind::Copy => "copy",
            Kind::Move => "move",
            Kind::Link => "link",
            Kind::Trash => "trash",
            Kind::Delete => "delete",
            Kind::EmptyTrash => "empty_trash",
            Kind::ExtractHere | Kind::ExtractTo => "extract",
            Kind::Compress => "compress",
            Kind::Encrypt => "encrypt",
            Kind::Decrypt => "decrypt",
        }
    }

    fn verb(self) -> &'static str {
        match self {
            Kind::Copy => "Copying",
            Kind::Move => "Moving",
            Kind::Link => "Linking",
            Kind::Trash => "Moving to Trash",
            Kind::Delete => "Deleting",
            Kind::EmptyTrash => "Emptying Trash",
            Kind::ExtractHere | Kind::ExtractTo => "Extracting",
            Kind::Compress => "Compressing",
            Kind::Encrypt => "Encrypting",
            Kind::Decrypt => "Decrypting",
        }
    }
}

struct Job {
    kind: Kind,
    sources: Vec<PathBuf>,
    dest: PathBuf,
    /// Password for Encrypt/Decrypt; zeroed when the job is dropped.
    secret: Option<SecretString>,
}

/// A reversible record of the last completed user operation.
enum Undo {
    Moved(Vec<(PathBuf, PathBuf)>),
    Created(Vec<PathBuf>),
    Renamed(PathBuf, PathBuf),
    Trashed(Vec<PathBuf>),
}

pub struct FileOpsRust {
    busy: bool,
    progress: f64,
    title: QString,
    detail: QString,
    queued: i32,
    can_undo: bool,
    undo_text: QString,
    kind: QString,
    done_bytes: f64,
    total_bytes: f64,
    done_items: i32,
    total_items: i32,
    current_file: QString,
    eta_seconds: f64,

    pending: VecDeque<Job>,
    cancel_flag: Arc<AtomicBool>,
    conflict_tx: Option<Sender<Choice>>,
    last: Option<Undo>,
}

impl Default for FileOpsRust {
    fn default() -> Self {
        Self {
            busy: false,
            progress: 0.0,
            title: QString::default(),
            detail: QString::default(),
            queued: 0,
            can_undo: false,
            undo_text: QString::default(),
            kind: QString::default(),
            done_bytes: 0.0,
            total_bytes: 0.0,
            done_items: 0,
            total_items: 0,
            current_file: QString::default(),
            eta_seconds: -1.0,
            pending: VecDeque::new(),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            conflict_tx: None,
            last: None,
        }
    }
}

fn to_paths(list: &QStringList) -> Vec<PathBuf> {
    list.iter()
        .map(|s| PathBuf::from(fsutil::uri_to_path(&s.to_string())))
        .collect()
}

/// JSON card data for the conflict dialog: name, type, size, modified, glyph.
fn describe(p: &Path) -> String {
    let Some(e) = fsutil::stat_entry(p, true) else {
        return "{}".into();
    };
    let size = if e.is_dir {
        let n = e.child_count.unwrap_or(0);
        format!("{n} item{}", plural(n as usize))
    } else {
        fsutil::format_size(e.size)
    };
    serde_json::json!({
        "name": e.name,
        "type": &*e.type_text,
        "size": size,
        "modified": fsutil::format_time_full(e.mtime),
        "glyph": e.glyph,
        "category": e.category,
        "isDir": e.is_dir,
    })
    .to_string()
}

fn valid_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name == "." || name == ".." {
        return Err("Invalid file name".into());
    }
    if name.contains('/') {
        return Err("File names cannot contain \"/\"".into());
    }
    Ok(())
}

impl qobject::FileOps {
    fn enqueue(self: Pin<&mut Self>, kind: Kind, sources: Vec<PathBuf>, dest: PathBuf) {
        self.enqueue_job(Job { kind, sources, dest, secret: None });
    }

    fn enqueue_job(mut self: Pin<&mut Self>, job: Job) {
        if job.sources.is_empty() && job.kind != Kind::EmptyTrash {
            return;
        }
        self.as_mut().rust_mut().pending.push_back(job);
        let q = self.pending.len() as i32;
        self.as_mut().set_queued(q);
        if !self.busy {
            self.start_next();
        }
    }

    fn start_next(mut self: Pin<&mut Self>) {
        let Some(job) = self.as_mut().rust_mut().pending.pop_front() else {
            self.as_mut().set_busy(false);
            self.as_mut().set_queued(0);
            return;
        };
        let q = self.pending.len() as i32;
        self.as_mut().set_queued(q);
        self.as_mut().set_progress(0.0);
        self.as_mut().set_title(QString::from(job.kind.verb()));
        self.as_mut().set_detail(QString::from("Preparing…"));
        self.as_mut().set_kind(QString::from(job.kind.id()));
        self.as_mut().set_done_bytes(0.0);
        self.as_mut().set_total_bytes(0.0);
        self.as_mut().set_done_items(0);
        self.as_mut().set_total_items(0);
        self.as_mut().set_current_file(QString::default());
        self.as_mut().set_eta_seconds(-1.0);

        let cancel = Arc::new(AtomicBool::new(false));
        self.as_mut().rust_mut().cancel_flag = cancel.clone();
        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().conflict_tx = Some(tx);
        self.as_mut().set_busy(true);
        self.as_mut().started();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let mut w = Worker::new(qt.clone(), cancel, rx, job.kind);
            w.run(&job);
            let cancelled = w.cancelled();
            let Worker {
                errors,
                error_count,
                created,
                fresh,
                moved,
                trashed,
                done_items,
                done_bytes,
                rejected,
                ..
            } = w;
            let kind = job.kind;
            let _ = qt.queue(move |mut m| {
                m.as_mut().rust_mut().conflict_tx = None;
                m.as_mut().set_progress(1.0);
                // Progress reports are throttled, so the last counts may never have been sent.
                m.as_mut().set_done_items(done_items as i32);
                m.as_mut().set_done_bytes(done_bytes as f64);
                let list: QStringList = created
                    .iter()
                    .map(|p| QString::from(&p.to_string_lossy().into_owned()))
                    .collect();
                let undo = match kind {
                    Kind::Move if !moved.is_empty() => Some(Undo::Moved(moved)),
                    Kind::Copy | Kind::Link | Kind::ExtractHere | Kind::ExtractTo | Kind::Compress
                        if !fresh.is_empty() =>
                    {
                        Some(Undo::Created(fresh))
                    }
                    Kind::Trash if !trashed.is_empty() => Some(Undo::Trashed(trashed)),
                    _ => None,
                };
                // Undo always means the latest operation: one that can't be
                // undone (e.g. a merge) must not leave an older one armed.
                match undo {
                    Some(u) => m.as_mut().set_last(u),
                    None => m.as_mut().clear_last(),
                }
                if error_count > 0 {
                    let mut msg = errors.iter().take(8).cloned().collect::<Vec<_>>().join("\n");
                    if error_count > 8 {
                        msg.push_str(&format!("\n… and {} more", error_count - 8));
                    }
                    m.as_mut().error_occurred(QString::from(&msg));
                }
                let failed = error_count > 0;
                if !rejected.is_empty() {
                    let paths: QStringList = rejected
                        .iter()
                        .map(|p| QString::from(&p.to_string_lossy().into_owned()))
                        .collect();
                    m.as_mut().password_rejected(paths);
                }
                m.as_mut().finished(list, cancelled, failed);
                m.start_next();
            });
        });
    }

    fn clear_last(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().last = None;
        self.as_mut().set_undo_text(QString::default());
        self.as_mut().set_can_undo(false);
    }

    fn set_last(mut self: Pin<&mut Self>, u: Undo) {
        let text = match &u {
            Undo::Moved(v) => format!("Undo Move ({} item{})", v.len(), plural(v.len())),
            Undo::Created(v) => format!("Undo Copy ({} item{})", v.len(), plural(v.len())),
            Undo::Renamed(_, to) => format!(
                "Undo Rename \"{}\"",
                to.file_name().unwrap_or_default().to_string_lossy()
            ),
            Undo::Trashed(v) => format!("Undo Trash ({} item{})", v.len(), plural(v.len())),
        };
        self.as_mut().rust_mut().last = Some(u);
        self.as_mut().set_undo_text(QString::from(&text));
        self.as_mut().set_can_undo(true);
    }

    pub fn copy(self: Pin<&mut Self>, sources: &QStringList, dest: &QString) {
        self.enqueue(Kind::Copy, to_paths(sources), PathBuf::from(dest.to_string()));
    }

    pub fn move_to(self: Pin<&mut Self>, sources: &QStringList, dest: &QString) {
        self.enqueue(Kind::Move, to_paths(sources), PathBuf::from(dest.to_string()));
    }

    pub fn link(self: Pin<&mut Self>, sources: &QStringList, dest: &QString) {
        self.enqueue(Kind::Link, to_paths(sources), PathBuf::from(dest.to_string()));
    }

    pub fn trash_files(self: Pin<&mut Self>, sources: &QStringList) {
        self.enqueue(Kind::Trash, to_paths(sources), PathBuf::new());
    }

    pub fn remove_files(self: Pin<&mut Self>, sources: &QStringList) {
        self.enqueue(Kind::Delete, to_paths(sources), PathBuf::new());
    }

    pub fn empty_trash(self: Pin<&mut Self>) {
        self.enqueue(Kind::EmptyTrash, Vec::new(), PathBuf::new());
    }

    pub fn extract_here(self: Pin<&mut Self>, archives: &QStringList) {
        self.enqueue(Kind::ExtractHere, to_paths(archives), PathBuf::new());
    }

    pub fn extract_to_folder(self: Pin<&mut Self>, archives: &QStringList) {
        self.enqueue(Kind::ExtractTo, to_paths(archives), PathBuf::new());
    }

    pub fn compress(self: Pin<&mut Self>, sources: &QStringList, target: &QString) {
        self.enqueue(Kind::Compress, to_paths(sources), PathBuf::from(target.to_string()));
    }

    pub fn encrypt(self: Pin<&mut Self>, sources: &QStringList, password: &QString) {
        self.enqueue_crypt(Kind::Encrypt, sources, password);
    }

    pub fn decrypt(self: Pin<&mut Self>, sources: &QStringList, password: &QString) {
        self.enqueue_crypt(Kind::Decrypt, sources, password);
    }

    fn enqueue_crypt(self: Pin<&mut Self>, kind: Kind, sources: &QStringList, password: &QString) {
        let secret = SecretString::from(password.to_string());
        self.enqueue_job(Job { kind, sources: to_paths(sources), dest: PathBuf::new(), secret: Some(secret) });
    }

    pub fn is_locked(&self, path: &QString) -> bool {
        fs::symlink_metadata(Path::new(&path.to_string()).join(crypt::VAULT)).is_ok_and(|m| m.is_file())
    }

    pub fn archive_stem(&self, path: &QString) -> QString {
        let p = PathBuf::from(path.to_string());
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        QString::from(archive::stem(&name).unwrap_or_default())
    }

    pub fn cancel(mut self: Pin<&mut Self>) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        if let Some(tx) = &self.conflict_tx {
            let _ = tx.send(Choice::Cancel);
        }
        self.as_mut().rust_mut().pending.clear();
        self.set_queued(0);
    }

    pub fn resolve_conflict(self: Pin<&mut Self>, choice: i32) {
        if let Some(tx) = &self.conflict_tx {
            let _ = tx.send(Choice { repr: choice as _ });
        }
    }

    pub fn rename(mut self: Pin<&mut Self>, path: &QString, new_name: &QString) -> QString {
        let from = PathBuf::from(path.to_string());
        let name = new_name.to_string();
        if let Err(e) = valid_name(&name) {
            return QString::from(&e);
        }
        let to = from.with_file_name(&name);
        if to == from {
            return QString::default();
        }
        if fs::symlink_metadata(&to).is_ok() {
            return QString::from(&format!("\"{name}\" already exists"));
        }
        match fs::rename(&from, &to) {
            Ok(()) => {
                self.as_mut().set_last(Undo::Renamed(from, to));
                QString::default()
            }
            Err(e) => QString::from(&e.to_string()),
        }
    }

    pub fn create_folder(mut self: Pin<&mut Self>, parent: &QString, name: &QString) -> QString {
        let name = name.to_string();
        if let Err(e) = valid_name(&name) {
            return QString::from(&format!("!{e}"));
        }
        let p = Path::new(&parent.to_string()).join(&name);
        match fs::create_dir(&p) {
            Ok(()) => {
                self.as_mut().set_last(Undo::Created(vec![p.clone()]));
                QString::from(&p.to_string_lossy().into_owned())
            }
            Err(e) => QString::from(&format!("!{e}")),
        }
    }

    pub fn create_file(
        mut self: Pin<&mut Self>,
        parent: &QString,
        name: &QString,
        template_path: &QString,
    ) -> QString {
        let name = name.to_string();
        if let Err(e) = valid_name(&name) {
            return QString::from(&format!("!{e}"));
        }
        let p = Path::new(&parent.to_string()).join(&name);
        if fs::symlink_metadata(&p).is_ok() {
            return QString::from(&format!("!\"{name}\" already exists"));
        }
        let tpl = template_path.to_string();
        let res = if tpl.is_empty() {
            File::create_new(&p).map(|_| ())
        } else {
            fs::copy(&tpl, &p).map(|_| ())
        };
        match res {
            Ok(()) => {
                self.as_mut().set_last(Undo::Created(vec![p.clone()]));
                QString::from(&p.to_string_lossy().into_owned())
            }
            Err(e) => QString::from(&format!("!{e}")),
        }
    }

    pub fn undo(mut self: Pin<&mut Self>) -> QString {
        let Some(last) = self.as_mut().rust_mut().last.take() else {
            return QString::default();
        };
        self.as_mut().set_can_undo(false);
        self.as_mut().set_undo_text(QString::default());
        let mut errors = Vec::new();
        match last {
            Undo::Renamed(from, to) => {
                if let Err(e) = fs::rename(&to, &from) {
                    errors.push(e.to_string());
                }
            }
            Undo::Moved(pairs) => {
                for (from, to) in pairs.into_iter().rev() {
                    if fs::symlink_metadata(&from).is_ok() {
                        errors.push(format!("{} already exists", from.display()));
                        continue;
                    }
                    if fs::rename(&to, &from).is_err() {
                        let list: QStringList =
                            std::iter::once(QString::from(&to.to_string_lossy().into_owned()))
                                .collect();
                        let parent = from.parent().unwrap_or(Path::new("/"));
                        self.as_mut().move_to(
                            &list,
                            &QString::from(&parent.to_string_lossy().into_owned()),
                        );
                    }
                }
            }
            Undo::Created(paths) => {
                let list: QStringList = paths
                    .iter()
                    .map(|p| QString::from(&p.to_string_lossy().into_owned()))
                    .collect();
                self.as_mut().trash_files(&list);
            }
            Undo::Trashed(originals) => {
                let originals: HashSet<PathBuf> = originals.into_iter().collect();
                let mut latest: HashMap<PathBuf, trash::TrashItem> = HashMap::new();
                for it in crate::trashdir::list_items() {
                    let orig = it.original_path();
                    if !originals.contains(&orig) {
                        continue;
                    }
                    match latest.get(&orig) {
                        Some(old) if old.time_deleted >= it.time_deleted => {}
                        _ => {
                            latest.insert(orig, it);
                        }
                    }
                }
                if let Err(e) = trash::os_limited::restore_all(latest.into_values()) {
                    errors.push(e.to_string());
                }
            }
        }
        QString::from(&errors.join("\n"))
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

struct Worker {
    qt: CxxQtThread<qobject::FileOps>,
    cancel: Arc<AtomicBool>,
    rx: Receiver<Choice>,
    kind: Kind,
    total_bytes: u64,
    done_bytes: u64,
    total_items: usize,
    done_items: usize,
    last_emit: Instant,
    started: Instant,
    sticky: Option<Choice>,
    skip_symlinks: bool,
    /// Items left behind on purpose (skipped conflicts, refused symlinks).
    skipped: usize,
    /// First `MAX_ERRORS` messages; `error_count` holds the real total.
    errors: Vec<String>,
    error_count: usize,
    /// Every top-level target written (fresh or replaced), for selection.
    created: Vec<PathBuf>,
    /// Top-level targets that did not exist before; safe to undo.
    fresh: Vec<PathBuf>,
    moved: Vec<(PathBuf, PathBuf)>,
    trashed: Vec<PathBuf>,
    /// Sources whose decryption failed on the password.
    rejected: Vec<PathBuf>,
    /// (items, bytes) per top-level source, from `scan`.
    totals: HashMap<PathBuf, (usize, u64)>,
    buf: Vec<u8>,
    /// Set for copies into a remote destination.
    pool: Option<Pool>,
    /// Pool copies that failed, so they are not reported as created.
    failed: HashSet<PathBuf>,
    /// Folder modes to apply once the pool has finished writing into them.
    deferred_modes: Vec<(PathBuf, u32)>,
    /// Names in each remote destination folder, listed once, so resolving a
    /// target is a set lookup instead of a FUSE round trip per file.
    listed: HashMap<PathBuf, HashSet<OsString>>,
}

const MAX_ERRORS: usize = 100;
const COPY_CHUNK: usize = 8 << 20;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Op {
    Copy,
    Move,
    Link,
}

enum Resolution {
    /// Target does not exist yet.
    Write(PathBuf),
    /// Folder onto existing folder: contents are merged.
    Merge(PathBuf),
    /// Existing target is replaced only once the new item is complete.
    Replace(PathBuf),
    Skip,
    Abort,
}

impl Worker {
    fn new(
        qt: CxxQtThread<qobject::FileOps>,
        cancel: Arc<AtomicBool>,
        rx: Receiver<Choice>,
        kind: Kind,
    ) -> Self {
        Self {
            qt,
            cancel,
            rx,
            kind,
            total_bytes: 0,
            done_bytes: 0,
            total_items: 0,
            done_items: 0,
            last_emit: Instant::now() - Duration::from_secs(1),
            started: Instant::now(),
            sticky: None,
            skip_symlinks: false,
            skipped: 0,
            errors: Vec::new(),
            error_count: 0,
            created: Vec::new(),
            fresh: Vec::new(),
            moved: Vec::new(),
            trashed: Vec::new(),
            rejected: Vec::new(),
            totals: HashMap::new(),
            buf: Vec::new(),
            pool: None,
            failed: HashSet::new(),
            deferred_modes: Vec::new(),
            listed: HashMap::new(),
        }
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    fn error(&mut self, msg: String) {
        self.error_count += 1;
        if self.errors.len() < MAX_ERRORS {
            self.errors.push(msg);
        }
    }

    fn fail(&mut self, p: &Path, e: io::Error) {
        if e.kind() != io::ErrorKind::Interrupted {
            self.error(format!("{}: {e}", p.display()));
        }
    }

    /// Error + skip count; unchanged across an operation means nothing was left behind.
    fn losses(&self) -> (usize, usize) {
        (self.error_count, self.skipped)
    }

    fn report(&mut self, current: &Path, force: bool) {
        if !force && self.last_emit.elapsed() < Duration::from_millis(80) {
            return;
        }
        self.last_emit = Instant::now();
        let frac = if self.total_bytes > 0 {
            self.done_bytes as f64 / self.total_bytes as f64
        } else if self.total_items > 0 {
            self.done_items as f64 / self.total_items as f64
        } else {
            0.0
        };
        let name = current
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let title = format!(
            "{} \"{}\" ({} of {})",
            self.kind.verb(),
            name,
            (self.done_items + 1).min(self.total_items.max(1)),
            self.total_items.max(1)
        );
        let secs = self.started.elapsed().as_secs_f64().max(0.001);
        let rate = self.done_bytes as f64 / secs;
        let eta = if self.total_bytes > 0 && self.done_bytes > 0 {
            self.total_bytes.saturating_sub(self.done_bytes) as f64 / rate
        } else {
            -1.0
        };
        let detail = if self.total_bytes > 0 {
            format!(
                "{} of {} — {}/s — {} left",
                fsutil::format_size(self.done_bytes),
                fsutil::format_size(self.total_bytes),
                fsutil::format_size(rate as u64),
                fmt_eta(eta.max(0.0))
            )
        } else {
            format!("{} of {} items", self.done_items, self.total_items)
        };
        let (db, tb) = (self.done_bytes as f64, self.total_bytes as f64);
        let (di, ti) = (self.done_items as i32, self.total_items as i32);
        let _ = self.qt.queue(move |mut m| {
            m.as_mut().set_progress(frac.clamp(0.0, 1.0));
            m.as_mut().set_title(QString::from(&title));
            m.as_mut().set_detail(QString::from(&detail));
            m.as_mut().set_done_bytes(db);
            m.as_mut().set_total_bytes(tb);
            m.as_mut().set_done_items(di);
            m.as_mut().set_total_items(ti);
            m.as_mut().set_current_file(QString::from(&name));
            m.as_mut().set_eta_seconds(eta);
        });
    }

    fn run(&mut self, job: &Job) {
        match job.kind {
            Kind::Copy | Kind::Move | Kind::Link => self.run_transfer(job),
            Kind::Trash => self.run_trash(job),
            Kind::Delete => self.run_delete(job),
            Kind::EmptyTrash => self.run_empty_trash(),
            Kind::ExtractHere | Kind::ExtractTo => self.run_extract(job),
            Kind::Compress => self.run_compress(job),
            Kind::Encrypt | Kind::Decrypt => self.run_crypt(job),
        }
    }

    fn scan(&mut self, sources: &[PathBuf], count_bytes: bool) {
        for s in sources {
            let (mut items, mut bytes) = (0usize, 0u64);
            let mut stack = vec![s.clone()];
            while let Some(p) = stack.pop() {
                if self.cancelled() {
                    return;
                }
                let Ok(m) = fs::symlink_metadata(&p) else { continue };
                items += 1;
                if m.is_dir() {
                    if let Ok(rd) = fs::read_dir(&p) {
                        stack.extend(rd.filter_map(|d| d.ok()).map(|d| d.path()));
                    }
                } else if count_bytes && m.is_file() {
                    bytes += m.len();
                }
            }
            self.total_items += items;
            self.total_bytes += bytes;
            self.totals.insert(s.clone(), (items, bytes));
        }
    }

    fn run_transfer(&mut self, job: &Job) {
        let op = match job.kind {
            Kind::Link => Op::Link,
            Kind::Move => Op::Move,
            _ => Op::Copy,
        };
        if op == Op::Link {
            self.total_items = job.sources.len();
        } else {
            self.scan(&job.sources, true);
            if fsutil::is_remote_fs(&job.dest) || job.sources.iter().any(|s| smb_io::remote(s).is_some()) {
                self.pool = Some(Pool::new(&self.cancel));
            }
        }
        if let Some(first) = job.sources.first() {
            self.report(first, true);
        }
        for src in &job.sources {
            if self.cancelled() {
                break;
            }
            let Some(name) = src.file_name() else { continue };
            let same_dir = src.parent().is_some_and(|p| same_file(p, &job.dest));
            if op != Op::Link && inside(&job.dest, src) {
                self.error(format!(
                    "Cannot {} \"{}\" into itself",
                    if op == Op::Move { "move" } else { "copy" },
                    src.display()
                ));
                continue;
            }
            let target = match op {
                Op::Move if same_dir => continue,
                Op::Copy if same_dir => fsutil::unique_name(&job.dest, &name.to_string_lossy(), "copy"),
                Op::Link if same_dir || fs::symlink_metadata(job.dest.join(name)).is_ok() => {
                    fsutil::unique_name(&job.dest, &name.to_string_lossy(), "link")
                }
                _ => job.dest.join(name),
            };
            let res = self.resolve(src, target);
            if let Some((target, is_new)) = self.place(op, src, res) {
                if is_new {
                    if op == Op::Move {
                        self.moved.push((src.clone(), target.clone()));
                    } else {
                        self.fresh.push(target.clone());
                    }
                }
                self.created.push(target);
            }
            if op == Op::Link {
                self.done_items += 1;
            }
            self.report(src, false);
        }
        self.drain();
        if !self.failed.is_empty() {
            let failed = std::mem::take(&mut self.failed);
            self.created.retain(|p| !failed.contains(p));
            self.fresh.retain(|p| !failed.contains(p));
            self.moved.retain(|(_, p)| !failed.contains(p));
        }
    }

    /// Whether `target` exists, from one listing of its folder when that is
    /// on a gvfs mount (a negative lookup there costs a server round trip).
    fn target_exists(&mut self, target: &Path) -> bool {
        let (Some(dir), Some(name)) = (target.parent(), target.file_name()) else {
            return fs::symlink_metadata(target).is_ok();
        };
        if !smb::on_gvfs(dir) {
            return fs::symlink_metadata(target).is_ok();
        }
        if !self.listed.contains_key(dir) {
            let Ok(rd) = fs::read_dir(dir) else { return fs::symlink_metadata(target).is_ok() };
            self.listed.insert(dir.to_path_buf(), rd.flatten().map(|d| d.file_name()).collect());
        }
        self.listed[dir].contains(name)
    }

    /// Records a name this job is about to create, for `target_exists`.
    fn note_written(&mut self, target: &Path) {
        if let (Some(dir), Some(name)) = (target.parent(), target.file_name()) {
            if let Some(names) = self.listed.get_mut(dir) {
                names.insert(name.to_os_string());
            }
        }
    }

    /// Decide where `src` should be written when `target` may already exist.
    fn resolve(&mut self, src: &Path, target: PathBuf) -> Resolution {
        if !self.target_exists(&target) {
            self.note_written(&target);
            return Resolution::Write(target);
        }
        let Ok(tmeta) = fs::symlink_metadata(&target) else {
            self.note_written(&target);
            return Resolution::Write(target);
        };
        let smeta = fs::symlink_metadata(src).ok();
        if smeta
            .as_ref()
            .is_some_and(|m| m.dev() == tmeta.dev() && m.ino() == tmeta.ino())
        {
            // Same entry reached through another path (symlink, bind mount):
            // replacing it would destroy the source.
            if self.kind != Kind::Copy {
                return Resolution::Skip;
            }
            let dir = target.parent().unwrap_or(Path::new("/"));
            let name = target.file_name().unwrap_or_default().to_string_lossy();
            let p = fsutil::unique_name(dir, &name, "copy");
            self.note_written(&p);
            return Resolution::Write(p);
        }
        let src_dir = smeta.is_some_and(|m| m.is_dir());
        let choice = match self.sticky {
            Some(c) => c,
            None => self.ask(src, &target),
        };
        match choice {
            Choice::Skip => Resolution::Skip,
            Choice::SkipAll => {
                self.sticky = Some(Choice::Skip);
                Resolution::Skip
            }
            Choice::Replace | Choice::ReplaceAll => {
                if choice == Choice::ReplaceAll {
                    self.sticky = Some(Choice::Replace);
                }
                if src_dir && tmeta.is_dir() && self.kind != Kind::Link {
                    Resolution::Merge(target)
                } else {
                    Resolution::Replace(target)
                }
            }
            Choice::KeepBoth | Choice::KeepBothAll => {
                if choice == Choice::KeepBothAll {
                    self.sticky = Some(Choice::KeepBoth);
                }
                let dir = target.parent().unwrap_or(Path::new("/")).to_path_buf();
                let name = target.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let p = fsutil::unique_name(&dir, &name, "");
                self.note_written(&p);
                Resolution::Write(p)
            }
            _ => {
                self.cancel.store(true, Ordering::SeqCst);
                Resolution::Abort
            }
        }
    }

    fn apply(&mut self, op: Op, src: &Path, dst: &Path) -> io::Result<()> {
        match op {
            Op::Copy => self.copy_item(src, dst),
            Op::Move => self.move_item(src, dst),
            Op::Link => self.make_link(src, dst),
        }
    }

    /// Carries out `op` for a resolved target. Returns the final path and
    /// whether it is a brand new item (not a merge or replacement).
    fn place(&mut self, op: Op, src: &Path, res: Resolution) -> Option<(PathBuf, bool)> {
        let (target, is_new) = match res {
            Resolution::Write(t) => (t, true),
            Resolution::Merge(t) => (t, false),
            Resolution::Replace(t) => return self.replace(op, src, t).map(|t| (t, false)),
            Resolution::Skip | Resolution::Abort => {
                self.skipped += 1;
                return None;
            }
        };
        let before = self.losses();
        match self.apply(op, src, &target) {
            Ok(()) if op == Op::Link && self.losses() != before => None,
            Ok(()) => Some((target, is_new)),
            Err(e) => {
                self.fail(src, e);
                None
            }
        }
    }

    /// Writes `src` next to `target` under a temporary name and swaps it in
    /// only once complete, so a failed or cancelled replace keeps the original.
    fn replace(&mut self, op: Op, src: &Path, target: PathBuf) -> Option<PathBuf> {
        let tmp = partial_path(&target);
        if let Some(ok) = self.replace_pooled(op, src, &tmp, &target) {
            return ok.then_some(target);
        }
        let before = self.losses();
        let res = self.apply(op, src, &tmp);
        self.drain();
        let complete = match op {
            // A move is done once the source is gone, whatever else happened.
            Op::Move => fs::symlink_metadata(src).is_err(),
            _ => res.is_ok() && !self.cancelled() && self.losses() == before,
        };
        if let Err(e) = res {
            self.fail(src, e);
        }
        if !complete {
            let _ = remove_any(&tmp);
            return None;
        }
        let swap = (|| {
            let dirs = fs::symlink_metadata(&target).is_ok_and(|m| m.is_dir())
                || fs::symlink_metadata(&tmp).is_ok_and(|m| m.is_dir());
            if dirs {
                remove_any(&target)?;
            }
            fs::rename(&tmp, &target)
        })();
        match swap {
            Ok(()) => Some(target),
            Err(e) => {
                if op == Op::Move {
                    self.error(format!("{}: {e} (kept as {})", target.display(), tmp.display()));
                } else {
                    self.fail(&target, e);
                    let _ = remove_any(&tmp);
                }
                None
            }
        }
    }

    /// Replace of a regular file onto a non-directory while the pool is
    /// active: the pool thread swaps the finished copy into place itself, so
    /// a run of replaced files keeps every connection busy instead of
    /// draining the pool for each one. None leaves it to the synchronous path.
    fn replace_pooled(&mut self, op: Op, src: &Path, tmp: &Path, target: &Path) -> Option<bool> {
        if self.pool.is_none() || op == Op::Link {
            return None;
        }
        let meta = fs::symlink_metadata(src).ok()?;
        if !meta.is_file() || fs::symlink_metadata(target).is_ok_and(|m| m.is_dir()) {
            return None;
        }
        if op == Op::Move {
            // Same filesystem: one atomic rename does the whole replace.
            match fs::rename(src, target) {
                Ok(()) => {
                    self.done_items += 1;
                    self.done_bytes += meta.len();
                    return Some(true);
                }
                Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {}
                Err(e) => {
                    self.fail(src, e);
                    return Some(false);
                }
            }
        }
        self.submit(src, tmp, meta, op == Op::Move, Some(target.to_path_buf()));
        Some(true)
    }

    /// Creates a symlink; when the destination filesystem refuses symlinks the
    /// user is asked once whether to skip it (or all of them).
    fn make_link(&mut self, target: &Path, link: &Path) -> io::Result<()> {
        if self.skip_symlinks {
            self.skipped += 1;
            return Ok(());
        }
        match symlink(target, link) {
            Err(e) if matches!(e.raw_os_error(), Some(libc::EPERM) | Some(libc::EOPNOTSUPP) | Some(libc::ENOSYS)) => {
                let name = QString::from(&link.file_name().unwrap_or_default().to_string_lossy().into_owned());
                let dest = QString::from(&link.parent().unwrap_or(Path::new("/")).to_string_lossy().into_owned());
                let _ = self.qt.queue(move |m| m.symlink_conflict(name, dest));
                self.skipped += 1;
                match self.rx.recv().unwrap_or(Choice::Cancel) {
                    Choice::Skip => Ok(()),
                    Choice::SkipAll => {
                        self.skip_symlinks = true;
                        Ok(())
                    }
                    _ => {
                        self.cancel.store(true, Ordering::SeqCst);
                        Ok(())
                    }
                }
            }
            other => other,
        }
    }

    fn ask(&mut self, src: &Path, target: &Path) -> Choice {
        let s = QString::from(&src.to_string_lossy().into_owned());
        let t = QString::from(&target.to_string_lossy().into_owned());
        let si = QString::from(&describe(src));
        let ti = QString::from(&describe(target));
        let _ = self.qt.queue(move |m| m.conflict(s, t, si, ti));
        self.rx.recv().unwrap_or(Choice::Cancel)
    }

    /// Resolves and places every child of `src` into the existing folder `dst`.
    fn merge_children(&mut self, op: Op, src: &Path, dst: &Path) -> io::Result<()> {
        for d in fs::read_dir(src)?.flatten() {
            if self.cancelled() {
                break;
            }
            let child = d.path();
            let res = self.resolve(&child, dst.join(d.file_name()));
            self.place(op, &child, res);
        }
        Ok(())
    }

    fn move_item(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
        if fs::symlink_metadata(dst).is_ok_and(|m| m.is_dir())
            && fs::symlink_metadata(src).is_ok_and(|m| m.is_dir())
        {
            // Merge move: the source folder is only removed if everything left it.
            self.merge_children(Op::Move, src, dst)?;
            self.drain();
            let _ = fs::remove_dir(src);
            self.done_items += 1;
            return Ok(());
        }
        match fs::rename(src, dst) {
            Ok(()) => {
                let (n, b) = match self.totals.get(src) {
                    Some(&t) => t,
                    None => tree_totals(dst),
                };
                self.done_items += n.max(1);
                self.done_bytes += b;
                self.report(src, false);
                Ok(())
            }
            Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
                let meta = fs::symlink_metadata(src)?;
                if self.pool.is_some() && meta.is_file() {
                    // The pool removes the source once its copy is complete.
                    self.submit(src, dst, meta, true, None);
                    return Ok(());
                }
                let before = self.losses();
                self.copy_item(src, dst)?;
                self.drain();
                if self.cancelled() || self.losses() != before {
                    // Incomplete copy (errors or skipped items): keep the source.
                    return Ok(());
                }
                remove_any(src)
            }
            Err(e) => Err(e),
        }
    }

    fn copy_item(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
        if self.cancelled() {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let meta = fs::symlink_metadata(src)?;
        let ft = meta.file_type();
        if ft.is_symlink() {
            let t = fs::read_link(src)?;
            self.make_link(&t, dst)?;
        } else if ft.is_dir() {
            if fs::symlink_metadata(dst).is_err() {
                fs::create_dir(dst)?;
            }
            self.merge_children(Op::Copy, src, dst)?;
            if self.pool.is_some() {
                // Pool copies into this folder may still be running, and a
                // read-only mode would make them fail: applied after drain.
                self.deferred_modes.push((dst.to_path_buf(), meta.mode()));
            } else {
                best_effort(fs::set_permissions(dst, fs::Permissions::from_mode(meta.mode())))?;
            }
        } else if ft.is_file() {
            if self.pool.is_some() {
                self.submit(src, dst, meta, false, None);
                return Ok(());
            }
            let mut buf = std::mem::take(&mut self.buf);
            let cancel = self.cancel.clone();
            let res = copy_file(src, dst, &meta, &cancel, &mut buf, &mut |n| {
                self.done_bytes += n;
                self.report(src, false);
            });
            self.buf = buf;
            res?;
        } else {
            self.skipped += 1;
            self.error(format!("{}: special files are skipped", src.display()));
        }
        self.done_items += 1;
        self.report(src, false);
        Ok(())
    }

    /// Hands a regular file to the copy pool, waiting (and reporting) while
    /// its queue is full.
    fn submit(&mut self, src: &Path, dst: &Path, meta: fs::Metadata, remove_src: bool, swap_to: Option<PathBuf>) {
        let mut task = Task { src: src.to_path_buf(), dst: dst.to_path_buf(), meta, remove_src, swap_to };
        loop {
            let pool = self.pool.as_mut().unwrap();
            match pool.tx.try_send(task) {
                Ok(()) => {
                    pool.pending += 1;
                    pool.current = src.to_path_buf();
                    return;
                }
                Err(mpsc::TrySendError::Full(t)) => {
                    task = t;
                    self.collect(Duration::from_millis(80));
                }
                Err(mpsc::TrySendError::Disconnected(_)) => return,
            }
        }
    }

    /// Folds in finished pool copies, waiting up to `wait` for the first one.
    fn collect(&mut self, wait: Duration) {
        let Some(pool) = self.pool.as_mut() else { return };
        self.done_bytes += pool.bytes.swap(0, Ordering::Relaxed);
        let mut finished = Vec::new();
        if let Ok(r) = pool.done.recv_timeout(wait) {
            finished.push(r);
            finished.extend(pool.done.try_iter());
        }
        pool.pending -= finished.len();
        let current = pool.current.clone();
        for (src, dst, res) in finished {
            self.done_items += 1;
            if let Err(e) = res {
                self.failed.insert(dst);
                self.fail(&src, e);
            }
        }
        self.report(&current, false);
    }

    /// Waits for every queued pool copy to finish.
    fn drain(&mut self) {
        while self.pool.as_ref().is_some_and(|p| p.pending > 0) {
            self.collect(Duration::from_millis(80));
        }
        if let Some(pool) = self.pool.as_ref() {
            self.done_bytes += pool.bytes.swap(0, Ordering::Relaxed);
        }
        for (p, mode) in std::mem::take(&mut self.deferred_modes) {
            let _ = fs::set_permissions(&p, fs::Permissions::from_mode(mode));
        }
    }

    fn run_trash(&mut self, job: &Job) {
        // gvfs mounts have no trash (see smb::on_gvfs): the only sane
        // "trash" there is a delete, which the UI confirms as such.
        if job.sources.iter().any(|s| smb::on_gvfs(s)) {
            return self.run_delete(job);
        }
        self.total_items = job.sources.len();
        for (i, src) in job.sources.iter().enumerate() {
            if self.cancelled() {
                break;
            }
            self.report(src, i == 0);
            match trash::delete(src) {
                Ok(()) => self.trashed.push(src.clone()),
                Err(e) => self.error(format!("{}: {e}", src.display())),
            }
            self.done_items += 1;
        }
    }

    fn run_delete(&mut self, job: &Job) {
        // Share trees are listed and removed over SMB directly: one round
        // trip per entry instead of a FUSE hop plus a round trip for every
        // stat and unlink.
        let mut clients: HashMap<String, Option<smb_io::Client>> = HashMap::new();
        let mut direct: Vec<(String, Vec<smb_io::Node>)> = Vec::new();
        let mut local = Vec::new();
        for src in &job.sources {
            if self.cancelled() {
                break;
            }
            let Some(r) = smb_io::remote(src) else {
                local.push(src.clone());
                continue;
            };
            let Some(c) = clients.entry(r.key.clone()).or_insert_with(|| connect_logged(&r)) else {
                local.push(src.clone());
                continue;
            };
            let dir = match fs::symlink_metadata(src) {
                Ok(m) => m.is_dir(),
                Err(e) => {
                    self.fail(src, e);
                    continue;
                }
            };
            let root = smb_io::Node { uri: r.uri, path: src.clone(), dir };
            let mut errors = Vec::new();
            let nodes = c.list_tree(root, &self.cancel, &mut |p, e| errors.push((p.to_path_buf(), e)));
            for (p, e) in errors {
                self.fail(&p, e);
            }
            self.total_items += nodes.len();
            direct.push((r.key, nodes));
        }
        self.scan(&local, false);
        for (key, nodes) in direct {
            let c = clients[&key].as_ref().unwrap();
            for n in nodes {
                if self.cancelled() {
                    return;
                }
                let res = if n.dir { c.rmdir(&n.uri) } else { c.unlink(&n.uri) };
                match res {
                    Ok(()) => self.done_items += 1,
                    Err(e) => self.fail(&n.path, e),
                }
                self.report(&n.path, false);
            }
        }
        for src in &local {
            if self.cancelled() {
                break;
            }
            self.delete_tree(src);
        }
    }

    /// Deletes as much of the tree as possible, recording failures as it goes.
    fn delete_tree(&mut self, p: &Path) {
        let m = match fs::symlink_metadata(p) {
            Ok(m) => m,
            Err(e) => return self.fail(p, e),
        };
        let res = if m.is_dir() {
            match fs::read_dir(p) {
                Ok(rd) => {
                    for d in rd.flatten() {
                        if self.cancelled() {
                            return;
                        }
                        self.delete_tree(&d.path());
                    }
                    fs::remove_dir(p)
                }
                Err(e) => Err(e),
            }
        } else {
            fs::remove_file(p)
        };
        if let Err(e) = res {
            return self.fail(p, e);
        }
        self.done_items += 1;
        self.report(p, false);
    }

    fn run_empty_trash(&mut self) {
        let items = crate::trashdir::list_items();
        self.total_items = items.len();
        self.report(Path::new("Trash"), true);
        if let Err(e) = trash::os_limited::purge_all(items) {
            self.error(e.to_string());
        }
        self.done_items = self.total_items;
    }
}

impl Worker {
    fn run_extract(&mut self, job: &Job) {
        let sizes: Vec<u64> = job
            .sources
            .iter()
            .map(|p| fs::metadata(p).map(|m| m.len()).unwrap_or(0))
            .collect();
        self.total_bytes = sizes.iter().sum();
        self.total_items = job.sources.len();
        for (src, size) in job.sources.iter().zip(sizes) {
            if self.cancelled() {
                break;
            }
            self.report(src, true);
            let base = self.done_bytes;
            self.extract_one(src, size);
            self.done_bytes = base + size;
            self.done_items += 1;
        }
    }

    /// Extracts into a hidden folder beside the archive, then moves the result
    /// into place through the normal conflict handling, so a failed or
    /// cancelled extraction never leaves partial files among the user's.
    fn extract_one(&mut self, archive: &Path, size: u64) {
        let name = archive.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let stem = archive::stem(&name).unwrap_or(&name).to_owned();
        let dest = archive.parent().unwrap_or(Path::new("/"));
        let tmp = dest.join(format!(".{stem}.3la-extract-{}", std::process::id()));
        if let Err(e) = fs::create_dir(&tmp) {
            return self.fail(&tmp, e);
        }
        // A fixed passphrase makes encrypted archives fail at once instead of
        // bsdtar retrying its (unanswerable) prompt many times.
        let args: Vec<OsString> = vec![
            "--passphrase".into(),
            "\u{1}".into(),
            "-xvf".into(),
            archive.into(),
            "-C".into(),
            tmp.clone().into(),
        ];
        match self.run_bsdtar(&args, size) {
            Ok(()) => self.settle(&tmp, dest, &stem),
            Err(e) if !e.is_empty() => self.error(format!("{}: {e}", archive.display())),
            Err(_) => {}
        }
        let _ = remove_any(&tmp);
    }

    /// Moves extracted content from `tmp` into `dest`. A lone top-level entry
    /// is kept as is for Extract Here, and reused as the folder for Extract
    /// To when it already carries the archive's name; anything else is
    /// wrapped in a folder named after the archive.
    fn settle(&mut self, tmp: &Path, dest: &Path, stem: &str) {
        let entries: Vec<PathBuf> = match fs::read_dir(tmp) {
            Ok(rd) => rd.flatten().map(|d| d.path()).collect(),
            Err(e) => return self.fail(tmp, e),
        };
        let src = match entries.as_slice() {
            [one] if self.kind == Kind::ExtractHere => one.clone(),
            [one]
                if one.file_name().is_some_and(|n| n == stem)
                    && fs::symlink_metadata(one).is_ok_and(|m| m.is_dir()) =>
            {
                one.clone()
            }
            _ => tmp.to_path_buf(),
        };
        let target = match src.file_name() {
            Some(n) if src != tmp => dest.join(n),
            _ => dest.join(stem),
        };
        // Moving into place renames whole trees; progress already counted the archive.
        let (items, bytes) = (self.done_items, self.done_bytes);
        let res = self.resolve(&src, target);
        if let Some((target, is_new)) = self.place(Op::Move, &src, res) {
            if is_new {
                self.fresh.push(target.clone());
            }
            self.created.push(target);
        }
        (self.done_items, self.done_bytes) = (items, bytes);
    }

    fn run_compress(&mut self, job: &Job) {
        let target = &job.dest;
        let Some(flags) = archive::create_flags(target) else {
            return self.error(format!("{}: unsupported archive format", target.display()));
        };
        if fs::symlink_metadata(target).is_ok() {
            return self.error(format!("\"{}\" already exists", target.display()));
        }
        self.scan(&job.sources, true);
        let tmp = partial_path(target);
        let mut args: Vec<OsString> = flags.iter().map(OsString::from).collect();
        args.extend(["-cvf".into(), tmp.clone().into()]);
        for src in &job.sources {
            let (Some(parent), Some(name)) = (src.parent(), src.file_name()) else { continue };
            args.extend(["-C".into(), parent.into()]);
            // bsdtar reads "-x" as an option and "@x" as "add the entries of archive x".
            let mut entry = OsString::new();
            if name.as_encoded_bytes().first().is_some_and(|b| *b == b'-' || *b == b'@') {
                entry.push("./");
            }
            entry.push(name);
            args.push(entry);
        }
        let total = self.total_bytes;
        match self.run_bsdtar(&args, total) {
            Ok(()) => {
                let target = if fs::symlink_metadata(target).is_ok() {
                    let dir = target.parent().unwrap_or(Path::new("/"));
                    fsutil::unique_name(dir, &target.file_name().unwrap_or_default().to_string_lossy(), "")
                } else {
                    target.clone()
                };
                match fs::rename(&tmp, &target) {
                    Ok(()) => {
                        self.fresh.push(target.clone());
                        self.created.push(target);
                        return;
                    }
                    Err(e) => self.fail(&target, e),
                }
            }
            Err(e) if !e.is_empty() => self.error(format!("{}: {e}", target.display())),
            Err(_) => {}
        }
        let _ = fs::remove_file(&tmp);
    }

    /// Runs bsdtar with `-v`, tracking its progress through the bytes it has
    /// read (at most `span` past the current `done_bytes`). Err("") means cancelled.
    fn run_bsdtar(&mut self, args: &[OsString], span: u64) -> Result<(), String> {
        let mut cmd = Command::new("bsdtar");
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        // SAFETY: setsid is async-signal-safe. Without a controlling terminal
        // bsdtar can't block on a /dev/tty passphrase prompt.
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let mut child = cmd.spawn().map_err(|e| format!("cannot run bsdtar: {e}"))?;
        let stderr = child.stderr.take().expect("stderr is piped");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).split(b'\n').map_while(Result::ok) {
                if tx.send(String::from_utf8_lossy(&line).into_owned()).is_err() {
                    break;
                }
            }
        });
        let pid = child.id();
        let base = self.done_bytes;
        let mut current = String::new();
        let mut problems: Vec<String> = Vec::new();
        let mut note = |msg: &str| {
            let msg = msg.trim_end_matches(": Unknown error -1");
            if problems.len() < 8 && !msg.ends_with("Error exit delayed from previous errors") {
                problems.push(msg.trim_start_matches("bsdtar: ").to_owned());
            }
        };
        let mut cancelled = false;
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(line) => {
                    if let Some(entry) = line.strip_prefix("x ").or_else(|| line.strip_prefix("a ")) {
                        // A failed entry gets its error appended: "x name: reason".
                        if entry.contains(": ") {
                            note(entry);
                        }
                        current = entry.to_owned();
                        if self.kind == Kind::Compress {
                            self.done_items += 1;
                        }
                    } else {
                        note(&line);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if self.cancelled() {
                let _ = child.kill();
                cancelled = true;
                break;
            }
            if let Some(n) = archive::bytes_read(pid) {
                self.done_bytes = base + n.min(span);
            }
            self.report(Path::new(&current), false);
        }
        let status = child.wait().map_err(|e| e.to_string())?;
        if cancelled {
            return Err(String::new());
        }
        if status.success() {
            return Ok(());
        }
        Err(if problems.iter().any(|p| p.contains("passphrase")) {
            "password-protected archives are not supported".into()
        } else if problems.is_empty() {
            format!("bsdtar failed ({status})")
        } else {
            problems.join("\n")
        })
    }
}

impl Worker {
    fn run_crypt(&mut self, job: &Job) {
        let Some(pass) = &job.secret else { return };
        let encrypt = self.kind == Kind::Encrypt;
        if encrypt {
            // Every byte is read twice: once to encrypt, once to verify.
            self.scan(&job.sources, true);
            self.total_bytes *= 2;
        } else {
            self.total_bytes = job
                .sources
                .iter()
                .map(|p| {
                    let f = if p.is_dir() { p.join(crypt::VAULT) } else { p.clone() };
                    fs::metadata(f).map(|m| m.len()).unwrap_or(0)
                })
                .sum();
        }
        self.total_items = job.sources.len();
        for src in &job.sources {
            if self.cancelled() {
                break;
            }
            self.report(src, true);
            let dir = fs::symlink_metadata(src).is_ok_and(|m| m.is_dir());
            let res = match (encrypt, dir) {
                (true, true) => self.seal_dir(src, pass),
                (true, false) => self.seal_file(src, pass),
                (false, true) => self.open_dir(src, pass),
                (false, false) => self.open_file(src, pass),
            };
            match res {
                Ok(p) => {
                    self.fresh.push(p.clone());
                    self.created.push(p);
                }
                Err(CryptError::Cancelled) => {}
                Err(CryptError::WrongPassword) => self.rejected.push(src.clone()),
                Err(e) => self.error(format!("{}: {e}", src.display())),
            }
            self.done_items += 1;
        }
    }

    /// Progress hook for the crypt helpers: advances `done_bytes` (up to `cap`).
    fn crypt_tick(&mut self, at: &Path, n: u64, cap: u64) -> bool {
        self.done_bytes = (self.done_bytes + n).min(cap);
        self.report(at, false);
        !self.cancelled()
    }

    /// Writes and verifies the vault first; the folder's original contents
    /// are deleted only once the vault is complete and readable.
    fn seal_dir(&mut self, dir: &Path, pass: &SecretString) -> Result<PathBuf, CryptError> {
        let vault = dir.join(crypt::VAULT);
        if fs::symlink_metadata(&vault).is_ok() {
            return Err(CryptError::Io("folder is already encrypted".into()));
        }
        let size = self.totals.get(dir).map_or(0, |t| t.1);
        let base = self.done_bytes;
        let tmp = dir.join(format!(".3la-vault.partial-{}", std::process::id()));
        let res = (|| -> Result<(), CryptError> {
            let half = base + size;
            crypt::encrypt_dir(dir, pass, &tmp, &mut |n| self.crypt_tick(dir, n, half))?;
            self.done_bytes = half;
            crypt::verify(&tmp, pass, true, &mut |n| self.crypt_tick(dir, n, half + size))?;
            fs::rename(&tmp, &vault)?;
            Ok(())
        })();
        if res.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        res?;
        self.done_bytes = base + 2 * size;
        for d in fs::read_dir(dir)?.flatten() {
            if d.file_name().to_string_lossy().starts_with(".3la-") {
                continue;
            }
            if let Err(e) = remove_any(&d.path()) {
                self.error(format!("{}: {e} (left unencrypted)", d.path().display()));
            }
        }
        Ok(dir.to_path_buf())
    }

    fn seal_file(&mut self, src: &Path, pass: &SecretString) -> Result<PathBuf, CryptError> {
        let mut name = src.file_name().unwrap_or_default().to_os_string();
        name.push(".age");
        let out = src.with_file_name(name);
        if fs::symlink_metadata(&out).is_ok() {
            return Err(CryptError::Io(format!("\"{}\" already exists", out.display())));
        }
        let size = self.totals.get(src).map_or(0, |t| t.1);
        let base = self.done_bytes;
        let tmp = partial_path(&out);
        let res = (|| -> Result<(), CryptError> {
            let half = base + size;
            crypt::encrypt_file(src, pass, &tmp, &mut |n| self.crypt_tick(src, n, half))?;
            self.done_bytes = half;
            crypt::verify(&tmp, pass, false, &mut |n| self.crypt_tick(src, n, half + size))?;
            fs::rename(&tmp, &out)?;
            Ok(())
        })();
        if res.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        res?;
        self.done_bytes = base + 2 * size;
        if let Err(e) = fs::remove_file(src) {
            self.error(format!("{}: {e} (original left in place)", src.display()));
        }
        Ok(out)
    }

    /// Unpacks into a hidden folder first, so a wrong password or a damaged
    /// vault leaves the folder untouched.
    fn open_dir(&mut self, dir: &Path, pass: &SecretString) -> Result<PathBuf, CryptError> {
        let vault = dir.join(crypt::VAULT);
        let size = fs::metadata(&vault).map_err(|_| CryptError::Io("folder is not encrypted".into()))?.len();
        let cap = self.done_bytes + size;
        let tmp = dir.join(format!(".3la-unlock-{}", std::process::id()));
        fs::create_dir(&tmp)?;
        if let Err(e) = crypt::decrypt_dir(&vault, pass, &tmp, &mut |n| self.crypt_tick(dir, n, cap)) {
            let _ = fs::remove_dir_all(&tmp);
            return Err(e);
        }
        for d in fs::read_dir(&tmp)?.flatten() {
            let name = d.file_name();
            let mut target = dir.join(&name);
            if fs::symlink_metadata(&target).is_ok() {
                target = fsutil::unique_name(dir, &name.to_string_lossy(), "decrypted");
            }
            if let Err(e) = fs::rename(d.path(), &target) {
                self.error(format!("{}: {e}", target.display()));
            }
        }
        if fs::remove_dir(&tmp).is_ok() {
            fs::remove_file(&vault)?;
        } else {
            self.error(format!("{}: not every item could be restored; the vault was kept", dir.display()));
        }
        self.done_bytes = cap;
        Ok(dir.to_path_buf())
    }

    fn open_file(&mut self, src: &Path, pass: &SecretString) -> Result<PathBuf, CryptError> {
        let name = src.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let stem = match name.len().checked_sub(4) {
            Some(n) if n > 0 && name[n..].eq_ignore_ascii_case(".age") => &name[..n],
            _ => return Err(CryptError::NotEncrypted),
        };
        let dir = src.parent().unwrap_or(Path::new("/"));
        let mut out = dir.join(stem);
        if fs::symlink_metadata(&out).is_ok() {
            out = fsutil::unique_name(dir, stem, "decrypted");
        }
        let cap = self.done_bytes + fs::metadata(src)?.len();
        let tmp = partial_path(&out);
        if let Err(e) = crypt::decrypt_file(src, pass, &tmp, &mut |n| self.crypt_tick(src, n, cap)) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
        fs::rename(&tmp, &out)?;
        if let Err(e) = fs::remove_file(src) {
            self.error(format!("{}: {e}", src.display()));
        }
        self.done_bytes = cap;
        Ok(out)
    }
}

/// Same file or directory (dev + inode), following symlinks.
fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(x), Ok(y)) => x.dev() == y.dev() && x.ino() == y.ino(),
        _ => false,
    }
}

/// Whether `dest` is `src` or lies beneath it, seen through symlinks in either path.
fn inside(dest: &Path, src: &Path) -> bool {
    let canon = |p: &Path| fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let src = match (src.parent(), src.file_name()) {
        (Some(parent), Some(name)) => canon(parent).join(name),
        _ => src.to_path_buf(),
    };
    canon(dest).starts_with(src)
}

fn partial_path(target: &Path) -> PathBuf {
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    target.with_file_name(format!(".{name}.3la-partial-{}", std::process::id()))
}

/// Parallel copies into a remote destination, where per-file round trips
/// rather than bandwidth set the pace.
const POOL_THREADS: usize = 8;

struct Task {
    src: PathBuf,
    dst: PathBuf,
    meta: fs::Metadata,
    /// Move: delete the source once the copy is complete.
    remove_src: bool,
    /// Replace: `dst` is a partial name, renamed to this once complete.
    swap_to: Option<PathBuf>,
}

struct Pool {
    tx: mpsc::SyncSender<Task>,
    done: Receiver<(PathBuf, PathBuf, io::Result<()>)>,
    /// Bytes copied by the pool since the worker last looked.
    bytes: Arc<AtomicU64>,
    pending: usize,
    /// Latest file handed out, for the progress title.
    current: PathBuf,
}

impl Pool {
    fn new(cancel: &Arc<AtomicBool>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<Task>(POOL_THREADS * 2);
        let (done_tx, done) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let bytes = Arc::new(AtomicU64::new(0));
        for _ in 0..POOL_THREADS {
            let (rx, done_tx, bytes, cancel) = (rx.clone(), done_tx.clone(), bytes.clone(), cancel.clone());
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                // One SMB connection per share, or None once it was refused.
                let mut clients: HashMap<String, Option<smb_io::Client>> = HashMap::new();
                loop {
                    let Ok(t) = rx.lock().unwrap().recv() else { return };
                    // Reported under the final name, so a failure prunes it from `created`.
                    let reported = t.swap_to.clone().unwrap_or_else(|| t.dst.clone());
                    let res = if cancel.load(Ordering::SeqCst) {
                        Err(io::ErrorKind::Interrupted.into())
                    } else {
                        let mut progress = |n| {
                            bytes.fetch_add(n, Ordering::Relaxed);
                        };
                        let copied = transfer_file(&t, &mut clients, &cancel, &mut buf, &mut progress);
                        let swapped = copied.and_then(|()| match &t.swap_to {
                            Some(target) => swap_into(&t.dst, target, &clients).map_err(|e| {
                                let _ = remove_direct(&t.dst, &clients);
                                e
                            }),
                            None => Ok(()),
                        });
                        swapped.and_then(|()| if t.remove_src { remove_direct(&t.src, &clients) } else { Ok(()) })
                    };
                    let _ = done_tx.send((t.src, reported, res));
                }
            });
        }
        Self { tx, done, bytes, pending: 0, current: PathBuf::new() }
    }
}

/// Copies one file, straight over SMB when exactly one side is on a gvfs
/// smb-share mount and the share accepts the app's credentials.
fn transfer_file(
    t: &Task,
    clients: &mut HashMap<String, Option<smb_io::Client>>,
    cancel: &AtomicBool,
    buf: &mut Vec<u8>,
    progress: &mut dyn FnMut(u64),
) -> io::Result<()> {
    let direct = match (smb_io::remote(&t.src), smb_io::remote(&t.dst)) {
        (None, Some(r)) => Some((r, true)),
        (Some(r), None) => Some((r, false)),
        _ => None,
    };
    if let Some((r, upload)) = direct {
        let client = clients.entry(r.key.clone()).or_insert_with(|| connect_logged(&r));
        if let Some(c) = client {
            return if upload {
                c.upload(&t.src, &r.uri, &t.meta, cancel, buf, progress)
            } else {
                c.download(&r.uri, &t.dst, &t.meta, cancel, buf, progress)
            };
        }
    }
    copy_file(&t.src, &t.dst, &t.meta, cancel, buf, progress)
}

/// Opens a direct SMB session for `r`, saying so on stderr when it falls
/// back to gvfs (several times slower) for want of accepted credentials.
fn connect_logged(r: &smb_io::Remote) -> Option<smb_io::Client> {
    let c = smb_io::Client::connect(r);
    if c.is_none() {
        eprintln!("3la-fileexplorer: no direct SMB session for //{}/{} (no accepted credentials), using gvfs", r.host, r.share);
    }
    c
}

/// Removes a file, over SMB when a session to its share is open.
fn remove_direct(p: &Path, clients: &HashMap<String, Option<smb_io::Client>>) -> io::Result<()> {
    if let Some(r) = smb_io::remote(p) {
        if let Some(Some(c)) = clients.get(&r.key) {
            return c.unlink(&r.uri);
        }
    }
    fs::remove_file(p)
}

/// Renames the finished copy `tmp` over `target`, over SMB when both are on
/// a share with an open session (the gvfs view can lag behind direct writes).
fn swap_into(tmp: &Path, target: &Path, clients: &HashMap<String, Option<smb_io::Client>>) -> io::Result<()> {
    if let (Some(a), Some(b)) = (smb_io::remote(tmp), smb_io::remote(target)) {
        if let Some(Some(c)) = clients.get(&a.key) {
            if a.key == b.key {
                return c.rename(&a.uri, &b.uri);
            }
        }
    }
    fs::rename(tmp, target)
}

fn copy_file(
    src: &Path,
    dst: &Path,
    meta: &fs::Metadata,
    cancel: &AtomicBool,
    buf: &mut Vec<u8>,
    progress: &mut dyn FnMut(u64),
) -> io::Result<()> {
    let mut input = File::open(src)?;
    // Created with the source mode so a private file is never briefly readable.
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(meta.mode() & 0o777)
        .open(dst)?;
    if let Err(e) = pump(&mut input, &mut output, cancel, buf, progress) {
        drop(output);
        let _ = fs::remove_file(dst);
        return Err(e);
    }
    best_effort(output.set_permissions(fs::Permissions::from_mode(meta.mode())))?;
    let times = FileTimes::new()
        .set_accessed(meta.accessed().unwrap_or(UNIX_EPOCH))
        .set_modified(meta.modified().unwrap_or(UNIX_EPOCH));
    let _ = output.set_times(times);
    Ok(())
}

/// Copies file contents, in-kernel via copy_file_range (reflinks / server-side
/// copy where supported), falling back to a userspace buffer.
fn pump(
    input: &mut File,
    output: &mut File,
    cancel: &AtomicBool,
    buf: &mut Vec<u8>,
    progress: &mut dyn FnMut(u64),
) -> io::Result<()> {
    let mut kernel = true;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let n = if kernel {
            // SAFETY: both fds are open for the duration of the call; null
            // offsets use and advance the file positions.
            let r = unsafe {
                libc::copy_file_range(
                    input.as_raw_fd(),
                    std::ptr::null_mut(),
                    output.as_raw_fd(),
                    std::ptr::null_mut(),
                    COPY_CHUNK,
                    0,
                )
            };
            if r < 0 {
                let e = io::Error::last_os_error();
                match e.raw_os_error() {
                    Some(libc::EXDEV | libc::ENOSYS | libc::EINVAL | libc::EOPNOTSUPP | libc::EPERM) => {
                        kernel = false;
                        continue;
                    }
                    _ => return Err(e),
                }
            }
            r as usize
        } else {
            if buf.is_empty() {
                *buf = vec![0u8; 1 << 20];
            }
            let n = input.read(buf)?;
            output.write_all(&buf[..n])?;
            n
        };
        if n == 0 {
            return Ok(());
        }
        progress(n as u64);
    }
}

/// Metadata the destination filesystem can't store (gvfs FUSE, vfat) is not a copy failure.
fn best_effort(r: io::Result<()>) -> io::Result<()> {
    match r {
        Err(e) if matches!(e.raw_os_error(), Some(libc::EOPNOTSUPP | libc::EPERM | libc::ENOSYS)) => Ok(()),
        other => other,
    }
}

fn remove_any(p: &Path) -> io::Result<()> {
    if fs::symlink_metadata(p)?.is_dir() {
        fs::remove_dir_all(p)
    } else {
        fs::remove_file(p)
    }
}

/// (items, bytes) of a tree in one walk.
fn tree_totals(p: &Path) -> (usize, u64) {
    let (mut n, mut b) = (0, 0);
    let mut stack = vec![p.to_path_buf()];
    while let Some(p) = stack.pop() {
        let Ok(m) = fs::symlink_metadata(&p) else { continue };
        n += 1;
        if m.is_dir() {
            if let Ok(rd) = fs::read_dir(&p) {
                stack.extend(rd.flatten().map(|d| d.path()));
            }
        } else if m.is_file() {
            b += m.len();
        }
    }
    (n, b)
}

fn fmt_eta(secs: f64) -> String {
    let s = secs.round() as u64;
    if s >= 3600 {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symlinked_paths_are_detected() {
        let root = std::env::temp_dir().join(format!("3la-ops-{}", std::process::id()));
        let src = root.join("src");
        fs::create_dir_all(src.join("sub")).unwrap();
        symlink(&src, root.join("alias")).unwrap();

        assert!(same_file(&src, &root.join("alias")));
        assert!(!same_file(&src, &root));
        assert!(inside(&root.join("alias/sub"), &src));
        assert!(inside(&src, &src));
        assert!(!inside(&root, &src));
        // Moving the link itself into its target is legitimate.
        assert!(!inside(&src, &root.join("alias")));

        let p = partial_path(&root.join("a.txt"));
        assert!(p.file_name().unwrap().to_string_lossy().starts_with(".a.txt.3la-partial-"));
        fs::remove_dir_all(&root).unwrap();
    }
}
