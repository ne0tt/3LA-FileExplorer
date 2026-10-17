#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, path)]
        #[qproperty(bool, show_hidden, cxx_name = "showHidden")]
        #[qproperty(i32, sort_key, cxx_name = "sortKey")]
        #[qproperty(bool, sort_descending, cxx_name = "sortDescending")]
        #[qproperty(bool, folders_first, cxx_name = "foldersFirst")]
        #[qproperty(bool, count_children, cxx_name = "countChildren")]
        #[qproperty(QString, filter)]
        #[qproperty(i32, count)]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error)]
        #[qproperty(bool, is_trash, cxx_name = "isTrash")]
        /// network:/// or smb://host/ — synthetic listings, not a real folder.
        #[qproperty(bool, is_virtual, cxx_name = "isVirtual")]
        /// The folder's contents are encrypted (it holds a crypt vault).
        #[qproperty(bool, locked)]
        type FolderModel = super::FolderModelRust;

        #[qsignal]
        fn loaded(self: Pin<&mut FolderModel>, reset: bool);

        /// A background trash restore/purge failed.
        #[qsignal]
        #[cxx_name = "operationFailed"]
        fn operation_failed(self: Pin<&mut FolderModel>, message: QString);

        #[qinvokable]
        fn reload(self: Pin<&mut FolderModel>);

        #[qinvokable]
        #[cxx_name = "pathAt"]
        fn path_at(self: &FolderModel, row: i32) -> QString;

        #[qinvokable]
        #[cxx_name = "isDirAt"]
        fn is_dir_at(self: &FolderModel, row: i32) -> bool;

        #[qinvokable]
        #[cxx_name = "mimeAt"]
        fn mime_at(self: &FolderModel, row: i32) -> QString;

        /// Re-emit all rows (e.g. after the date format changed).
        #[qinvokable]
        #[cxx_name = "refreshDisplay"]
        fn refresh_display(self: Pin<&mut FolderModel>);

        #[qinvokable]
        #[cxx_name = "origPathOf"]
        fn orig_path_of(self: &FolderModel, path: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "rowOf"]
        fn row_of(self: &FolderModel, path: &QString) -> i32;

        #[qinvokable]
        #[cxx_name = "findPrefix"]
        fn find_prefix(self: &FolderModel, prefix: &QString, start: i32) -> i32;

        #[qinvokable]
        #[cxx_name = "allPaths"]
        fn all_paths(self: &FolderModel) -> QStringList;

        #[qinvokable]
        #[cxx_name = "pathsMatching"]
        fn paths_matching(self: &FolderModel, pattern: &QString) -> QStringList;

        /// The given paths that are in the view, in view order.
        #[qinvokable]
        fn ordered(self: &FolderModel, paths: &QStringList) -> QStringList;

        /// JSON [{path, name, video}] of the pictures and videos in view order.
        #[qinvokable]
        #[cxx_name = "mediaItems"]
        fn media_items(self: &FolderModel) -> QString;

        #[qinvokable]
        fn summary(self: &FolderModel, selected: &QStringList) -> QString;

        /// Runs off-thread; failures arrive as operationFailed.
        #[qinvokable]
        #[cxx_name = "restoreFromTrash"]
        fn restore_from_trash(self: Pin<&mut FolderModel>, paths: &QStringList);

        #[qinvokable]
        #[cxx_name = "purgeFromTrash"]
        fn purge_from_trash(self: Pin<&mut FolderModel>, paths: &QStringList);
    }

    impl cxx_qt::Threading for FolderModel {}
    impl cxx_qt::Initialize for FolderModel {}

    extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(
            self: Pin<&mut FolderModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        unsafe fn begin_remove_rows(
            self: Pin<&mut FolderModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        unsafe fn end_remove_rows(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut FolderModel>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut FolderModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[inherit]
        fn index(
            self: &FolderModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    extern "RustQt" {
        #[cxx_override]
        fn data(self: &FolderModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &FolderModel) -> QHash_i32_QByteArray;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &FolderModel, parent: &QModelIndex) -> i32;
    }
}

use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cxx_qt::{CxxQtType, Initialize, Threading};
use cxx_qt_lib::{
    QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QStringList, QVariant,
    QVector,
};
use notify::event::{AccessKind, AccessMode, ModifyKind};
use notify::{EventKind, RecursiveMode, Watcher};

use crate::fsutil::{self, Entry, SortKey, SortSpec, TrashInfo};
use crate::smb;

pub const TRASH_URI: &str = "trash:///";

#[derive(Clone, Copy)]
enum Role {
    Name,
    Path,
    IsDir,
    IsLink,
    IsHidden,
    Size,
    SizeText,
    Mtime,
    ModifiedText,
    TypeText,
    Mime,
    Glyph,
    Category,
    Permissions,
    Owner,
    Group,
    IsImage,
    IsExec,
    LinkTarget,
    BrokenLink,
    OrigPath,
    DeletedText,
}

const ROLES: &[(Role, &str)] = &[
    (Role::Name, "name"),
    (Role::Path, "path"),
    (Role::IsDir, "isDir"),
    (Role::IsLink, "isLink"),
    (Role::IsHidden, "isHidden"),
    (Role::Size, "size"),
    (Role::SizeText, "sizeText"),
    (Role::Mtime, "mtime"),
    (Role::ModifiedText, "modifiedText"),
    (Role::TypeText, "typeText"),
    (Role::Mime, "mime"),
    (Role::Glyph, "glyph"),
    (Role::Category, "category"),
    (Role::Permissions, "permissions"),
    (Role::Owner, "owner"),
    (Role::Group, "group"),
    (Role::IsImage, "isImage"),
    (Role::IsExec, "isExec"),
    (Role::LinkTarget, "linkTarget"),
    (Role::BrokenLink, "brokenLink"),
    (Role::OrigPath, "origPath"),
    (Role::DeletedText, "deletedText"),
];
const ROLE_BASE: i32 = 0x0100;

/// Watcher events are coalesced until the folder has been quiet this long…
const QUIET: Duration = Duration::from_millis(200);
/// …but a constantly changing folder still refreshes this often.
const MAX_WAIT: Duration = Duration::from_secs(1);
/// More row groups than this in one update and a model reset is cheaper.
const MAX_DIFF_OPS: usize = 256;

pub struct FolderModelRust {
    path: QString,
    show_hidden: bool,
    sort_key: i32,
    sort_descending: bool,
    folders_first: bool,
    count_children: bool,
    filter: QString,
    count: i32,
    loading: bool,
    error: QString,
    is_trash: bool,
    is_virtual: bool,
    locked: bool,

    /// Entries are shared between the raw listing and the visible rows.
    all: Vec<Arc<Entry>>,
    entries: Vec<Arc<Entry>>,
    /// Path hash -> visible row, for rowOf.
    rows: HashMap<u64, u32>,
    trash_items: Vec<trash::TrashItem>,
    free_space: Option<u64>,
    generation: Arc<AtomicU64>,
    /// Generation of the load still running, if any; reloads requested
    /// meanwhile set `dirty` and run once it finishes.
    in_flight: Option<u64>,
    dirty: bool,
    watcher: Option<notify::RecommendedWatcher>,
}

impl Default for FolderModelRust {
    fn default() -> Self {
        Self {
            path: QString::default(),
            show_hidden: false,
            sort_key: 0,
            sort_descending: false,
            folders_first: true,
            count_children: true,
            filter: QString::default(),
            count: 0,
            loading: false,
            error: QString::default(),
            is_trash: false,
            is_virtual: false,
            locked: false,
            all: Vec::new(),
            entries: Vec::new(),
            rows: HashMap::new(),
            trash_items: Vec::new(),
            free_space: None,
            generation: Arc::new(AtomicU64::new(0)),
            in_flight: None,
            dirty: false,
            watcher: None,
        }
    }
}

impl Initialize for qobject::FolderModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_path_changed(|m| m.navigate())
            .release();
        self.as_mut()
            .on_show_hidden_changed(|m| m.refilter(false))
            .release();
        self.as_mut()
            .on_filter_changed(|m| m.refilter(false))
            .release();
        self.as_mut()
            .on_sort_key_changed(|m| m.refilter(true))
            .release();
        self.as_mut()
            .on_sort_descending_changed(|m| m.refilter(true))
            .release();
        self.as_mut()
            .on_folders_first_changed(|m| m.refilter(true))
            .release();
        self.as_mut()
            .on_count_children_changed(|m| m.reload())
            .release();
    }
}

fn trash_files_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| fsutil::home().join(".local/share"))
        .join("Trash/files")
}

fn path_key(p: &str) -> u64 {
    let mut h = DefaultHasher::new();
    p.hash(&mut h);
    h.finish()
}

fn qs(s: &str) -> QString {
    QString::from(s)
}

/// One step of turning the visible rows into the next list, with positions
/// in the list as it stands after the previous steps.
enum DiffOp {
    Remove { at: usize, len: usize },
    Insert { at: usize, from: Range<usize> },
    Change { at: usize, from: Range<usize> },
}

/// Merge-walk two lists sorted by the same comparator, grouping consecutive
/// inserts, removals and changes so each group is a single model signal.
fn diff(old: &[Arc<Entry>], new: &[Arc<Entry>], spec: SortSpec) -> Vec<DiffOp> {
    let mut ops = Vec::new();
    let (mut at, mut oi, mut ni) = (0, 0, 0);
    loop {
        match (old.get(oi), new.get(ni)) {
            (None, None) => break,
            (Some(_), None) => {
                ops.push(DiffOp::Remove { at, len: old.len() - oi });
                break;
            }
            (None, Some(_)) => {
                ops.push(DiffOp::Insert { at, from: ni..new.len() });
                break;
            }
            (Some(o), Some(n)) => match fsutil::compare(o, n, spec) {
                Ordering::Equal => {
                    if o.changed_from(n) {
                        match ops.last_mut() {
                            Some(DiffOp::Change { at: start, from }) if *start + from.len() == at => from.end = ni + 1,
                            _ => ops.push(DiffOp::Change { at, from: ni..ni + 1 }),
                        }
                    }
                    (at, oi, ni) = (at + 1, oi + 1, ni + 1);
                }
                Ordering::Less => {
                    let start = oi;
                    while oi < old.len() && fsutil::compare(&old[oi], n, spec) == Ordering::Less {
                        oi += 1;
                    }
                    ops.push(DiffOp::Remove { at, len: oi - start });
                }
                Ordering::Greater => {
                    let start = ni;
                    while ni < new.len() && fsutil::compare(o, &new[ni], spec) == Ordering::Greater {
                        ni += 1;
                    }
                    ops.push(DiffOp::Insert { at, from: start..ni });
                    at += ni - start;
                }
            },
        }
    }
    ops
}

type Listing = Result<(Vec<Entry>, Vec<trash::TrashItem>), String>;

impl qobject::FolderModel {
    fn spec(&self) -> SortSpec {
        SortSpec {
            key: SortKey::from_i32(self.sort_key),
            descending: self.sort_descending,
            folders_first: self.folders_first,
        }
    }

    fn navigate(mut self: Pin<&mut Self>) {
        let path = self.path.to_string();
        let trash = path == TRASH_URI;
        let virtual_view = path == smb::NETWORK_URI || path.starts_with("smb://");
        self.as_mut().set_is_trash(trash);
        self.as_mut().set_is_virtual(virtual_view);
        self.as_mut().set_error(QString::default());
        self.as_mut().rust_mut().watcher = None;
        unsafe {
            self.as_mut().begin_reset_model();
            let mut r = self.as_mut().rust_mut();
            r.entries.clear();
            r.all.clear();
            r.rows.clear();
            r.free_space = None;
            self.as_mut().end_reset_model();
        }
        self.as_mut().set_count(0);
        if !virtual_view {
            let watch_dir = if trash {
                trash_files_dir()
            } else {
                PathBuf::from(&path)
            };
            self.as_mut().watch(&watch_dir);
        }
        self.load(true);
    }

    /// Watches `dir`, coalescing bursts of events into one reload: after
    /// QUIET without events, or MAX_WAIT into a continuous burst. At most one
    /// debounce thread exists at a time.
    fn watch(mut self: Pin<&mut Self>, dir: &Path) {
        let qt = self.qt_thread();
        let burst: Arc<Mutex<Option<(Instant, Instant)>>> = Arc::default();
        let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(ev) = res else { return };
            // Content writes stream in for as long as a file is being copied
            // (and each reload competes with it on a network mount); the close
            // that ends them is enough to pick up the new size.
            match ev.kind {
                EventKind::Access(AccessKind::Close(AccessMode::Write)) => {}
                EventKind::Access(_) | EventKind::Modify(ModifyKind::Data(_)) => return,
                _ => {}
            }
            let now = Instant::now();
            {
                let mut b = burst.lock().unwrap();
                if let Some((_, last)) = b.as_mut() {
                    *last = now;
                    return;
                }
                *b = Some((now, now));
            }
            let (qt, burst) = (qt.clone(), burst.clone());
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(50));
                let mut b = burst.lock().unwrap();
                let Some((first, last)) = *b else { return };
                if last.elapsed() >= QUIET || first.elapsed() >= MAX_WAIT {
                    *b = None;
                    drop(b);
                    let _ = qt.queue(|m| m.load(false));
                    return;
                }
            });
        });
        match watcher.and_then(|mut w| w.watch(dir, RecursiveMode::NonRecursive).map(|()| w)) {
            Ok(w) => self.as_mut().rust_mut().watcher = Some(w),
            Err(e) => eprintln!("3la-fileexplorer: not watching {}: {e}", dir.display()),
        }
    }

    pub fn reload(self: Pin<&mut Self>) {
        self.load(false);
    }

    /// Lists the folder off-thread. Child counts come in a second pass so the
    /// listing shows first; counts for subfolders whose mtime is unchanged are
    /// carried over instead of re-read.
    fn load(mut self: Pin<&mut Self>, reset: bool) {
        let path = self.path.to_string();
        if path.is_empty() {
            return;
        }
        if !reset && self.in_flight.is_some() {
            self.as_mut().rust_mut().dirty = true;
            return;
        }
        let gen = self.generation.fetch_add(1, AtomicOrdering::SeqCst) + 1;
        self.as_mut().rust_mut().in_flight = Some(gen);
        self.as_mut().rust_mut().dirty = false;
        let gen_ref = self.generation.clone();
        let trash = self.is_trash;
        let count = self.count_children && !trash && !self.is_virtual;
        let known: HashMap<String, (i64, u32)> = if count {
            self.all
                .iter()
                .filter_map(|e| Some((e.path.clone(), (e.mtime, e.child_count?))))
                .collect()
        } else {
            HashMap::new()
        };
        let qt = self.qt_thread();
        self.as_mut().set_loading(true);
        std::thread::spawn(move || {
            let stale = || gen_ref.load(AtomicOrdering::SeqCst) != gen;
            let dir = Path::new(&path);
            let mut free = None;
            let mut result: Listing = if trash {
                Ok(load_trash())
            } else if path == smb::NETWORK_URI {
                Ok((smb::discover_hosts(&smb::bookmark_uris()), Vec::new()))
            } else if let Some(u) = smb::parse(&path) {
                smb::list_shares(&u.host).map(|e| (e, Vec::new()))
            } else {
                free = fsutil::free_space(dir).map(|(f, _)| f);
                fsutil::read_dir(dir, &stale).map(|e| (e, Vec::new()))
            };
            if stale() {
                return;
            }
            let mut pending = Vec::new();
            // Probing every subfolder for a vault costs a round trip each on
            // network mounts, so locked folders are only marked locally.
            let local = !trash && !fsutil::is_remote_fs(dir);
            if let Ok((entries, _)) = &mut result {
                let mut ids = HashSet::new();
                for e in entries.iter_mut() {
                    // Warm the owner caches here, not in data() on the GUI thread.
                    if ids.insert((e.uid, e.gid)) {
                        fsutil::user_name(e.uid);
                        fsutil::group_name(e.gid);
                    }
                    if local && e.is_dir && !e.is_link && Path::new(&e.path).join(crate::crypt::VAULT).is_file() {
                        // Its only entry is the vault; an item count would mislead.
                        e.glyph = crate::icons::SPX_FOLDER_LOCKED;
                        e.type_text = fsutil::intern("Encrypted folder");
                        e.child_count = None;
                        continue;
                    }
                    if !count || !e.is_dir {
                        continue;
                    }
                    match known.get(&e.path) {
                        Some(&(mtime, n)) if mtime == e.mtime => e.child_count = Some(n),
                        _ => pending.push(e.path.clone()),
                    }
                }
            }
            if !local {
                pending.clear();
            }
            let more = !pending.is_empty();
            let _ = qt.queue(move |m| m.finish_load(gen, reset, result, free, more));
            if !more {
                return;
            }
            let mut counts = Vec::with_capacity(pending.len());
            for (i, p) in pending.into_iter().enumerate() {
                if i % 32 == 31 && stale() {
                    return;
                }
                if let Some(n) = fsutil::child_count(Path::new(&p)) {
                    counts.push((p, n));
                }
            }
            let _ = qt.queue(move |m| m.apply_counts(gen, counts));
        });
    }

    fn finish_load(mut self: Pin<&mut Self>, gen: u64, reset: bool, result: Listing, free: Option<u64>, more: bool) {
        if self.generation.load(AtomicOrdering::SeqCst) != gen {
            return;
        }
        if !more {
            self.as_mut().rust_mut().in_flight = None;
        }
        self.as_mut().set_loading(false);
        match result {
            Ok((entries, items)) => {
                self.as_mut().set_error(QString::default());
                let locked = !self.is_trash && entries.iter().any(|e| e.name == crate::crypt::VAULT && !e.is_dir);
                self.as_mut().set_locked(locked);
                let mut r = self.as_mut().rust_mut();
                r.all = entries.into_iter().map(Arc::new).collect();
                r.trash_items = items;
                r.free_space = free;
            }
            Err(e) => {
                self.as_mut().set_locked(false);
                self.as_mut().set_error(QString::from(&e));
                self.as_mut().rust_mut().all.clear();
            }
        }
        let reset = reset || !self.error.is_empty();
        self.as_mut().refilter(reset);
        self.as_mut().loaded(reset);
        if !more {
            self.finish_pending();
        }
    }

    fn apply_counts(mut self: Pin<&mut Self>, gen: u64, counts: Vec<(String, u32)>) {
        if self.generation.load(AtomicOrdering::SeqCst) != gen {
            return;
        }
        self.as_mut().rust_mut().in_flight = None;
        let counts: HashMap<String, u32> = counts.into_iter().collect();
        let all = std::mem::take(&mut self.as_mut().rust_mut().all);
        self.as_mut().rust_mut().all = all
            .into_iter()
            .map(|e| match counts.get(&e.path) {
                Some(&n) if e.child_count != Some(n) => Arc::new(Entry { child_count: Some(n), ..(*e).clone() }),
                _ => e,
            })
            .collect();
        self.as_mut().refilter(false);
        self.finish_pending();
    }

    /// Runs the reload that was requested while the last one was in flight.
    fn finish_pending(mut self: Pin<&mut Self>) {
        if self.dirty {
            self.as_mut().rust_mut().dirty = false;
            self.load(false);
        }
    }

    fn visible(&self) -> Vec<Arc<Entry>> {
        let needle = self.filter.to_string().to_lowercase();
        let mut v: Vec<Arc<Entry>> = self
            .all
            .iter()
            .filter(|e| self.show_hidden || self.is_trash || !e.hidden)
            // The encrypted payload of a locked folder is never listed.
            .filter(|e| e.name != crate::crypt::VAULT)
            .filter(|e| needle.is_empty() || e.name.to_lowercase().contains(&needle))
            .cloned()
            .collect();
        let spec = self.spec();
        v.sort_by(|a, b| fsutil::compare(a, b, spec));
        v
    }

    fn refilter(mut self: Pin<&mut Self>, reset: bool) {
        let next = self.visible();
        let ops = if reset || self.entries.is_empty() {
            None
        } else {
            Some(diff(&self.entries, &next, self.spec())).filter(|ops| ops.len() <= MAX_DIFF_OPS)
        };
        match ops {
            Some(ops) => self.as_mut().apply_diff(ops, &next),
            None => unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().rust_mut().entries = next;
                self.as_mut().end_reset_model();
            },
        }
        let rows = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (path_key(&e.path), i as u32))
            .collect();
        self.as_mut().rust_mut().rows = rows;
        let n = self.entries.len() as i32;
        self.set_count(n);
    }

    /// Applies `diff` output so views keep scroll and selection state.
    fn apply_diff(mut self: Pin<&mut Self>, ops: Vec<DiffOp>, next: &[Arc<Entry>]) {
        let root = QModelIndex::default();
        for op in ops {
            match op {
                DiffOp::Remove { at, len } => unsafe {
                    self.as_mut().begin_remove_rows(&root, at as i32, (at + len) as i32 - 1);
                    self.as_mut().rust_mut().entries.drain(at..at + len);
                    self.as_mut().end_remove_rows();
                },
                DiffOp::Insert { at, from } => unsafe {
                    self.as_mut().begin_insert_rows(&root, at as i32, (at + from.len()) as i32 - 1);
                    self.as_mut().rust_mut().entries.splice(at..at, next[from].iter().cloned());
                    self.as_mut().end_insert_rows();
                },
                DiffOp::Change { at, from } => {
                    let len = from.len();
                    self.as_mut().rust_mut().entries[at..at + len].clone_from_slice(&next[from]);
                    let first = self.index(at as i32, 0, &root);
                    let last = self.index((at + len) as i32 - 1, 0, &root);
                    self.as_mut().data_changed(&first, &last, &QVector::default());
                }
            }
        }
    }

    fn entry(&self, row: i32) -> Option<&Entry> {
        usize::try_from(row).ok().and_then(|r| self.entries.get(r)).map(|e| &**e)
    }

    pub fn path_at(&self, row: i32) -> QString {
        self.entry(row).map(|e| qs(&e.path)).unwrap_or_default()
    }

    pub fn is_dir_at(&self, row: i32) -> bool {
        self.entry(row).is_some_and(|e| e.is_dir)
    }

    pub fn mime_at(&self, row: i32) -> QString {
        self.entry(row).map(|e| qs(&e.mime)).unwrap_or_default()
    }

    pub fn refresh_display(mut self: Pin<&mut Self>) {
        let n = self.entries.len() as i32;
        if n == 0 {
            return;
        }
        let root = QModelIndex::default();
        let first = self.index(0, 0, &root);
        let last = self.index(n - 1, 0, &root);
        self.as_mut().data_changed(&first, &last, &QVector::default());
    }

    pub fn orig_path_of(&self, path: &QString) -> QString {
        let row = self.row_of(path);
        self.entry(row)
            .and_then(|e| e.trash.as_ref())
            .map(|t| qs(&t.orig_path))
            .unwrap_or_default()
    }

    pub fn row_of(&self, path: &QString) -> i32 {
        let p = path.to_string();
        match self.rows.get(&path_key(&p)) {
            Some(&r) if self.entries.get(r as usize).is_some_and(|e| e.path == p) => r as i32,
            // Hash collision (or a stale map): fall back to a scan.
            _ => self.entries.iter().position(|e| e.path == p).map_or(-1, |i| i as i32),
        }
    }

    pub fn find_prefix(&self, prefix: &QString, start: i32) -> i32 {
        let p = prefix.to_string().to_lowercase();
        if p.is_empty() || self.entries.is_empty() {
            return -1;
        }
        let n = self.entries.len();
        let start = start.max(0) as usize % n;
        (0..n)
            .map(|k| (start + k) % n)
            .find(|&i| self.entries[i].name.to_lowercase().starts_with(&p))
            .map_or(-1, |i| i as i32)
    }

    pub fn all_paths(&self) -> QStringList {
        self.entries.iter().map(|e| qs(&e.path)).collect()
    }

    pub fn paths_matching(&self, pattern: &QString) -> QStringList {
        let matches = glob_matcher(&pattern.to_string());
        self.entries
            .iter()
            .filter(|e| matches(&e.name))
            .map(|e| qs(&e.path))
            .collect()
    }

    pub fn ordered(&self, paths: &QStringList) -> QStringList {
        let wanted: HashSet<String> = paths.iter().map(|s| s.to_string()).collect();
        self.entries
            .iter()
            .filter(|e| wanted.contains(&e.path))
            .map(|e| qs(&e.path))
            .collect()
    }

    pub fn media_items(&self) -> QString {
        let v: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.mime.starts_with("image/") || e.mime.starts_with("video/"))
            .map(|e| serde_json::json!({ "path": e.path, "name": e.name, "video": e.mime.starts_with("video/") }))
            .collect();
        qs(&serde_json::Value::Array(v).to_string())
    }

    pub fn summary(&self, selected: &QStringList) -> QString {
        let sel: HashSet<String> = selected.iter().map(|s| s.to_string()).collect();
        if sel.is_empty() {
            let size: u64 = self.entries.iter().map(|e| e.size).sum();
            let n = self.entries.len();
            let mut s = format!(
                "{n} item{}, {}",
                if n == 1 { "" } else { "s" },
                fsutil::format_size(size)
            );
            if let Some(free) = self.free_space.filter(|_| !self.is_trash && !self.is_virtual) {
                s.push_str(&format!(" | Free space: {}", fsutil::format_size(free)));
            }
            return qs(&s);
        }
        let chosen: Vec<&Entry> = self
            .entries
            .iter()
            .filter(|e| sel.contains(&e.path))
            .map(|e| &**e)
            .collect();
        if chosen.len() == 1 {
            let e = chosen[0];
            let s = if e.is_dir {
                match e.child_count {
                    Some(c) => format!("\"{}\": folder, {c} item{}", e.name, if c == 1 { "" } else { "s" }),
                    None => format!("\"{}\": folder", e.name),
                }
            } else if e.is_link {
                format!(
                    "\"{}\": link to {} ({})",
                    e.name,
                    e.link_target,
                    fsutil::format_size(e.size)
                )
            } else {
                format!(
                    "\"{}\": {} ({})",
                    e.name,
                    fsutil::format_size(e.size),
                    e.type_text
                )
            };
            return qs(&s);
        }
        let dirs = chosen.iter().filter(|e| e.is_dir).count();
        let files = chosen.len() - dirs;
        let size: u64 = chosen.iter().map(|e| e.size).sum();
        let mut parts = Vec::new();
        if dirs > 0 {
            parts.push(format!("{dirs} folder{}", if dirs == 1 { "" } else { "s" }));
        }
        if files > 0 {
            parts.push(format!(
                "{files} file{} ({})",
                if files == 1 { "" } else { "s" },
                fsutil::format_size(size)
            ));
        }
        qs(&format!("{} selected", parts.join(", ")))
    }

    fn trash_items_for(&self, paths: &QStringList) -> Vec<trash::TrashItem> {
        let wanted: HashSet<String> = paths.iter().map(|s| s.to_string()).collect();
        self.all
            .iter()
            .zip(self.trash_items.iter())
            .filter(|(e, _)| wanted.contains(&e.path))
            .map(|(_, t)| t.clone())
            .collect()
    }

    /// Restores or purges trash items off the GUI thread (a big folder can
    /// take a while), then reloads.
    fn trash_op(
        self: Pin<&mut Self>,
        paths: &QStringList,
        op: fn(Vec<trash::TrashItem>) -> Result<(), trash::Error>,
    ) {
        let items = self.trash_items_for(paths);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let res = op(items);
            let _ = qt.queue(move |mut m| {
                if let Err(e) = res {
                    m.as_mut().operation_failed(qs(&e.to_string()));
                }
                m.load(false);
            });
        });
    }

    pub fn restore_from_trash(self: Pin<&mut Self>, paths: &QStringList) {
        self.trash_op(paths, trash::os_limited::restore_all);
    }

    pub fn purge_from_trash(self: Pin<&mut Self>, paths: &QStringList) {
        self.trash_op(paths, trash::os_limited::purge_all);
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(e) = self.entry(index.row()) else {
            return QVariant::default();
        };
        let Some(&(role, _)) = usize::try_from(role - ROLE_BASE)
            .ok()
            .and_then(|r| ROLES.get(r))
        else {
            return QVariant::default();
        };
        let s = |v: &str| QVariant::from(&QString::from(v));
        match role {
            Role::Name => s(&e.name),
            Role::Path => s(&e.path),
            Role::IsDir => QVariant::from(&e.is_dir),
            Role::IsLink => QVariant::from(&e.is_link),
            Role::IsHidden => QVariant::from(&e.hidden),
            Role::Size => QVariant::from(&(e.size as i64)),
            Role::SizeText => s(&if e.is_dir {
                match e.child_count {
                    Some(c) => format!("{c} item{}", if c == 1 { "" } else { "s" }),
                    None => "—".into(),
                }
            } else {
                fsutil::format_size(e.size)
            }),
            Role::Mtime => QVariant::from(&e.mtime),
            Role::ModifiedText => s(&fsutil::format_time(e.mtime)),
            Role::TypeText => s(&e.type_text),
            Role::Mime => s(&e.mime),
            Role::Glyph => s(e.glyph),
            Role::Category => s(e.category),
            Role::Permissions => s(&fsutil::perm_string(e.mode, e.is_dir, e.is_link)),
            Role::Owner => s(&fsutil::user_name(e.uid)),
            Role::Group => s(&fsutil::group_name(e.gid)),
            Role::IsImage => QVariant::from(&e.is_image()),
            Role::IsExec => QVariant::from(&e.is_exec()),
            Role::LinkTarget => s(&e.link_target),
            Role::BrokenLink => QVariant::from(&e.broken_link),
            Role::OrigPath => s(e.trash.as_ref().map_or("", |t| &t.orig_path)),
            Role::DeletedText => s(&e
                .trash
                .as_ref()
                .filter(|t| t.deleted > 0)
                .map(|t| fsutil::format_time(t.deleted))
                .unwrap_or_default()),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (i, (_, name)) in ROLES.iter().enumerate() {
            roles.insert(ROLE_BASE + i as i32, QByteArray::from(*name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.entries.len() as i32
    }
}

fn load_trash() -> (Vec<Entry>, Vec<trash::TrashItem>) {
    let items = crate::trashdir::list_items();
    let mut entries = Vec::with_capacity(items.len());
    let mut kept = Vec::with_capacity(items.len());
    for item in items {
        let info = PathBuf::from(&item.id);
        let file = info
            .parent()
            .and_then(Path::parent)
            .map(|trash| {
                let stem = info
                    .file_name()
                    .map(|n| n.to_string_lossy().trim_end_matches(".trashinfo").to_string())
                    .unwrap_or_default();
                trash.join("files").join(stem)
            });
        let Some(mut e) = file.as_deref().and_then(|f| fsutil::stat_entry(f, false)) else {
            continue;
        };
        e.name = item.name.to_string_lossy().into_owned();
        e.trash = Some(Box::new(TrashInfo {
            orig_path: item.original_parent.join(&item.name).to_string_lossy().into_owned(),
            deleted: item.time_deleted,
        }));
        e.hidden = false;
        entries.push(e);
        kept.push(item);
    }
    (entries, kept)
}

/// Case-insensitive `*`/`?` glob, compiled once for matching many names.
/// Iterative star backtracking: linear-ish, never exponential.
pub fn glob_matcher(pattern: &str) -> impl Fn(&str) -> bool {
    let p: Vec<char> = pattern.chars().flat_map(char::to_lowercase).collect();
    move |name: &str| {
        let n: Vec<char> = name.chars().flat_map(char::to_lowercase).collect();
        let (mut pi, mut ni, mut star, mut mark) = (0, 0, None, 0);
        while ni < n.len() {
            if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
                pi += 1;
                ni += 1;
            } else if pi < p.len() && p[pi] == '*' {
                star = Some(pi);
                mark = ni;
                pi += 1;
            } else if let Some(s) = star {
                pi = s + 1;
                mark += 1;
                ni = mark;
            } else {
                return false;
            }
        }
        p[pi..].iter().all(|&c| c == '*')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glob_match(p: &str, n: &str) -> bool {
        glob_matcher(p)(n)
    }

    #[test]
    fn globs() {
        assert!(glob_match("*.JPG", "photo.jpg"));
        assert!(glob_match("file?.txt", "file1.txt"));
        assert!(!glob_match("file?.txt", "file10.txt"));
        assert!(glob_match("*", ""));
        assert!(glob_match("a*b*c", "aXXbYYc"));
        assert!(!glob_match("a*b", "a"));
        assert!(!glob_match("*a*a*a*a*a*a*a*b", &"a".repeat(64)));
    }

    fn entries(names: &[&str]) -> Vec<Arc<Entry>> {
        names
            .iter()
            .map(|n| Arc::new(Entry { name: n.to_string(), path: format!("/d/{n}"), ..Default::default() }))
            .collect()
    }

    /// Replays diff ops on `old` the way apply_diff does.
    fn replay(old: &[Arc<Entry>], new: &[Arc<Entry>]) -> (Vec<String>, usize) {
        let spec = SortSpec { key: SortKey::Name, descending: false, folders_first: true };
        let ops = diff(old, new, spec);
        let mut v = old.to_vec();
        for op in &ops {
            match op {
                DiffOp::Remove { at, len } => {
                    v.drain(*at..at + len);
                }
                DiffOp::Insert { at, from } => {
                    v.splice(*at..*at, new[from.clone()].iter().cloned());
                }
                DiffOp::Change { at, from } => v[*at..at + from.len()].clone_from_slice(&new[from.clone()]),
            }
        }
        (v.iter().map(|e| e.name.clone()).collect(), ops.len())
    }

    #[test]
    fn diff_groups_runs() {
        let old = entries(&["a", "b", "c", "d", "e", "f"]);
        let new = entries(&["a", "b2", "b3", "e", "f", "g", "h"]);
        let (names, ops) = replay(&old, &new);
        assert_eq!(names, ["a", "b2", "b3", "e", "f", "g", "h"]);
        // [b] removed, [b2,b3] inserted, [c,d] removed, [g,h] appended.
        assert_eq!(ops, 4);

        let mut changed = entries(&["a", "b", "c"]);
        Arc::make_mut(&mut changed[1]).size = 9;
        Arc::make_mut(&mut changed[2]).size = 9;
        let (names, ops) = replay(&entries(&["a", "b", "c"]), &changed);
        assert_eq!(names, ["a", "b", "c"]);
        assert_eq!(ops, 1);

        assert_eq!(replay(&entries(&["a", "b"]), &entries(&[])).0, Vec::<String>::new());
        assert_eq!(replay(&entries(&["x"]), &entries(&["a", "x", "z"])).0, ["a", "x", "z"]);
    }
}
