//! Direct SMB file transfers through libsmbclient. gvfs's FUSE bridge writes
//! in small synchronous chunks (~14 MB/s measured on gigabit), while
//! libsmbclient with multi-megabyte writes runs at line speed.

use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::fs::{self, File, FileTimes, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

use crate::smb;

/// Bytes per SMB read/write call; large enough that a round trip per call
/// costs little.
const CHUNK: usize = 8 << 20;

/// Context setup and teardown touch libsmbclient's global configuration,
/// which crashes (talloc use-after-free in loadparm) when done concurrently.
static SETUP: Mutex<()> = Mutex::new(());

#[repr(C)]
struct Ctx {
    _p: [u8; 0],
}

#[repr(C)]
struct SFile {
    _p: [u8; 0],
}

type OpenFn = unsafe extern "C" fn(*mut Ctx, *const c_char, c_int, libc::mode_t) -> *mut SFile;
type ReadFn = unsafe extern "C" fn(*mut Ctx, *mut SFile, *mut c_void, usize) -> isize;
type WriteFn = unsafe extern "C" fn(*mut Ctx, *mut SFile, *const c_void, usize) -> isize;
type CloseFn = unsafe extern "C" fn(*mut Ctx, *mut SFile) -> c_int;
type UnlinkFn = unsafe extern "C" fn(*mut Ctx, *const c_char) -> c_int;
type StatFn = unsafe extern "C" fn(*mut Ctx, *const c_char, *mut libc::stat) -> c_int;
type AuthFn = unsafe extern "C" fn(
    *mut Ctx,
    *const c_char,
    *const c_char,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
);
type UtimesFn = unsafe extern "C" fn(*mut Ctx, *const c_char, *mut libc::timeval) -> c_int;
type OpendirFn = unsafe extern "C" fn(*mut Ctx, *const c_char) -> *mut SFile;
type ReaddirFn = unsafe extern "C" fn(*mut Ctx, *mut SFile) -> *mut Dirent;

const SMBC_DIR: c_uint = 7;

/// libsmbclient's `struct smbc_dirent`; `name` is NUL-terminated in place.
#[repr(C)]
struct Dirent {
    smbc_type: c_uint,
    dirlen: c_uint,
    commentlen: c_uint,
    comment: *mut c_char,
    namelen: c_uint,
    name: [c_char; 1],
}

#[link(name = "smbclient")]
extern "C" {
    fn smbc_new_context() -> *mut Ctx;
    fn smbc_init_context(c: *mut Ctx) -> *mut Ctx;
    fn smbc_free_context(c: *mut Ctx, shutdown_ctx: c_int) -> c_int;
    fn smbc_setDebug(c: *mut Ctx, debug: c_int);
    fn smbc_setOptionNoAutoAnonymousLogin(c: *mut Ctx, b: c_int);
    fn smbc_setOptionUserData(c: *mut Ctx, data: *mut c_void);
    fn smbc_getOptionUserData(c: *mut Ctx) -> *mut c_void;
    fn smbc_setFunctionAuthDataWithContext(c: *mut Ctx, f: AuthFn);
    fn smbc_getFunctionOpen(c: *mut Ctx) -> OpenFn;
    fn smbc_getFunctionRead(c: *mut Ctx) -> ReadFn;
    fn smbc_getFunctionWrite(c: *mut Ctx) -> WriteFn;
    fn smbc_getFunctionClose(c: *mut Ctx) -> CloseFn;
    fn smbc_getFunctionUnlink(c: *mut Ctx) -> UnlinkFn;
    fn smbc_getFunctionStat(c: *mut Ctx) -> StatFn;
    fn smbc_getFunctionUtimes(c: *mut Ctx) -> UtimesFn;
    fn smbc_getFunctionOpendir(c: *mut Ctx) -> OpendirFn;
    fn smbc_getFunctionReaddir(c: *mut Ctx) -> ReaddirFn;
    fn smbc_getFunctionClosedir(c: *mut Ctx) -> CloseFn;
    fn smbc_getFunctionRmdir(c: *mut Ctx) -> UnlinkFn;
}

/// A file on a share reached through a gvfs mount, addressed both ways.
pub struct Remote {
    /// host/share, which one client connection serves.
    pub key: String,
    pub host: String,
    pub share: String,
    pub uri: String,
}

/// The SMB address of a path inside a gvfs smb-share mount.
pub fn remote(path: &Path) -> Option<Remote> {
    let (host, share, rest) = smb::gvfs_location(path)?;
    let mut uri = format!("smb://{}/{}", encode(&host), encode(&share));
    for seg in rest.split('/').filter(|s| !s.is_empty()) {
        uri.push('/');
        uri.push_str(&encode(seg));
    }
    Some(Remote { key: format!("{}/{}", host.to_lowercase(), share.to_lowercase()), host, share, uri })
}

/// Percent-encodes everything but unreserved characters, so names holding
/// `;`, `@`, `?` or `%` are not read as URI syntax by libsmbclient.
fn encode(s: &str) -> String {
    crate::fsutil::percent_encode(s, b"-_.~")
}

fn cstr(s: &str) -> io::Result<CString> {
    CString::new(s).map_err(|_| io::ErrorKind::InvalidInput.into())
}

/// One authenticated libsmbclient context. Contexts are not thread-safe, so
/// each copy thread owns its own.
pub struct Client {
    ctx: *mut Ctx,
    /// Read by `auth` through the context's user data; boxed so it stays put.
    _auth: Box<Auth>,
    open: OpenFn,
    read: ReadFn,
    write: WriteFn,
    close: CloseFn,
    unlink: UnlinkFn,
    stat: StatFn,
    utimes: UtimesFn,
    opendir: OpendirFn,
    readdir: ReaddirFn,
    closedir: CloseFn,
    rmdir: UnlinkFn,
}

/// One entry of a share subtree, addressed both ways like `Remote`.
pub struct Node {
    pub uri: String,
    pub path: PathBuf,
    pub dir: bool,
}

// SAFETY: a context is only ever used by the thread that currently owns it.
unsafe impl Send for Client {}

impl Drop for Client {
    fn drop(&mut self) {
        let _g = SETUP.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: ctx came from smbc_init_context and is freed exactly once,
        // before the Auth it points at.
        unsafe { smbc_free_context(self.ctx, 1) };
    }
}

impl Client {
    /// Connects to `r`'s share with the credentials the app holds for its
    /// host; None when there are none or they are refused.
    pub fn connect(r: &Remote) -> Option<Self> {
        let creds = smb::creds_for(&r.host)?;
        let (user, pass) = if creds.anonymous { ("", "") } else { (creds.user.as_str(), creds.password.as_str()) };
        let domain = if creds.domain.is_empty() { "WORKGROUP" } else { creds.domain.as_str() };
        let mut auth = Box::new(Auth { domain: cstr(domain).ok()?, user: cstr(user).ok()?, pass: cstr(pass).ok()? });
        let _g = SETUP.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: plain libsmbclient setup; `auth` outlives the context (see Drop).
        let client = unsafe {
            let raw = smbc_new_context();
            if raw.is_null() {
                return None;
            }
            smbc_setDebug(raw, 0);
            smbc_setOptionNoAutoAnonymousLogin(raw, (!creds.anonymous) as c_int);
            smbc_setOptionUserData(raw, (&mut *auth as *mut Auth).cast());
            smbc_setFunctionAuthDataWithContext(raw, auth_cb);
            let ctx = smbc_init_context(raw);
            if ctx.is_null() {
                smbc_free_context(raw, 0);
                return None;
            }
            Client {
                ctx,
                _auth: auth,
                open: smbc_getFunctionOpen(ctx),
                read: smbc_getFunctionRead(ctx),
                write: smbc_getFunctionWrite(ctx),
                close: smbc_getFunctionClose(ctx),
                unlink: smbc_getFunctionUnlink(ctx),
                stat: smbc_getFunctionStat(ctx),
                utimes: smbc_getFunctionUtimes(ctx),
                opendir: smbc_getFunctionOpendir(ctx),
                readdir: smbc_getFunctionReaddir(ctx),
                closedir: smbc_getFunctionClosedir(ctx),
                rmdir: smbc_getFunctionRmdir(ctx),
            }
        };
        // libsmbclient connects lazily; probe the share so bad credentials
        // fall back to gvfs instead of failing every file.
        let root = format!("smb://{}/{}/", encode(&r.host), encode(&r.share));
        client.stat_ok(&root).then_some(client)
    }

    fn stat_ok(&self, uri: &str) -> bool {
        let Ok(c) = cstr(uri) else { return false };
        // SAFETY: c is NUL-terminated and st a valid out pointer.
        unsafe {
            let mut st: libc::stat = std::mem::zeroed();
            (self.stat)(self.ctx, c.as_ptr(), &mut st) == 0
        }
    }

    fn open_remote(&self, uri: &CString, flags: c_int, mode: u32) -> io::Result<*mut SFile> {
        // SAFETY: uri is NUL-terminated; ctx is live.
        let f = unsafe { (self.open)(self.ctx, uri.as_ptr(), flags, mode as libc::mode_t) };
        if f.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(f)
        }
    }

    fn close_remote(&self, f: *mut SFile) -> io::Result<()> {
        // SAFETY: f was opened on this context and is closed once.
        if unsafe { (self.close)(self.ctx, f) } < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    /// Removes the file (or symlink) at `uri`.
    pub fn unlink(&self, uri: &str) -> io::Result<()> {
        let c = cstr(uri)?;
        // SAFETY: c is NUL-terminated; ctx is live.
        check(unsafe { (self.unlink)(self.ctx, c.as_ptr()) })
    }

    /// Removes the empty directory at `uri`.
    pub fn rmdir(&self, uri: &str) -> io::Result<()> {
        let c = cstr(uri)?;
        // SAFETY: c is NUL-terminated; ctx is live.
        check(unsafe { (self.rmdir)(self.ctx, c.as_ptr()) })
    }

    /// The direct children of the directory at `uri` / `path`.
    pub fn list_dir(&self, uri: &str, path: &Path) -> io::Result<Vec<Node>> {
        let c = cstr(uri)?;
        // SAFETY: c is NUL-terminated; ctx is live.
        let d = unsafe { (self.opendir)(self.ctx, c.as_ptr()) };
        if d.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut out = Vec::new();
        loop {
            // SAFETY: d is an open directory handle on this context; the
            // entry stays valid until the next readdir, and is copied before.
            let e = unsafe { (self.readdir)(self.ctx, d) };
            if e.is_null() {
                break;
            }
            // SAFETY: name is NUL-terminated within the entry.
            let (kind, name) = unsafe { ((*e).smbc_type, CStr::from_ptr((*e).name.as_ptr()).to_string_lossy().into_owned()) };
            if name == "." || name == ".." {
                continue;
            }
            out.push(Node { uri: format!("{uri}/{}", encode(&name)), path: path.join(&name), dir: kind == SMBC_DIR });
        }
        // SAFETY: d is closed exactly once.
        unsafe { (self.closedir)(self.ctx, d) };
        Ok(out)
    }

    /// Every entry below `root` in post-order (children before their
    /// directory), so removing them in sequence empties each directory first.
    /// `root` itself comes last. Unreadable directories are reported through
    /// `err` and treated as empty.
    pub fn list_tree(&self, root: Node, cancel: &AtomicBool, err: &mut dyn FnMut(&Path, io::Error)) -> Vec<Node> {
        let mut out = Vec::new();
        // (node, expanded): a directory is pushed back once its children are queued.
        let mut stack = vec![(root, false)];
        while let Some((node, expanded)) = stack.pop() {
            if cancel.load(Ordering::SeqCst) {
                break;
            }
            if node.dir && !expanded {
                let children = match self.list_dir(&node.uri, &node.path) {
                    Ok(c) => c,
                    Err(e) => {
                        err(&node.path, e);
                        Vec::new()
                    }
                };
                stack.push((node, true));
                stack.extend(children.into_iter().map(|c| (c, false)));
            } else {
                out.push(node);
            }
        }
        out
    }

    /// Copies local `src` to the new remote file `uri` (which must not exist).
    pub fn upload(
        &self,
        src: &Path,
        uri: &str,
        meta: &fs::Metadata,
        cancel: &AtomicBool,
        buf: &mut Vec<u8>,
        progress: &mut dyn FnMut(u64),
    ) -> io::Result<()> {
        let mut input = File::open(src)?;
        let c = cstr(uri)?;
        let f = self.open_remote(&c, libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL, meta.mode() & 0o777)?;
        let res = (|| {
            ensure_buf(buf);
            loop {
                if cancel.load(Ordering::SeqCst) {
                    return Err(io::ErrorKind::Interrupted.into());
                }
                let n = read_full(&mut input, buf)?;
                if n == 0 {
                    return Ok(());
                }
                let mut off = 0;
                while off < n {
                    // SAFETY: buf[off..n] is initialised and outlives the call.
                    let w = unsafe { (self.write)(self.ctx, f, buf[off..n].as_ptr().cast(), n - off) };
                    if w <= 0 {
                        return Err(if w < 0 { io::Error::last_os_error() } else { io::ErrorKind::WriteZero.into() });
                    }
                    off += w as usize;
                }
                progress(n as u64);
            }
        })();
        let closed = self.close_remote(f);
        if let Err(e) = res.and(closed) {
            let _ = self.unlink(uri);
            return Err(e);
        }
        let secs = |t: i64, ns: i64| libc::timeval { tv_sec: t as libc::time_t, tv_usec: (ns / 1000) as libc::suseconds_t };
        let mut times = [secs(meta.atime(), meta.atime_nsec()), secs(meta.mtime(), meta.mtime_nsec())];
        // SAFETY: c is NUL-terminated and times holds the two entries utimes reads.
        unsafe { (self.utimes)(self.ctx, c.as_ptr(), times.as_mut_ptr()) };
        Ok(())
    }

    /// Copies remote `uri` to the new local file `dst` (which must not exist).
    /// `meta` is the source's metadata as seen through the gvfs mount.
    pub fn download(
        &self,
        uri: &str,
        dst: &Path,
        meta: &fs::Metadata,
        cancel: &AtomicBool,
        buf: &mut Vec<u8>,
        progress: &mut dyn FnMut(u64),
    ) -> io::Result<()> {
        let c = cstr(uri)?;
        let f = self.open_remote(&c, libc::O_RDONLY, 0)?;
        let res = (|| {
            // Created with the source mode so a private file is never briefly readable.
            let mut output = OpenOptions::new().write(true).create_new(true).mode(meta.mode() & 0o777).open(dst)?;
            let copied = (|| {
                ensure_buf(buf);
                loop {
                    if cancel.load(Ordering::SeqCst) {
                        return Err(io::ErrorKind::Interrupted.into());
                    }
                    // SAFETY: buf is CHUNK bytes and outlives the call.
                    let n = unsafe { (self.read)(self.ctx, f, buf.as_mut_ptr().cast(), buf.len()) };
                    if n < 0 {
                        return Err(io::Error::last_os_error());
                    }
                    if n == 0 {
                        return Ok(());
                    }
                    output.write_all(&buf[..n as usize])?;
                    progress(n as u64);
                }
            })();
            if let Err(e) = copied {
                drop(output);
                let _ = fs::remove_file(dst);
                return Err(e);
            }
            let _ = output.set_permissions(fs::Permissions::from_mode(meta.mode() & 0o7777));
            let times = FileTimes::new()
                .set_accessed(meta.accessed().unwrap_or(UNIX_EPOCH))
                .set_modified(meta.modified().unwrap_or(UNIX_EPOCH));
            let _ = output.set_times(times);
            Ok(())
        })();
        let _ = self.close_remote(f);
        res
    }
}

struct Auth {
    domain: CString,
    user: CString,
    pass: CString,
}

/// libsmbclient's credential callback: fills its buffers from the Auth the
/// context carries.
unsafe extern "C" fn auth_cb(
    c: *mut Ctx,
    _srv: *const c_char,
    _shr: *const c_char,
    wg: *mut c_char,
    wglen: c_int,
    un: *mut c_char,
    unlen: c_int,
    pw: *mut c_char,
    pwlen: c_int,
) {
    let a = smbc_getOptionUserData(c) as *const Auth;
    if a.is_null() {
        return;
    }
    let a = &*a;
    for (src, dst, len) in [(&a.domain, wg, wglen), (&a.user, un, unlen), (&a.pass, pw, pwlen)] {
        let bytes = src.as_bytes_with_nul();
        if len > 0 && bytes.len() <= len as usize {
            std::ptr::copy_nonoverlapping(bytes.as_ptr().cast(), dst, bytes.len());
        }
    }
}

fn check(rc: c_int) -> io::Result<()> {
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn ensure_buf(buf: &mut Vec<u8>) {
    if buf.len() < CHUNK {
        *buf = vec![0u8; CHUNK];
    }
}

/// Fills `buf` unless the file ends first, so each SMB write is a full chunk.
fn read_full(input: &mut File, buf: &mut [u8]) -> io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        match input.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => n += k,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_fully_encoded() {
        assert_eq!(encode("a b;c@d?e%f#g"), "a%20b%3Bc%40d%3Fe%25f%23g");
        assert_eq!(encode("plain-name_1.txt"), "plain-name_1.txt");
    }
}

#[cfg(test)]
mod live {
    use super::*;

    /// SMB_TEST_DIR: an empty scratch folder inside a gvfs smb-share mount;
    /// SMB_TEST_SRC: a local folder of files. Uploads and re-downloads every
    /// file from 4 threads at once, then compares.
    #[test]
    #[ignore]
    fn parallel_round_trip() {
        let dir = std::path::PathBuf::from(std::env::var("SMB_TEST_DIR").unwrap());
        let src = std::path::PathBuf::from(std::env::var("SMB_TEST_SRC").unwrap());
        let back = src.with_extension("back");
        fs::create_dir_all(&back).unwrap();
        let files: Vec<_> = fs::read_dir(&src).unwrap().flatten().map(|d| d.path()).collect();
        let t = std::time::Instant::now();
        std::thread::scope(|sc| {
            for chunk in files.chunks(files.len().div_ceil(4)) {
                let (dir, back) = (&dir, &back);
                sc.spawn(move || {
                    let c = Client::connect(&remote(&dir.join("x")).unwrap()).expect("connect");
                    let mut buf = Vec::new();
                    let stop = AtomicBool::new(false);
                    for f in chunk {
                        let name = f.file_name().unwrap();
                        let r = remote(&dir.join(name)).unwrap();
                        let meta = fs::metadata(f).unwrap();
                        c.upload(f, &r.uri, &meta, &stop, &mut buf, &mut |_| {}).unwrap();
                        c.download(&r.uri, &back.join(name), &meta, &stop, &mut buf, &mut |_| {}).unwrap();
                    }
                });
            }
        });
        eprintln!("{} files up+down in {:.1}s", files.len(), t.elapsed().as_secs_f64());
        for f in &files {
            assert_eq!(fs::read(f).unwrap(), fs::read(back.join(f.file_name().unwrap())).unwrap());
        }
    }
}
