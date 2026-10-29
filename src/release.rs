// Releases on GitHub: binaries of a version for every platform, with their
// SHA-256 sums. A client fetches from there what it needs and has no copy
// of (a host's platform), and moves its own install to a newer release.

use crate::version;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

const REPO: &str = "https://github.com/enif-lee/rsh";
/// How often a client looks for a newer release.
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// The release this client was built as.
pub fn this_tag() -> String {
    format!("v{}", version::VERSION)
}

/// `url` into `dest` with curl; false if it could not be had.
fn fetch(url: &str, dest: &Path, timeout: Duration) -> bool {
    Command::new("curl")
        .args(["-fsSL", "--connect-timeout", "5", "--max-time", &timeout.as_secs().to_string(), "-o"])
        .arg(dest)
        .arg(url)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// `rsh-<platform>` of release `tag`, checked against the release's
/// SHA256SUMS. Downloaded into a directory of this user's own under
/// `~/.rsh`, made new (0700, never one already there): in a shared /tmp
/// another user could swap the files between download and check.
pub fn download(tag: &str, platform: &str, home: &Path) -> Result<Vec<u8>, String> {
    use std::os::unix::fs::DirBuilderExt;
    let rsh = home.join(".rsh");
    crate::net::private_dir(&rsh).map_err(|err| format!("{}: {err}", rsh.display()))?;
    let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let dir = rsh.join(format!("download-{}-{nanos}", std::process::id()));
    fs::DirBuilder::new().mode(0o700).create(&dir).map_err(|err| format!("{}: {err}", dir.display()))?;
    let name = format!("rsh-{platform}");
    let (file, sums) = (dir.join(&name), dir.join("SHA256SUMS"));
    let base = format!("{REPO}/releases/download/{tag}");
    let got = fetch(&format!("{base}/{name}"), &file, Duration::from_secs(120)) && fetch(&format!("{base}/SHA256SUMS"), &sums, Duration::from_secs(30));
    let read = (fs::read(&file), fs::read_to_string(&sums));
    let _ = fs::remove_dir_all(&dir);
    let (Ok(bytes), Ok(sums)) = read else {
        return Err(format!("could not download {name} of release {tag} from {REPO}"));
    };
    if !got {
        return Err(format!("could not download {name} of release {tag} from {REPO}"));
    }
    let sum: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    let listed = sums.lines().any(|line| line.split_whitespace().collect::<Vec<_>>() == [sum.as_str(), name.as_str()]);
    if !listed {
        return Err(format!("{name} of release {tag} does not match its SHA256SUMS"));
    }
    Ok(bytes)
}

/// How long a failed download of a host's binary (no network, or a release
/// still spreading: its files can answer 404 for minutes) keeps a connect
/// from trying again. A release of another build (a build of one's own)
/// is not tried again for `CHECK_EVERY`.
const RETRY_AFTER: Duration = Duration::from_secs(10 * 60);

/// Marks that this build's `rsh-<platform>` could not be had from its
/// release: `other_build` when the release has another build, else the
/// download failed.
pub fn note_unavailable(home: &Path, platform: &str, other_build: bool) {
    let why = if other_build { "other" } else { "failed" };
    let _ = fs::write(unavailable_stamp(home, platform), format!("{why} {}", version::build()));
}

/// Whether `note_unavailable` was said of this build lately.
pub fn unavailable(home: &Path, platform: &str) -> bool {
    let stamp = unavailable_stamp(home, platform);
    let Ok(text) = fs::read_to_string(&stamp) else {
        return false;
    };
    let wait = match text.split_once(' ') {
        Some(("other", build)) if build == version::build() => CHECK_EVERY,
        Some(("failed", build)) if build == version::build() => RETRY_AFTER,
        _ => return false,
    };
    fs::metadata(&stamp)
        .and_then(|meta| meta.modified())
        .is_ok_and(|at| SystemTime::now().duration_since(at).is_ok_and(|age| age < wait))
}

fn unavailable_stamp(home: &Path, platform: &str) -> PathBuf {
    home.join(".rsh").join(format!("unavailable-{platform}"))
}

/// Writes `bytes` as an executable at `path`, by rename: a running copy is
/// never rewritten in place.
pub fn install(bytes: &[u8], path: &Path) -> Result<(), String> {
    use std::os::unix::fs::DirBuilderExt;
    let dir = path.parent().ok_or("no directory")?;
    // 0755 whatever the umask: a group-writable directory is not trusted
    // for what is installed from it (`remote::trusted`).
    fs::DirBuilder::new().recursive(true).mode(0o755).create(dir).map_err(|err| format!("{}: {err}", dir.display()))?;
    let tmp = path.with_extension("new");
    let mut file = fs::File::create(&tmp).map_err(|err| format!("{}: {err}", tmp.display()))?;
    file.write_all(bytes).map_err(|err| err.to_string())?;
    file.set_permissions(fs::Permissions::from_mode(0o755)).map_err(|err| err.to_string())?;
    drop(file);
    fs::rename(&tmp, path).map_err(|err| format!("{}: {err}", path.display()))
}

/// The newest release's tag (`v1.2.3`), from where GitHub sends
/// `releases/latest`.
fn latest_tag() -> Option<String> {
    let out = Command::new("curl")
        .args(["-fsSIL", "--max-time", "3", "-o", "/dev/null", "-w", "%{url_effective}"])
        .arg(format!("{REPO}/releases/latest"))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let url = String::from_utf8(out.stdout).ok()?;
    let tag = url.rsplit_once("/tag/")?.1.trim().to_string();
    numbers(&tag).map(|_| tag)
}

/// `v1.2.3` or `1.2.3` as numbers.
fn numbers(version: &str) -> Option<Vec<u64>> {
    version.trim_start_matches('v').split('.').map(|part| part.parse().ok()).collect()
}

/// Whether `tag` is a newer version than this client.
fn is_newer(tag: &str) -> bool {
    matches!((numbers(tag), numbers(version::VERSION)), (Some(new), Some(me)) if new > me)
}

/// The install this client manages: `~/.rsh/bin/rsh`, a file of its own
/// (not a link to a build elsewhere, which an upgrade would overwrite),
/// when it is what runs.
fn managed_exe(home: &Path) -> Option<PathBuf> {
    let managed = home.join(".rsh").join("bin").join("rsh");
    if !fs::symlink_metadata(&managed).ok()?.file_type().is_file() {
        return None;
    }
    let exe = fs::canonicalize(std::env::current_exe().ok()?).ok()?;
    (exe == fs::canonicalize(&managed).ok()?).then_some(managed)
}

/// Once a day at most, and only for the install it manages (not a build
/// run from elsewhere, not with RSH_NO_UPDATE set): when a newer release
/// is out, puts it in place and runs it with the same arguments. Anything
/// that does not work out leaves this client to carry on as it is.
pub fn upgrade_and_rerun(home: &Path, args: &[String]) {
    if std::env::var_os("RSH_NO_UPDATE").is_some() {
        return;
    }
    let Some(exe) = managed_exe(home) else {
        return;
    };
    let stamp = home.join(".rsh").join("update-check");
    let recent = fs::metadata(&stamp)
        .and_then(|meta| meta.modified())
        .is_ok_and(|at| SystemTime::now().duration_since(at).is_ok_and(|age| age < CHECK_EVERY));
    if recent {
        return;
    }
    let _ = fs::write(&stamp, b"");
    let Some(tag) = latest_tag().filter(|tag| is_newer(tag)) else {
        return;
    };
    let mut err = std::io::stderr();
    let _ = writeln!(err, "rsh: upgrading to {tag}");
    match download(&tag, &version::local_platform(), home).and_then(|bytes| install(&bytes, &exe)) {
        Ok(()) => {
            let failed = Command::new(&exe).args(args).env("RSH_NO_UPDATE", "1").exec();
            let _ = writeln!(err, "rsh: could not run {}: {failed}", exe.display());
        }
        Err(why) => {
            let _ = writeln!(err, "rsh: not upgraded: {why}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_as_numbers() {
        assert_eq!(numbers("v1.10.2"), Some(vec![1, 10, 2]));
        assert_eq!(numbers("v1.x"), None);
        assert!(is_newer("v999.0.0"));
        assert!(!is_newer(&this_tag()));
        assert!(!is_newer("v0.0.1"));
    }

    #[test]
    fn a_failed_download_is_tried_again_sooner_than_another_build() {
        let home = std::env::temp_dir().join(format!("rsh-unavailable-{}", std::process::id()));
        fs::create_dir_all(home.join(".rsh")).unwrap();
        let aged = |minutes: u64| {
            let file = fs::File::options().write(true).open(unavailable_stamp(&home, "linux-x86_64")).unwrap();
            file.set_modified(SystemTime::now() - Duration::from_secs(minutes * 60)).unwrap();
        };
        assert!(!unavailable(&home, "linux-x86_64"));
        note_unavailable(&home, "linux-x86_64", false);
        assert!(unavailable(&home, "linux-x86_64"));
        aged(11);
        assert!(!unavailable(&home, "linux-x86_64"), "a failure is tried again after 10 minutes");
        note_unavailable(&home, "linux-x86_64", true);
        aged(11);
        assert!(unavailable(&home, "linux-x86_64"), "another build waits a day");
        fs::write(unavailable_stamp(&home, "linux-x86_64"), "other 000000000000").unwrap();
        assert!(!unavailable(&home, "linux-x86_64"), "said of another build of this client");
        let _ = fs::remove_dir_all(&home);
    }

    /// Needs the network and this version's release: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn this_release_carries_this_build() {
        for platform in ["linux-x86_64", "linux-aarch64", "darwin-aarch64", "darwin-x86_64"] {
            let home = std::env::temp_dir().join(format!("rsh-release-test-{}", std::process::id()));
            let bytes = download(&this_tag(), platform, &home).unwrap();
            let _ = fs::remove_dir_all(&home);
            assert!(version::is_this_build(&bytes), "{platform}");
        }
    }
}
