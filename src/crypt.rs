//! Password encryption in the age format (scrypt passphrase recipient,
//! ChaCha20-Poly1305 payload). A folder's contents are packed into a tar
//! stream stored as `VAULT` inside the folder; a single file becomes
//! `<name>.age`. Both decrypt with the stock `age -d` tool.

use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::iter;
use std::path::Path;

use age::secrecy::SecretString;

pub const VAULT: &str = ".3la-vault.age";
/// Names this module creates inside a folder; never packed into a vault.
const OWN_PREFIX: &str = ".3la-";

#[derive(Debug)]
pub enum CryptError {
    WrongPassword,
    NotEncrypted,
    Cancelled,
    Io(String),
}

impl fmt::Display for CryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptError::WrongPassword => f.write_str("wrong password"),
            CryptError::NotEncrypted => f.write_str("not a password-encrypted age file"),
            CryptError::Cancelled => f.write_str("cancelled"),
            CryptError::Io(e) => f.write_str(e),
        }
    }
}

impl From<io::Error> for CryptError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::Other && e.to_string() == CANCELLED {
            CryptError::Cancelled
        } else {
            CryptError::Io(e.to_string())
        }
    }
}

impl From<age::DecryptError> for CryptError {
    fn from(e: age::DecryptError) -> Self {
        match e {
            age::DecryptError::DecryptionFailed | age::DecryptError::NoMatchingKeys => CryptError::WrongPassword,
            age::DecryptError::InvalidHeader | age::DecryptError::UnknownFormat => CryptError::NotEncrypted,
            other => CryptError::Io(other.to_string()),
        }
    }
}

const CANCELLED: &str = "3la: cancelled";

/// Progress hook: told how many bytes just went by; returning false cancels.
pub type Tick<'a> = &'a mut dyn FnMut(u64) -> bool;

/// Reader that reports progress and fails once the operation is cancelled.
/// (Not ErrorKind::Interrupted: io::copy would silently retry that.)
struct Meter<'a, R> {
    inner: R,
    tick: Tick<'a>,
}

impl<R: Read> Read for Meter<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if !(self.tick)(n as u64) {
            return Err(io::Error::other(CANCELLED));
        }
        Ok(n)
    }
}

fn writer(out: &Path, pass: &SecretString) -> io::Result<age::stream::StreamWriter<BufWriter<File>>> {
    let file = File::options().write(true).create_new(true).open(out)?;
    age::Encryptor::with_user_passphrase(pass.clone()).wrap_output(BufWriter::new(file))
}

fn close(w: age::stream::StreamWriter<BufWriter<File>>) -> io::Result<()> {
    let file = w.finish()?.into_inner().map_err(|e| e.into_error())?;
    file.sync_all()
}

/// Opens an age file for reading with `pass`; header problems (wrong
/// password, not age) surface here before any plaintext is produced.
fn reader<'a>(src: &Path, pass: &SecretString, tick: Tick<'a>) -> Result<impl Read + 'a, CryptError> {
    let input = BufReader::new(Meter { inner: File::open(src)?, tick });
    let dec = age::Decryptor::new_buffered(input)?;
    if !dec.is_scrypt() {
        return Err(CryptError::NotEncrypted);
    }
    let id = age::scrypt::Identity::new(pass.clone());
    Ok(dec.decrypt(iter::once(&id as &dyn age::Identity))?)
}

/// Packs everything in `dir` (except this module's own files) into `out`.
pub fn encrypt_dir(dir: &Path, pass: &SecretString, out: &Path, tick: Tick) -> Result<(), CryptError> {
    let mut tar = tar::Builder::new(writer(out, pass)?);
    tar.follow_symlinks(false);
    pack(&mut tar, dir, Path::new(""), tick)?;
    close(tar.into_inner()?)?;
    Ok(())
}

fn pack<W: Write>(tar: &mut tar::Builder<W>, dir: &Path, rel: &Path, tick: Tick) -> Result<(), CryptError> {
    for d in fs::read_dir(dir)? {
        let d = d?;
        let name = d.file_name();
        if rel.as_os_str().is_empty() && name.to_string_lossy().starts_with(OWN_PREFIX) {
            continue;
        }
        let path = d.path();
        let rel = rel.join(&name);
        let meta = fs::symlink_metadata(&path)?;
        let ft = meta.file_type();
        if ft.is_dir() {
            tar.append_dir(&rel, &path)?;
            pack(tar, &path, &rel, tick)?;
        } else if ft.is_file() {
            let mut header = tar::Header::new_gnu();
            header.set_metadata_in_mode(&meta, tar::HeaderMode::Complete);
            let file = Meter { inner: File::open(&path)?, tick: &mut *tick };
            tar.append_data(&mut header, &rel, file)?;
        } else if ft.is_symlink() {
            tar.append_path_with_name(&path, &rel)?;
        } else {
            // Sockets, FIFOs and devices can't be stored; refusing keeps the
            // originals, since nothing is deleted unless all of it was packed.
            return Err(CryptError::Io(format!("{}: unsupported file type", path.display())));
        }
    }
    Ok(())
}

pub fn encrypt_file(src: &Path, pass: &SecretString, out: &Path, tick: Tick) -> Result<(), CryptError> {
    let mut w = writer(out, pass)?;
    io::copy(&mut Meter { inner: File::open(src)?, tick }, &mut w)?;
    close(w)?;
    Ok(())
}

/// Reads `src` back in full, so a write problem shows up before the
/// originals are deleted. A vault is also parsed as tar.
pub fn verify(src: &Path, pass: &SecretString, vault: bool, tick: Tick) -> Result<(), CryptError> {
    let mut r = reader(src, pass, tick)?;
    if vault {
        let mut archive = tar::Archive::new(r);
        for entry in archive.entries()? {
            io::copy(&mut entry?, &mut io::sink())?;
        }
    } else {
        io::copy(&mut r, &mut io::sink())?;
    }
    Ok(())
}

/// Unpacks a vault into the empty directory `dest`.
pub fn decrypt_dir(vault: &Path, pass: &SecretString, dest: &Path, tick: Tick) -> Result<(), CryptError> {
    let mut archive = tar::Archive::new(reader(vault, pass, tick)?);
    archive.set_preserve_permissions(true);
    archive.set_preserve_mtime(true);
    archive.set_overwrite(false);
    // unpack() refuses entries that would land outside `dest`.
    archive.unpack(dest)?;
    Ok(())
}

pub fn decrypt_file(src: &Path, pass: &SecretString, out: &Path, tick: Tick) -> Result<(), CryptError> {
    let mut r = reader(src, pass, tick)?;
    let mut file = File::options().write(true).create_new(true).open(out)?;
    io::copy(&mut r, &mut file)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("3la-crypt-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn go(_: u64) -> bool {
        true
    }

    #[test]
    fn dir_round_trip() {
        let root = scratch("dir");
        let src = root.join("src");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.txt"), "alpha").unwrap();
        fs::write(src.join("sub/b.bin"), vec![7u8; 100_000]).unwrap();
        std::os::unix::fs::symlink("a.txt", src.join("link")).unwrap();
        let pass = SecretString::from("correct horse");
        let vault = src.join(VAULT);
        encrypt_dir(&src, &pass, &vault, &mut go).unwrap();
        verify(&vault, &pass, true, &mut go).unwrap();
        assert!(matches!(
            verify(&vault, &SecretString::from("nope"), true, &mut go),
            Err(CryptError::WrongPassword)
        ));
        let out = root.join("out");
        fs::create_dir(&out).unwrap();
        decrypt_dir(&vault, &pass, &out, &mut go).unwrap();
        assert_eq!(fs::read_to_string(out.join("a.txt")).unwrap(), "alpha");
        assert_eq!(fs::read(out.join("sub/b.bin")).unwrap().len(), 100_000);
        assert_eq!(fs::read_link(out.join("link")).unwrap(), Path::new("a.txt"));
        assert!(!out.join(VAULT).exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn file_round_trip_and_cancel() {
        let root = scratch("file");
        let src = root.join("doc.txt");
        fs::write(&src, "secret text").unwrap();
        let pass = SecretString::from("pw12345678");
        let enc = root.join("doc.txt.age");
        encrypt_file(&src, &pass, &enc, &mut go).unwrap();
        assert!(!fs::read(&enc).unwrap().windows(6).any(|w| w == b"secret"));
        let out = root.join("doc.out");
        decrypt_file(&enc, &pass, &out, &mut go).unwrap();
        assert_eq!(fs::read_to_string(&out).unwrap(), "secret text");
        let mut stop = |_| false;
        assert!(matches!(
            encrypt_file(&src, &pass, &root.join("c.age"), &mut stop),
            Err(CryptError::Cancelled)
        ));
        fs::remove_dir_all(&root).unwrap();
    }
}
