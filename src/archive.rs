use std::fs;
use std::path::Path;

/// Suffixes bsdtar (libarchive) can extract.
const READABLE: &[&str] = &[
    ".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst", ".tar.lz4", ".tar.lzma", ".tar.lz", ".tgz",
    ".tbz", ".tbz2", ".txz", ".tzst", ".tar", ".zip", ".jar", ".xpi", ".7z", ".rar", ".iso",
    ".cpio", ".cab", ".deb", ".rpm",
];

/// Archive name without its archive suffix ("foo.tar.gz" -> "foo"), or None
/// when the name isn't a recognised archive.
pub fn stem(name: &str) -> Option<&str> {
    let lower = name.to_ascii_lowercase();
    READABLE
        .iter()
        .find(|s| lower.ends_with(*s) && lower.len() > s.len())
        .map(|s| &name[..name.len() - s.len()])
}

/// bsdtar flags that write the format named by `target`'s suffix.
pub fn create_flags(target: &Path) -> Option<&'static [&'static str]> {
    let name = target.file_name()?.to_string_lossy().to_ascii_lowercase();
    let flags: &[&str] = if name.ends_with(".zip") {
        &["--format", "zip"]
    } else if name.ends_with(".7z") {
        &["--format", "7zip"]
    } else if name.ends_with(".tar.gz") {
        &["-z"]
    } else if name.ends_with(".tar.xz") {
        &["-J"]
    } else if name.ends_with(".tar.zst") {
        &["--zstd"]
    } else if name.ends_with(".tar") {
        &[]
    } else {
        return None;
    };
    Some(flags)
}

/// Bytes a process has read so far (`rchar`), used as bsdtar's progress.
pub fn bytes_read(pid: u32) -> Option<u64> {
    let io = fs::read_to_string(format!("/proc/{pid}/io")).ok()?;
    io.lines()
        .find_map(|l| l.strip_prefix("rchar:"))
        .and_then(|v| v.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems() {
        assert_eq!(stem("foo.tar.gz"), Some("foo"));
        assert_eq!(stem("Foo.ZIP"), Some("Foo"));
        assert_eq!(stem("a.b.7z"), Some("a.b"));
        assert_eq!(stem(".zip"), None);
        assert_eq!(stem("notes.txt"), None);
        assert_eq!(stem("x.gz"), None);
    }

    #[test]
    fn formats() {
        assert_eq!(create_flags(Path::new("/t/a.tar.zst")), Some(&["--zstd"][..]));
        assert_eq!(create_flags(Path::new("/t/a.tar")), Some(&[][..]));
        assert!(create_flags(Path::new("/t/a.rar")).is_none());
    }
}
