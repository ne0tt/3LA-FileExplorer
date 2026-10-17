use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

#[derive(Clone, Debug, serde::Serialize)]
pub struct App {
    pub id: String,
    pub name: String,
    pub icon: String,
    #[serde(skip)]
    pub file: PathBuf,
    #[serde(skip)]
    pub no_display: bool,
}

fn data_dirs() -> Vec<PathBuf> {
    let mut v = vec![dirs::data_dir().unwrap_or_else(|| crate::fsutil::home().join(".local/share"))];
    let sys = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    v.extend(sys.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    v
}

/// `None` marks an id hidden by a higher-priority entry (Hidden=true), which
/// masks the same id in later data dirs.
fn scan(dir: &Path, prefix: &str, out: &mut HashMap<String, Option<App>>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for d in rd.flatten() {
        let name = d.file_name().to_string_lossy().into_owned();
        // file_type does not follow symlinks, so a symlink loop can't recurse forever.
        if d.file_type().is_ok_and(|t| t.is_dir()) {
            scan(&d.path(), &format!("{prefix}{name}-"), out);
            continue;
        }
        let Some(stem) = name.strip_suffix(".desktop") else { continue };
        let id = format!("{prefix}{stem}.desktop");
        if let std::collections::hash_map::Entry::Vacant(slot) = out.entry(id) {
            let app = parse(&d.path(), slot.key());
            slot.insert(app);
        }
    }
}

fn parse(path: &Path, id: &str) -> Option<App> {
    let text = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let (mut name, mut icon, mut kind, mut no_display) = (None, String::new(), String::new(), false);
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim();
        match k.trim() {
            "Name" => name = Some(v.to_string()),
            "Icon" => icon = v.to_string(),
            "Type" => kind = v.to_string(),
            "NoDisplay" => no_display = v == "true",
            "Hidden" if v == "true" => return None,
            _ => {}
        }
    }
    if kind != "Application" {
        return None;
    }
    Some(App {
        id: id.to_string(),
        name: name?,
        icon,
        file: path.to_path_buf(),
        no_display,
    })
}

/// mtimes that change when files are added to / removed from a set of paths.
fn stamp(paths: impl IntoIterator<Item = PathBuf>) -> Vec<Option<SystemTime>> {
    paths
        .into_iter()
        .map(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
        .collect()
}

struct Registry {
    stamp: Vec<Option<SystemTime>>,
    apps: Arc<HashMap<String, Option<App>>>,
}

fn app_dirs() -> Vec<PathBuf> {
    data_dirs().into_iter().map(|d| d.join("applications")).collect()
}

/// The parsed .desktop registry, rebuilt when an applications dir changes.
fn registry() -> Arc<HashMap<String, Option<App>>> {
    static R: Mutex<Option<Registry>> = Mutex::new(None);
    let now = stamp(app_dirs());
    let mut r = R.lock().unwrap();
    if let Some(reg) = r.as_ref().filter(|reg| reg.stamp == now) {
        return reg.apps.clone();
    }
    let mut m = HashMap::new();
    for d in app_dirs() {
        scan(&d, "", &mut m);
    }
    let apps = Arc::new(m);
    *r = Some(Registry { stamp: now, apps: apps.clone() });
    apps
}

/// Builds the registry ahead of first use (call off the GUI thread).
pub fn warm() {
    registry();
}

pub fn lookup(id: &str) -> Option<App> {
    registry().get(id).cloned().flatten()
}

pub fn all_apps() -> Vec<App> {
    let mut v: Vec<App> = registry()
        .values()
        .flatten()
        .filter(|a| !a.no_display)
        .cloned()
        .collect();
    v.sort_by_cached_key(|a| a.name.to_lowercase());
    v
}

type MimeApps = (Option<String>, Vec<String>);

fn mime_files() -> Vec<PathBuf> {
    let mut v = vec![dirs::config_dir()
        .unwrap_or_else(|| crate::fsutil::home().join(".config"))
        .join("mimeapps.list")];
    v.extend(app_dirs().into_iter().flat_map(|d| [d.join("mimeinfo.cache"), d.join("mimeapps.list")]));
    v
}

/// (stamp of the mime files, cached answers).
type MimeCache = (Vec<Option<SystemTime>>, HashMap<String, MimeApps>);

fn mime_cache() -> &'static Mutex<MimeCache> {
    static C: OnceLock<Mutex<MimeCache>> = OnceLock::new();
    C.get_or_init(Default::default)
}

/// Uses `gio mime`, which honours mimeapps.list and mime subclassing exactly as
/// Thunar does. Returns (default id, registered+recommended ids). Cached until
/// any mimeapps.list / mimeinfo.cache changes.
pub fn for_mime(mime: &str) -> MimeApps {
    let now = stamp(mime_files());
    {
        let mut c = mime_cache().lock().unwrap();
        if c.0 != now {
            *c = (now.clone(), HashMap::new());
        } else if let Some(hit) = c.1.get(mime) {
            return hit.clone();
        }
    }
    let res = query_mime(mime);
    mime_cache().lock().unwrap().1.insert(mime.to_string(), res.clone());
    res
}

fn query_mime(mime: &str) -> MimeApps {
    // gio's headings are translated; the C locale keeps them parseable.
    let Ok(out) = Command::new("gio").env("LC_ALL", "C").args(["mime", mime]).output() else {
        return (None, Vec::new());
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut default = None;
    let mut ids = Vec::new();
    for line in text.lines() {
        if let Some((_, d)) = line.split_once("”: ").or_else(|| line.split_once("\": ")) {
            if line.starts_with("Default application") {
                default = Some(d.trim().to_string());
            }
            continue;
        }
        if line.starts_with('\t') || line.starts_with("  ") {
            let id = line.trim().to_string();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    (default, ids)
}

pub fn set_default(mime: &str, id: &str) -> Result<(), String> {
    let st = Command::new("gio")
        .args(["mime", mime, id])
        .stdout(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;
    mime_cache().lock().unwrap().1.clear();
    if st.success() {
        Ok(())
    } else {
        Err(format!("gio mime exited with {st}"))
    }
}

/// `gio launch`; stderr is piped so a failure can be reported (see `reap`).
pub fn launch(id: &str, files: &[String]) -> Result<Child, String> {
    let app = lookup(id).ok_or_else(|| format!("Application {id} not found"))?;
    Command::new("gio")
        .arg("launch")
        .arg(&app.file)
        .args(files)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())
}

pub fn open_default(files: &[String]) -> Result<Child, String> {
    Command::new("gio")
        .arg("open")
        .args(files)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())
}

/// Expands Exec-style field codes into a `sh -c` script whose positional
/// parameters are the files. Returns (script, run once per file).
fn expand_command(command: &str) -> (String, bool) {
    let mut out = String::new();
    let (mut per_file, mut any) = (false, false);
    let mut chars = command.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('F' | 'U') => {
                out.push_str("\"$@\"");
                any = true;
            }
            Some('f' | 'u') => {
                out.push_str("\"$1\"");
                (per_file, any) = (true, true);
            }
            Some('%') => out.push('%'),
            Some('i' | 'c' | 'k' | 'd' | 'D' | 'n' | 'N' | 'v' | 'm') | None => {}
            Some(other) => {
                out.push('%');
                out.push(other);
            }
        }
    }
    if !any {
        out.push_str(" \"$@\"");
    }
    (out, per_file)
}

/// Runs a user-typed command; %f/%u start one instance per file, as Thunar does.
pub fn run_custom(command: &str, files: &[String]) -> Result<Vec<Child>, String> {
    let (script, per_file) = expand_command(command);
    let spawn = |args: &[String]| {
        Command::new("sh")
            .arg("-c")
            .arg(&script)
            .arg("sh")
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())
    };
    if per_file && files.len() > 1 {
        files.iter().map(|f| spawn(std::slice::from_ref(f))).collect()
    } else {
        Ok(vec![spawn(files)?])
    }
}

/// Waits for `child` on a background thread so it never lingers as a zombie.
/// If it fails, `on_error` gets its stderr (when piped). Remaining output is
/// drained, since a launched app may inherit the pipe and write to it later.
pub fn reap(mut child: Child, on_error: impl FnOnce(String) + Send + 'static) {
    std::thread::spawn(move || {
        let mut stderr = child.stderr.take();
        let Ok(status) = child.wait() else { return };
        if !status.success() {
            let mut msg = String::new();
            if let Some(e) = stderr.as_mut() {
                let _ = e.take(4096).read_to_string(&mut msg);
            }
            let msg = msg.trim();
            on_error(if msg.is_empty() { format!("Failed to open ({status})") } else { msg.to_string() });
        }
        if let Some(mut e) = stderr {
            let _ = std::io::copy(&mut e, &mut std::io::sink());
        }
    });
}

pub fn terminal() -> String {
    if let Ok(t) = std::env::var("TERMINAL") {
        if !t.is_empty() {
            return t;
        }
    }
    for t in ["kitty", "foot", "alacritty", "wezterm", "konsole", "xfce4-terminal", "gnome-terminal", "xterm"] {
        if which(t) {
            return t.to_string();
        }
    }
    "xterm".into()
}

pub fn which(bin: &str) -> bool {
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|d| Path::new(d).join(bin).is_file())
}

#[cfg(test)]
mod tests {
    use super::expand_command;

    #[test]
    fn field_codes() {
        assert_eq!(expand_command("gimp"), ("gimp \"$@\"".into(), false));
        assert_eq!(expand_command("mpv %U"), ("mpv \"$@\"".into(), false));
        assert_eq!(expand_command("feh %f"), ("feh \"$1\"".into(), true));
        assert_eq!(expand_command("app %i %c %k %F"), ("app    \"$@\"".into(), false));
        assert_eq!(expand_command("printf 100%% %f"), ("printf 100% \"$1\"".into(), true));
    }
}
