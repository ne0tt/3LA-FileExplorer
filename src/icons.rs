use std::path::Path;
use std::sync::OnceLock;

use crate::fsutil::{self, Entry};

// Nerd Font Material Design glyphs (verified against glyphnames.json).
pub const FOLDER: &str = "\u{F024B}";
pub const FOLDER_HOME: &str = "\u{F10B5}";
pub const FOLDER_DOWNLOAD: &str = "\u{F024D}";
pub const FOLDER_MUSIC: &str = "\u{F1359}";
pub const FOLDER_IMAGE: &str = "\u{F024F}";
pub const FOLDER_PLAY: &str = "\u{F19FA}";
pub const FOLDER_TEXT: &str = "\u{F0C82}";
pub const FOLDER_COG: &str = "\u{F107F}";
pub const FOLDER_STAR: &str = "\u{F069D}";
pub const FOLDER_KEY: &str = "\u{F08AC}";
pub const FOLDER_ACCOUNT: &str = "\u{F024C}";
pub const FOLDER_LOCK: &str = "\u{F0250}";
pub const FOLDER_WRENCH: &str = "\u{F19FC}";
pub const FOLDER_NETWORK: &str = "\u{F0870}";
pub const DESKTOP: &str = "\u{F01C4}";
pub const GIT: &str = "\u{F02A2}";
pub const NODEJS: &str = "\u{F0399}";


struct UserDirs {
    home: String,
    special: Vec<(String, &'static str)>,
}

fn user_dirs() -> &'static UserDirs {
    static U: OnceLock<UserDirs> = OnceLock::new();
    U.get_or_init(|| {
        let mut special = Vec::new();
        let mut add = |p: Option<std::path::PathBuf>, g: &'static str| {
            if let Some(p) = p {
                special.push((p.to_string_lossy().into_owned(), g));
            }
        };
        add(dirs::desktop_dir(), DESKTOP);
        add(dirs::download_dir(), FOLDER_DOWNLOAD);
        add(dirs::document_dir(), FOLDER_TEXT);
        add(dirs::audio_dir(), FOLDER_MUSIC);
        add(dirs::picture_dir(), FOLDER_IMAGE);
        add(dirs::video_dir(), FOLDER_PLAY);
        add(dirs::template_dir(), FOLDER_STAR);
        add(dirs::public_dir(), FOLDER_ACCOUNT);
        add(dirs::config_dir(), FOLDER_COG);
        UserDirs {
            home: fsutil::home().to_string_lossy().into_owned(),
            special,
        }
    })
}

pub fn for_folder_path(path: &str) -> &'static str {
    let u = user_dirs();
    if path == u.home {
        return FOLDER_HOME;
    }
    if let Some((_, g)) = u.special.iter().find(|(p, _)| p == path) {
        return g;
    }
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match name.as_str() {
        ".git" => GIT,
        "node_modules" => NODEJS,
        ".ssh" | ".gnupg" => FOLDER_KEY,
        ".cache" | "target" | "build" => FOLDER_WRENCH,
        ".local" | ".config" => FOLDER_COG,
        "lost+found" | "root" => FOLDER_LOCK,
        "Network" | "gvfs" => FOLDER_NETWORK,
        _ => FOLDER,
    }
}

// File icons, matching SPX-FileExplorer: a small set of Font Awesome glyphs
// chosen from the extension-derived mime type.
pub const SPX_FOLDER: &str = "\u{F07B}";
const SPX_IMAGE: &str = "\u{F03E}";
const SPX_VIDEO: &str = "\u{F03D}";
const SPX_AUDIO: &str = "\u{F001}";
const SPX_TEXT: &str = "\u{F15C}";
const SPX_PDF: &str = "\u{F1C1}";
const SPX_ARCHIVE: &str = "\u{F1C6}";
const SPX_DATA: &str = "\u{F1C9}";
const SPX_CONFIG: &str = "\u{F013}";
const SPX_FILE: &str = "\u{F15B}";
/// Material "file-lock" / "folder-lock": encrypted items.
const SPX_FILE_LOCKED: &str = "\u{F0221}";
pub const SPX_FOLDER_LOCKED: &str = "\u{F0250}";

/// Returns (glyph, category) the way SPX-FileExplorer picks its icon, from
/// `ext_mime` (the mime type guessed from the name alone). The category
/// drives thumbnails and colours in QML.
pub fn for_entry(e: &Entry, ext_mime: &str) -> (&'static str, &'static str) {
    if e.is_dir {
        return (SPX_FOLDER, "folder");
    }
    if e.name.to_lowercase().ends_with(".age") {
        return (SPX_FILE_LOCKED, "encrypted");
    }
    let mime = ext_mime.to_lowercase();
    let has = |tokens: &[&str]| tokens.iter().any(|t| mime.contains(t));
    if mime.starts_with("image/") {
        (SPX_IMAGE, "image")
    } else if mime.starts_with("video/") {
        (SPX_VIDEO, "video")
    } else if mime.starts_with("audio/") {
        (SPX_AUDIO, "audio")
    } else if mime.starts_with("text/") {
        (SPX_TEXT, "text")
    } else if mime.contains("pdf") {
        (SPX_PDF, "document")
    } else if has(&["zip", "archive", "compressed", "tar", "gz"]) {
        (SPX_ARCHIVE, "archive")
    } else if has(&["json", "xml", "yaml"]) {
        (SPX_DATA, "config")
    } else if has(&["executable", "x-elf"]) {
        (SPX_CONFIG, "exec")
    } else if [".md", ".txt", ".rst"].iter().any(|x| e.name.to_lowercase().ends_with(x)) {
        (SPX_TEXT, "text")
    } else {
        (SPX_FILE, "file")
    }
}
