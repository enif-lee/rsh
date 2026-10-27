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
        .args(["-fsSL", "--max-time", &timeout.as_secs().to_string(), "-o"])
        .arg(dest)
        .arg(url)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// `rsh-<platform>` of release `tag`, checked against the release's
/// SHA256SUMS.
pub fn download(tag: &str, platform: &str) -> Result<Vec<u8>, String> {
    let dir = std::env::temp_dir().join(format!("rsh-release-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
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

/// Writes `bytes` as an executable at `path`, by rename: a running copy is
/// never rewritten in place.
pub fn install(bytes: &[u8], path: &Path) -> Result<(), String> {
    let dir = path.parent().ok_or("no directory")?;
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
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

/// The install this client manages: `~/.rsh/bin/rsh`, when it is what runs.
fn managed_exe(home: &Path) -> Option<PathBuf> {
    let exe = fs::canonicalize(std::env::current_exe().ok()?).ok()?;
    let managed = fs::canonicalize(home.join(".rsh").join("bin").join("rsh")).ok()?;
    (exe == managed).then_some(managed)
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
    match download(&tag, &version::local_platform()).and_then(|bytes| install(&bytes, &exe)) {
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

    /// Needs the network and this version's release: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn this_release_carries_this_build() {
        for platform in ["linux-x86_64", "linux-aarch64", "darwin-aarch64", "darwin-x86_64"] {
            let bytes = download(&this_tag(), platform).unwrap();
            assert!(version::is_this_build(&bytes), "{platform}");
        }
    }
}
