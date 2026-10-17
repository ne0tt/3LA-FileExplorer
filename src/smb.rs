use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::fsutil::Entry;

pub const NETWORK_URI: &str = "network:///";
const MOUNT_TIMEOUT: Duration = Duration::from_secs(40);
const SERVER_GLYPH: &str = "\u{F048D}";
const SHARE_GLYPH: &str = "\u{F0870}";

#[derive(Clone, Default)]
pub struct Creds {
    pub user: String,
    pub domain: String,
    pub password: String,
    pub anonymous: bool,
    /// Kept in the login keyring across restarts; otherwise memory-only.
    pub remember: bool,
}

impl Drop for Creds {
    // Best-effort scrub so passwords do not linger in freed memory.
    fn drop(&mut self) {
        // SAFETY: zero bytes are valid UTF-8, and the String is dropped next.
        for b in unsafe { self.password.as_bytes_mut() } {
            unsafe { std::ptr::write_volatile(b, 0) };
        }
    }
}

/// In-memory credential cache for this run, keyed by lower-case host.
fn creds() -> &'static Mutex<HashMap<String, Creds>> {
    static C: OnceLock<Mutex<HashMap<String, Creds>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

/// Shares mounted with memory-only credentials; unmounted on exit so access
/// is forgotten along with the password.
fn session_mounts() -> &'static Mutex<Vec<(String, String)>> {
    static M: OnceLock<Mutex<Vec<(String, String)>>> = OnceLock::new();
    M.get_or_init(Default::default)
}

pub fn set_creds(host: &str, c: Creds) {
    let host = host.to_lowercase();
    let stored = c.clone();
    let h = host.clone();
    // The keyring may need unlocking (a desktop prompt), so stay off the GUI thread.
    std::thread::spawn(move || {
        if stored.remember {
            keyring::store(&h, &stored);
        } else {
            keyring::clear(&h);
        }
    });
    misses().lock().unwrap().remove(&host);
    creds().lock().unwrap().insert(host, c);
}

/// Hosts with no keyring entry, so each listing doesn't re-run secret-tool.
fn misses() -> &'static Mutex<std::collections::HashSet<String>> {
    static M: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    M.get_or_init(Default::default)
}

fn creds_for(host: &str) -> Option<Creds> {
    let key = host.to_lowercase();
    if let Some(c) = creds().lock().unwrap().get(&key).cloned() {
        return Some(c);
    }
    if misses().lock().unwrap().contains(&key) {
        return None;
    }
    let Some(c) = keyring::lookup(&key) else {
        misses().lock().unwrap().insert(key);
        return None;
    };
    creds().lock().unwrap().insert(key, c.clone());
    Some(c)
}

/// Unmounts shares opened with memory-only credentials, in parallel so exit
/// waits at most one timeout. Called on exit.
pub fn forget_session() {
    let mounts = std::mem::take(&mut *session_mounts().lock().unwrap());
    let jobs: Vec<_> = mounts
        .into_iter()
        .map(|(host, share)| {
            std::thread::spawn(move || {
                let uri = build_uri(&host, Some(&share), "");
                run_timeout(Command::new("gio").args(["mount", "-u", &uri]), Duration::from_secs(5))
            })
        })
        .collect();
    for j in jobs {
        let _ = j.join();
    }
    creds().lock().unwrap().clear();
    misses().lock().unwrap().clear();
}

/// Secret Service (gnome-keyring/KWallet) via `secret-tool`. The password is
/// only ever passed on stdin/stdout, never as an argument.
mod keyring {
    use std::io::Write;
    use std::process::{Command, Stdio};

    use super::Creds;

    const SERVICE: &str = "3la-fileexplorer-smb";

    pub fn store(host: &str, c: &Creds) {
        let anon = if c.anonymous { "1" } else { "0" };
        let child = Command::new("secret-tool")
            .args(["store", &format!("--label=3LA File Explorer: smb://{host}")])
            .args(["service", SERVICE, "host", host, "user", &c.user, "domain", &c.domain, "anonymous", anon])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(mut child) = child {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(c.password.as_bytes());
            }
            let _ = child.wait();
        }
    }

    pub fn clear(host: &str) {
        let _ = Command::new("secret-tool")
            .args(["clear", "service", SERVICE, "host", host])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    pub fn lookup(host: &str) -> Option<Creds> {
        // `search` prints attributes on stderr; `lookup` prints just the secret.
        let attrs = Command::new("secret-tool")
            .args(["search", "service", SERVICE, "host", host])
            .stdin(Stdio::null())
            .output()
            .ok()?;
        let meta = String::from_utf8_lossy(&attrs.stderr);
        let attr = |k: &str| {
            meta.lines()
                .find_map(|l| l.strip_prefix(&format!("attribute.{k} = ")))
                .map(str::to_string)
        };
        let user = attr("user")?;
        let secret = Command::new("secret-tool")
            .args(["lookup", "service", SERVICE, "host", host])
            .stdin(Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        Some(Creds {
            user,
            domain: attr("domain").unwrap_or_default(),
            password: String::from_utf8_lossy(&secret.stdout).into_owned(),
            anonymous: attr("anonymous").as_deref() == Some("1"),
            remember: true,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SmbUri {
    pub host: String,
    pub share: Option<String>,
    pub rest: String,
}

/// smb://[domain;][user@]host[/share[/rest]]
pub fn parse(uri: &str) -> Option<SmbUri> {
    let body = uri.strip_prefix("smb://")?;
    let body = crate::fsutil::percent_decode(body);
    let (auth_host, path) = body.split_once('/').unwrap_or((body.as_str(), ""));
    let host = auth_host.rsplit_once('@').map_or(auth_host, |(_, h)| h).to_string();
    if host.is_empty() {
        return None;
    }
    let mut parts = path.splitn(2, '/');
    let share = parts.next().filter(|s| !s.is_empty()).map(str::to_string);
    let rest = parts.next().unwrap_or("").trim_matches('/').to_string();
    Some(SmbUri { host, share, rest })
}

const SEGMENT_SAFE: &[u8] = b"-_.~!$&'()*+,;=:@";

/// Encoded smb:// URI from decoded parts; the inverse of `parse`.
pub fn build_uri(host: &str, share: Option<&str>, rest: &str) -> String {
    let mut out = format!("smb://{}/", crate::fsutil::percent_encode(host, SEGMENT_SAFE));
    if let Some(share) = share {
        out.push_str(&crate::fsutil::percent_encode(share, SEGMENT_SAFE));
        for seg in rest.split('/').filter(|s| !s.is_empty()) {
            out.push('/');
            out.push_str(&crate::fsutil::percent_encode(seg, SEGMENT_SAFE));
        }
    }
    out
}

/// Decoded (display) form of an smb:// URI, as used throughout the UI.
pub fn display_uri(uri: &str) -> String {
    match parse(uri) {
        Some(SmbUri { host, share: Some(share), rest }) if rest.is_empty() => format!("smb://{host}/{share}"),
        Some(SmbUri { host, share: Some(share), rest }) => format!("smb://{host}/{share}/{rest}"),
        Some(SmbUri { host, share: None, .. }) => format!("smb://{host}/"),
        None => uri.to_string(),
    }
}

/// Encoded form of a display smb:// URI, for gio and the bookmarks file.
pub fn encode_uri(uri: &str) -> String {
    match parse(uri) {
        Some(u) => build_uri(&u.host, u.share.as_deref(), &u.rest),
        None => uri.to_string(),
    }
}

fn gvfs_dir() -> PathBuf {
    // SAFETY: getuid has no failure modes.
    PathBuf::from(format!("/run/user/{}/gvfs", unsafe { libc::getuid() }))
}

fn gvfs_fields(name: &str) -> Option<HashMap<String, String>> {
    let rest = name.strip_prefix("smb-share:")?;
    Some(
        rest.split(',')
            .filter_map(|kv| kv.split_once('='))
            .map(|(k, v)| (k.to_lowercase(), crate::fsutil::percent_decode(v)))
            .collect(),
    )
}

/// Local FUSE path of an already-mounted share, if any.
pub fn mounted_root(host: &str, share: &str) -> Option<PathBuf> {
    let rd = std::fs::read_dir(gvfs_dir()).ok()?;
    rd.flatten().map(|d| d.path()).find(|p| {
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        gvfs_fields(&name).is_some_and(|f| {
            f.get("server").is_some_and(|s| s.eq_ignore_ascii_case(host))
                && f.get("share").is_some_and(|s| s.eq_ignore_ascii_case(share))
        })
    })
}

/// Maps a local gvfs path back to (smb://host/share, local root).
pub fn uri_for_local(path: &str) -> Option<(String, String)> {
    let base = gvfs_dir();
    let rel = PathBuf::from(path);
    let rel = rel.strip_prefix(&base).ok()?;
    let first = rel.components().next()?.as_os_str().to_string_lossy().into_owned();
    let f = gvfs_fields(&first)?;
    let uri = format!("smb://{}/{}", f.get("server")?, f.get("share")?);
    Some((uri, base.join(first).to_string_lossy().into_owned()))
}

/// smb:// URIs from the bookmarks file.
pub fn bookmark_uris() -> Vec<String> {
    std::fs::read_to_string(crate::config::bookmarks_file())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|u| u.starts_with("smb://"))
        .map(display_uri)
        .collect()
}

pub enum MountResult {
    Ready(String),
    AuthRequired(String),
    Failed(String),
}

/// Mounts smb://host/share through gvfs and returns the local path of the
/// requested location. gio's interactive prompts are answered from the
/// session cache; with no cached credentials they get EOF, which gio reports
/// as a cancelled password dialog — our cue to ask the user.
pub fn open(uri: &str) -> MountResult {
    let Some(u) = parse(uri) else {
        return MountResult::Failed(format!("Not a valid SMB address: {uri}"));
    };
    let Some(share) = u.share.clone() else {
        return MountResult::Failed("No share in address".into());
    };
    let local = |root: PathBuf| {
        let p = if u.rest.is_empty() { root } else { root.join(&u.rest) };
        MountResult::Ready(p.to_string_lossy().into_owned())
    };
    if let Some(root) = mounted_root(&u.host, &share) {
        return local(root);
    }
    let creds = creds_for(&u.host);
    let mount_uri = build_uri(&u.host, Some(&share), "");
    match gio_mount(&mount_uri, creds.as_ref()) {
        Ok(()) if creds.as_ref().is_some_and(|c| !c.remember) => {
            let mut sm = session_mounts().lock().unwrap();
            let entry = (u.host.clone(), share.clone());
            if !sm.contains(&entry) {
                sm.push(entry);
            }
            drop(sm);
            match mounted_root(&u.host, &share) {
                Some(root) => local(root),
                None => MountResult::Failed("Mounted, but the share did not appear under gvfs".into()),
            }
        }
        Ok(()) => match mounted_root(&u.host, &share) {
            Some(root) => local(root),
            None => MountResult::Failed("Mounted, but the share did not appear under gvfs".into()),
        },
        // gio asked for credentials: either we had none (it got EOF and exits
        // 2 with no message) or the cached ones were rejected.
        // No credentials tried yet: just a login request, so no message.
        Err(e) if e.prompted || is_auth_error(&e.message) => MountResult::AuthRequired(if creds.is_some() {
            format!("Login failed. {}", e.message).trim().to_string()
        } else {
            String::new()
        }),
        Err(e) => MountResult::Failed(e.message),
    }
}

fn is_auth_error(e: &str) -> bool {
    let l = e.to_lowercase();
    ["password", "cancelled", "canceled", "permission denied", "logon", "authentic", "access denied", "nt_status_access"]
        .iter()
        .any(|k| l.contains(k))
}

struct MountError {
    message: String,
    /// gio printed a User/Domain/Password prompt.
    prompted: bool,
}

fn gio_mount(uri: &str, creds: Option<&Creds>) -> Result<(), MountError> {
    gio_mount_with("gio", uri, creds, MOUNT_TIMEOUT)
}

fn gio_mount_with(program: &str, uri: &str, creds: Option<&Creds>, timeout: Duration) -> Result<(), MountError> {
    let fail = |message: String| MountError { message, prompted: false };
    let mut cmd = Command::new(program);
    cmd.arg("mount");
    if creds.is_some_and(|c| c.anonymous) {
        cmd.arg("--anonymous");
    }
    // Own process group, so the watchdog can kill gio and any helper it
    // spawned (a surviving child would hold the stdout pipe open).
    let mut child = cmd
        .arg(uri)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| fail(e.to_string()))?;
    let mut stdin = child.stdin.take();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let err_reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });

    // gio blocks on reads and gvfs can stall on a slow server, so a watchdog
    // kills the process after MOUNT_TIMEOUT; the read loop then sees EOF.
    let pid = child.id() as libc::pid_t;
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let timed_out = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let (done, timed_out) = (done.clone(), timed_out.clone());
        std::thread::spawn(move || {
            let start = Instant::now();
            while start.elapsed() < timeout {
                if done.load(std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            if !done.load(std::sync::atomic::Ordering::SeqCst) {
                timed_out.store(true, std::sync::atomic::Ordering::SeqCst);
                // SAFETY: pid leads the process group we created for our own
                // child, which has not been reaped (wait() runs after the read loop).
                unsafe { libc::kill(-pid, libc::SIGKILL) };
            }
        });
    }

    // Answer prompts ("User [x]: ", "Domain [y]: ", "Password: ") as they
    // appear; gio only asks for what the backend needs. Each kind is answered
    // once: gvfs re-asks after a rejected login, and answering again with the
    // same credentials would loop forever.
    let mut seen = String::new();
    let mut handled = 0usize;
    let mut asked: Vec<&'static str> = Vec::new();
    let mut rejected = false;
    let mut buf = [0u8; 256];
    if creds.is_none() {
        // Nothing to answer with: EOF on the first prompt makes gio give up.
        stdin = None;
    }
    loop {
        let n = stdout.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        seen.push_str(&String::from_utf8_lossy(&buf[..n]));
        let ends: Vec<usize> = seen.match_indices(": ").map(|(i, _)| i).collect();
        while handled < ends.len() {
            let line = seen[..ends[handled]].rsplit('\n').next().unwrap_or("").to_lowercase();
            handled += 1;
            let kind = if line.contains("password") {
                "password"
            } else if line.contains("domain") {
                "domain"
            } else if line.contains("user") {
                "user"
            } else {
                continue;
            };
            if asked.contains(&kind) {
                rejected = true;
            }
            asked.push(kind);
            let reply = creds.filter(|_| !rejected).map(|c| match kind {
                "password" => c.password.clone(),
                "domain" => c.domain.clone(),
                _ => c.user.clone(),
            });
            match (reply, stdin.as_mut()) {
                (Some(r), Some(w)) => {
                    let _ = writeln!(w, "{r}");
                }
                _ => stdin = None,
            }
        }
    }
    drop(stdin);
    let status = child.wait().map_err(|e| fail(e.to_string()))?;
    done.store(true, std::sync::atomic::Ordering::SeqCst);
    let err = err_reader.join().unwrap_or_default();
    if status.success() {
        return Ok(());
    }
    if timed_out.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(MountError {
            message: format!("Timed out after {}s waiting for the server", timeout.as_secs()),
            prompted: false,
        });
    }
    // Prompts land on stdout; anything after the last one is gio's verdict.
    let tail = seen.rsplit(": ").next().unwrap_or("").trim();
    let message = [err.trim(), tail]
        .into_iter()
        .find(|m| !m.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("gio mount failed ({status})"));
    let message = if rejected { format!("The server rejected the credentials. {message}") } else { message };
    Err(MountError { message, prompted: !asked.is_empty() })
}

fn virtual_entry(name: &str, path: String, glyph: &'static str, type_text: &str) -> Entry {
    Entry {
        name: name.to_string(),
        path,
        is_dir: true,
        mime: crate::fsutil::intern("inode/directory"),
        type_text: crate::fsutil::intern(type_text),
        glyph,
        category: "folder",
        ..Default::default()
    }
}

/// Disk shares on a host via `smbclient -L`. Credentials come from the
/// session cache; the password travels in the PASSWD environment variable,
/// never on the command line.
pub fn list_shares(host: &str) -> Result<Vec<Entry>, String> {
    let creds = creds_for(host);
    let mut cmd = Command::new("smbclient");
    cmd.args(["-L", &format!("//{host}"), "-g", "--timeout=8"]);
    match &creds {
        Some(c) if !c.anonymous => {
            let user = if c.domain.is_empty() { c.user.clone() } else { format!("{}/{}", c.domain, c.user) };
            cmd.args(["-U", &user]).env("PASSWD", &c.password);
        }
        _ => {
            cmd.arg("-N");
        }
    }
    // --timeout doesn't cover name resolution stalls; bound the whole run.
    let (status, stdout, stderr) = run_output(&mut cmd, Duration::from_secs(15)).ok_or("Could not run smbclient")?;
    let Some(status) = status else {
        return Err(format!("Timed out listing shares on {host}"));
    };
    let text = String::from_utf8_lossy(&stdout);
    let shares: Vec<Entry> = text
        .lines()
        .filter_map(|l| {
            let mut f = l.splitn(3, '|');
            match (f.next(), f.next()) {
                (Some("Disk"), Some(name)) if !name.ends_with('$') => Some(virtual_entry(
                    name,
                    format!("smb://{host}/{name}"),
                    SHARE_GLYPH,
                    "Network share",
                )),
                _ => None,
            }
        })
        .collect();
    if !shares.is_empty() || status.success() {
        return Ok(shares);
    }
    let err = format!("{}{}", String::from_utf8_lossy(&stderr), text);
    if is_auth_error(&err) || err.contains("NT_STATUS_LOGON_FAILURE") {
        Err(format!("Authentication required for {host}"))
    } else {
        Err(err.lines().find(|l| l.contains("NT_STATUS") || l.contains("failed")).unwrap_or(err.trim()).trim().to_string())
    }
}

const DISCOVERY_TTL: Duration = Duration::from_secs(60);

/// NetBIOS sweep results, reused for DISCOVERY_TTL: a sweep takes seconds.
fn cached_netbios_hosts() -> Vec<String> {
    static C: Mutex<Option<(Instant, Vec<String>)>> = Mutex::new(None);
    if let Some((at, hosts)) = C.lock().unwrap().as_ref() {
        if at.elapsed() < DISCOVERY_TTL {
            return hosts.clone();
        }
    }
    let hosts = netbios_hosts();
    *C.lock().unwrap() = Some((Instant::now(), hosts.clone()));
    hosts
}

/// Servers to show under network:///: NetBIOS broadcast discovery plus any
/// server mentioned by SMB bookmarks or current gvfs mounts.
pub fn discover_hosts(bookmark_uris: &[String]) -> Vec<Entry> {
    let mut hosts: Vec<String> = Vec::new();
    let mut add = |h: &str| {
        let h = h.trim();
        if !h.is_empty() && !hosts.iter().any(|x| x.eq_ignore_ascii_case(h)) {
            hosts.push(h.to_string());
        }
    };
    for u in bookmark_uris {
        if let Some(p) = parse(u) {
            add(&p.host);
        }
    }
    if let Ok(rd) = std::fs::read_dir(gvfs_dir()) {
        for d in rd.flatten() {
            if let Some(f) = gvfs_fields(&d.file_name().to_string_lossy()) {
                if let Some(s) = f.get("server") {
                    add(s);
                }
            }
        }
    }
    for h in cached_netbios_hosts() {
        add(&h);
    }
    hosts.sort_by_key(|h| h.to_lowercase());
    hosts
        .iter()
        .map(|h| virtual_entry(h, format!("smb://{h}/"), SERVER_GLYPH, "Server"))
        .collect()
}

/// Broadcast `nmblookup '*'`, then resolve each responder's file-server
/// (<20>) NetBIOS name. Bounded by short timeouts so a quiet LAN is fast.
fn netbios_hosts() -> Vec<String> {
    let Some(out) = run_timeout(Command::new("nmblookup").arg("*"), Duration::from_secs(4)) else {
        return Vec::new();
    };
    let ips: Vec<String> = out
        .lines()
        .filter(|l| l.contains("*<00>"))
        .filter_map(|l| l.split_whitespace().next().map(str::to_string))
        .collect();
    let handles: Vec<_> = ips
        .into_iter()
        .take(64)
        .map(|ip| {
            std::thread::spawn(move || {
                let out = run_timeout(Command::new("nmblookup").args(["-A", &ip]), Duration::from_secs(3))?;
                out.lines()
                    .find(|l| l.contains("<20>") && !l.contains("<GROUP>"))
                    .and_then(|l| l.split_whitespace().next())
                    .map(str::to_string)
            })
        })
        .collect();
    handles.into_iter().filter_map(|h| h.join().ok().flatten()).collect()
}

/// Stdout of `cmd`, including whatever it printed before a timeout kill.
fn run_timeout(cmd: &mut Command, limit: Duration) -> Option<String> {
    let (_, stdout, _) = run_output(cmd, limit)?;
    Some(String::from_utf8_lossy(&stdout).into_owned())
}

/// Runs `cmd` capturing (status, stdout, stderr); kills its process group
/// after `limit`, which leaves status None. Output is read concurrently so a
/// chatty child can't block on a full pipe.
fn run_output(cmd: &mut Command, limit: Duration) -> Option<(Option<std::process::ExitStatus>, Vec<u8>, Vec<u8>)> {
    let mut child = cmd
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let drain = |p: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut v = Vec::new();
            if let Some(mut p) = p {
                let _ = p.read_to_end(&mut v);
            }
            v
        })
    };
    let out = drain(child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let err = drain(child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) if start.elapsed() > limit => {
                // SAFETY: the child leads its own, not yet reaped, process group.
                unsafe { libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL) };
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => break None,
        }
    };
    Some((status, out.join().unwrap_or_default(), err.join().unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uris() {
        assert_eq!(
            parse("smb://fileserver/storage"),
            Some(SmbUri { host: "fileserver".into(), share: Some("storage".into()), rest: "".into() })
        );
        assert_eq!(
            parse("smb://me@nas/media/Films/2024/"),
            Some(SmbUri { host: "nas".into(), share: Some("media".into()), rest: "Films/2024".into() })
        );
        assert_eq!(parse("smb://dbserver/"), Some(SmbUri { host: "dbserver".into(), share: None, rest: "".into() }));
        assert_eq!(parse("smb://"), None);
        assert_eq!(parse("/home"), None);
        assert_eq!(parse("smb://h/my%20share").unwrap().share.as_deref(), Some("my share"));
    }

    #[test]
    fn gvfs_names() {
        let f = gvfs_fields("smb-share:domain=WORKGROUP,server=fileserver,share=storage,user=sispx").unwrap();
        assert_eq!(f["server"], "fileserver");
        assert_eq!(f["share"], "storage");
        assert!(gvfs_fields("sftp:host=x").is_none());
    }

    /// Writing a script while another test thread forks can leak the write
    /// handle into that child, making exec fail with ETXTBSY; serialise.
    static FAKE_GIO: Mutex<()> = Mutex::new(());

    /// A stand-in for `gio mount` driven by a shell script, so the prompt
    /// handling is tested without any network.
    fn fake_gio(name: &str, body: &str) -> FakeGio {
        let dir = std::env::temp_dir().join(format!("tla-fe-fakegio-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        FakeGio(p.to_string_lossy().into_owned())
    }

    /// Removes the script, and its directory once empty, when the test ends.
    struct FakeGio(String);

    impl Drop for FakeGio {
        fn drop(&mut self) {
            let p = std::path::Path::new(&self.0);
            let _ = std::fs::remove_file(p);
            if let Some(dir) = p.parent() {
                let _ = std::fs::remove_dir(dir);
            }
        }
    }

    const PROMPTS: &str = r#"
printf 'Authentication Required\nEnter user and password for share “s” on “h”:\n'
printf 'User [me]: '; read u || exit 2
printf 'Domain [WORKGROUP]: '; read d || exit 2
printf 'Password: '; read p || exit 2
"#;

    fn creds(pw: &str) -> Creds {
        Creds { user: "bob".into(), domain: "LAN".into(), password: pw.into(), anonymous: false, remember: false }
    }

    #[test]
    fn mount_answers_prompts() {
        let _guard = FAKE_GIO.lock().unwrap_or_else(|e| e.into_inner());
        let ok = fake_gio("ok", &format!(r#"{PROMPTS}
[ "$u" = bob ] && [ "$d" = LAN ] && [ "$p" = secret ] && exit 0
exit 1"#));
        assert!(gio_mount_with(&ok.0, "smb://h/s", Some(&creds("secret")), Duration::from_secs(5)).is_ok());
    }

    #[test]
    fn mount_without_creds_needs_auth() {
        let _guard = FAKE_GIO.lock().unwrap_or_else(|e| e.into_inner());
        let g = fake_gio("nocreds", PROMPTS);
        let e = gio_mount_with(&g.0, "smb://h/s", None, Duration::from_secs(5)).unwrap_err();
        assert!(e.prompted);
    }

    #[test]
    fn mount_rejected_creds_do_not_loop() {
        let _guard = FAKE_GIO.lock().unwrap_or_else(|e| e.into_inner());
        // Re-prompts forever after a bad password, like gvfs does.
        let g = fake_gio("reject", &format!("while true; do {PROMPTS}\ndone"));
        let t = Instant::now();
        let e = gio_mount_with(&g.0, "smb://h/s", Some(&creds("wrong")), Duration::from_secs(5)).unwrap_err();
        assert!(e.prompted && e.message.contains("rejected"), "{}", e.message);
        assert!(t.elapsed() < Duration::from_secs(4));
    }

    #[test]
    fn mount_times_out() {
        let _guard = FAKE_GIO.lock().unwrap_or_else(|e| e.into_inner());
        let g = fake_gio("hang", "sleep 30");
        let t = Instant::now();
        let e = gio_mount_with(&g.0, "smb://h/s", None, Duration::from_secs(1)).unwrap_err();
        assert!(e.message.contains("Timed out"), "{}", e.message);
        assert!(t.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn auth_errors() {
        assert!(is_auth_error("gio: smb://h/s: Password dialog cancelled"));
        assert!(is_auth_error("NT_STATUS_ACCESS_DENIED"));
        assert!(!is_auth_error("No route to host"));
    }
}

#[cfg(test)]
mod keyring_tests {
    use super::*;

    /// Touches the real login keyring under a throwaway host; run explicitly.
    #[test]
    #[ignore]
    fn keyring_round_trip() {
        let host = "tla-fe-selftest-host";
        let c = Creds { user: "bob".into(), domain: "LAN".into(), password: "dummy pw ü".into(), anonymous: false, remember: true };
        keyring::store(host, &c);
        let got = keyring::lookup(host).expect("stored entry");
        assert_eq!((got.user.as_str(), got.domain.as_str(), got.password.as_str(), got.remember), ("bob", "LAN", "dummy pw ü", true));
        keyring::clear(host);
        assert!(keyring::lookup(host).is_none());
    }
}
