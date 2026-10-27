// Onboarding over ssh: find out what the host runs, put this client's build
// there when it is missing or different, and have it start (or upgrade) its
// server and hand back a token. The client's build is the reference.

use crate::version::{self, Need, Remote};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Default place of rsh on a host, as a shell word.
pub const DEFAULT_REMOTE_BIN: &str = "\"$HOME/.rsh/bin/rsh\"";

/// `--server` as a shell word for the host, with a leading `~/` meaning the
/// remote home.
pub fn remote_bin(path: Option<&str>) -> String {
    match path {
        None => DEFAULT_REMOTE_BIN.to_string(),
        Some(path) => match path.strip_prefix("~/") {
            Some(rest) => format!("\"$HOME\"/{}", shell_quote(rest)),
            None => shell_quote(path),
        },
    }
}

const PROBE_MARK: &str = "RSH-PROBE";

pub struct Ssh {
    command: String,
    host: String,
    control: Option<PathBuf>,
}

/// A short private directory for ssh control sockets: unix socket paths
/// are limited to about 104 bytes, too little under a long home directory.
pub fn control_dir() -> Option<PathBuf> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
    let uid = unsafe { libc::getuid() };
    let dir = PathBuf::from(format!("/tmp/rsh-{uid}"));
    let _ = std::fs::DirBuilder::new().mode(0o700).create(&dir);
    let meta = std::fs::symlink_metadata(&dir).ok()?;
    (meta.is_dir() && meta.uid() == uid && meta.permissions().mode() & 0o077 == 0).then_some(dir)
}

impl Ssh {
    /// `command` is the ssh to run, with any options (`--ssh`). Connections
    /// share one master, when a private socket directory exists, so a
    /// password is asked once per onboarding.
    pub fn new(command: &str, host: &str, control: Option<PathBuf>) -> Self {
        Self {
            command: command.to_string(),
            host: host.to_string(),
            control: control.map(|dir| dir.join("%C")),
        }
    }

    pub fn args(&self, script: &str) -> Vec<String> {
        let mut args: Vec<String> = vec!["-o".into(), "ConnectTimeout=15".into()];
        if let Some(control) = &self.control {
            args.extend([
                "-o".into(),
                "ControlMaster=auto".into(),
                "-o".into(),
                format!("ControlPath={}", control.display()),
                "-o".into(),
                "ControlPersist=60".into(),
            ]);
        }
        args.push(self.host.clone());
        args.push(format!("sh -c {}", shell_quote(script)));
        args
    }

    /// The address ssh would connect to for this host, after its config
    /// (`HostName` of an alias), without connecting.
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn hostname(&self) -> Option<String> {
        let out = Command::new("sh")
            .arg("-c")
            .arg(format!("{} -G \"$@\"", self.command))
            .arg("sh")
            .arg(&self.host)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("hostname ").map(|h| h.trim().to_string()))
            .filter(|h| !h.is_empty())
    }

    /// Runs `script` with the host's `sh`, feeding `input` on stdin. The
    /// user sees ssh's own prompts and errors on the terminal.
    pub fn run(&self, script: &str, input: Option<&[u8]>) -> io::Result<Output> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(format!("{} \"$@\"", self.command))
            .arg("sh")
            .args(self.args(script))
            .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let writer = input.map(|bytes| {
            let mut stdin = child.stdin.take().expect("stdin");
            let bytes = bytes.to_vec();
            // Bytes taken by ssh, on stderr when that is a terminal. The
            // pipe takes 64 KiB before ssh reads anything, so the bar shows
            // only once more than that has gone: ssh is past any password
            // prompt by then. Short enough for 80 columns; cleared on error.
            let bar = indicatif::ProgressBar::with_draw_target(Some(bytes.len() as u64), indicatif::ProgressDrawTarget::hidden());
            if let Ok(style) = indicatif::ProgressStyle::with_template("rsh: uploading {bar:20} {bytes}/{total_bytes} {eta}") {
                bar.set_style(style.progress_chars("=> "));
            }
            std::thread::spawn(move || {
                for chunk in bytes.chunks(16 * 1024) {
                    if let Err(err) = stdin.write_all(chunk) {
                        bar.finish_and_clear();
                        return Err(err);
                    }
                    bar.inc(chunk.len() as u64);
                    if bar.position() > 128 * 1024 && bar.is_hidden() {
                        bar.set_draw_target(indicatif::ProgressDrawTarget::stderr());
                    }
                }
                bar.finish_and_clear();
                Ok::<(), io::Error>(())
            })
        });
        let output = child.wait_with_output()?;
        if let Some(writer) = writer {
            let _ = writer.join();
        }
        Ok(output)
    }
}

pub fn shell_quote(text: &str) -> String {
    if !text.is_empty() && text.chars().all(|c| c.is_ascii_alphanumeric() || "@%+=:,./-_".contains(c)) {
        text.to_string()
    } else {
        format!("'{}'", text.replace('\'', "'\\''"))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Probe {
    pub platform: String,
    pub installed: Option<Remote>,
}

/// One round trip: the host's platform and the version of rsh at `bin`.
pub fn probe(ssh: &Ssh, bin: &str) -> Result<Probe, String> {
    let script = format!(
        "echo {PROBE_MARK}; uname -sm; if command -v timeout >/dev/null 2>&1; then timeout 5 {bin} --version; else {bin} --version; fi 2>/dev/null; true"
    );
    let output = ssh.run(&script, None).map_err(|err| format!("ssh: {err}"))?;
    if !output.status.success() {
        return Err(format!("ssh failed ({})", output.status));
    }
    parse_probe(&String::from_utf8_lossy(&output.stdout)).ok_or_else(|| "unexpected answer from the host".to_string())
}

/// Reads what follows the mark, so whatever a login rc file prints first
/// does not matter.
pub fn parse_probe(text: &str) -> Option<Probe> {
    let mut lines = text.lines().skip_while(|line| line.trim() != PROBE_MARK).skip(1);
    let mut uname = lines.next()?.split_whitespace();
    let platform = version::platform(uname.next()?, uname.next()?);
    let installed = lines.find_map(version::parse);
    Some(Probe { platform, installed })
}

/// Where binaries for other platforms are looked for: `$RSH_DIST` and
/// `~/.rsh/dist`. Whatever is found there is run on every host onboarded,
/// so only files nobody else could have put there count (`trusted`).
pub fn dist_dirs(home: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = std::env::var_os("RSH_DIST") {
        dirs.push(PathBuf::from(dir));
    }
    dirs.push(home.join(".rsh").join("dist"));
    dirs
}

/// Owned by this user and not writable by anyone else, file and directory,
/// after following symlinks to the file that would be read.
pub fn trusted(path: &Path) -> bool {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let uid = unsafe { libc::getuid() };
    let ok = |p: &Path| std::fs::metadata(p).is_ok_and(|m| m.uid() == uid && m.permissions().mode() & 0o022 == 0);
    std::fs::canonicalize(path).is_ok_and(|real| ok(&real) && real.parent().is_some_and(ok))
}

/// A platform name as `uname` gives it to us, before it names a file.
pub fn valid_platform(platform: &str) -> bool {
    let mut parts = platform.split('-');
    let word = |w: Option<&str>| w.is_some_and(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
    word(parts.next()) && word(parts.next()) && parts.next().is_none()
}

pub struct Binary {
    pub bytes: Vec<u8>,
    pub path: PathBuf,
    /// Built from the same sources as this client.
    pub this_build: bool,
}

/// This executable when the host has the same platform, else
/// `rsh-<platform>` from the dist directories, else from this version's
/// release on GitHub (kept in `~/.rsh/dist` for the next host).
pub fn binary_for(platform: &str, home: &Path) -> Result<Binary, String> {
    if !valid_platform(platform) {
        return Err(format!("unexpected platform {platform:?}"));
    }
    if platform == version::local_platform() {
        let path = std::env::current_exe().map_err(|err| err.to_string())?;
        let bytes = std::fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?;
        return Ok(Binary { bytes, path, this_build: true });
    }
    let name = format!("rsh-{platform}");
    let mut stale = None;
    for dir in dist_dirs(home) {
        let path = dir.join(&name);
        if !path.exists() {
            continue;
        }
        if !trusted(&path) {
            return Err(format!("{} is not owned by you or is writable by others; not using it", path.display()));
        }
        if let Ok(bytes) = std::fs::read(&path) {
            let this_build = version::is_this_build(&bytes);
            // Another build at hand (an older one): the release may have
            // this one; if not, that is what the caller decides about.
            if this_build || cfg!(test) {
                return Ok(Binary { bytes, path, this_build });
            }
            stale.get_or_insert(Binary { bytes, path, this_build });
        }
    }
    let tag = crate::release::this_tag();
    let fetched = if cfg!(test) {
        Err("not in tests".to_string())
    } else {
        let _ = writeln!(io::stderr(), "rsh: fetching {name} of {tag} from GitHub");
        crate::release::download(&tag, platform)
    };
    match fetched {
        Ok(bytes) if version::is_this_build(&bytes) => {
            let path = home.join(".rsh").join("dist").join(&name);
            crate::release::install(&bytes, &path)?;
            Ok(Binary { bytes, path, this_build: true })
        }
        _ if stale.is_some() => Ok(stale.expect("stale")),
        Ok(_) => Err(format!("{name} of release {tag} is another build than this rsh; put this build's {name} in ~/.rsh/dist (scripts/dist.sh builds them)")),
        Err(why) => Err(format!("no rsh binary for {platform}: {why}; put {name} in ~/.rsh/dist (scripts/dist.sh builds them)")),
    }
}

/// What onboarding decided to put on the host.
pub fn plan(probe: &Probe, home: &Path) -> Result<Option<Binary>, String> {
    let need = version::need(probe.installed.as_ref());
    if need == Need::Nothing {
        return Ok(None);
    }
    let binary = binary_for(&probe.platform, home)?;
    if !binary.this_build {
        // Only this client's build is ever installed.
        if need == Need::Update {
            let _ = writeln!(
                io::stderr(),
                "rsh: {} is from another build than this rsh; not updating the host (rebuild with scripts/dist.sh)",
                binary.path.display()
            );
            return Ok(None);
        }
        return Err(format!("{} is from another build than this rsh (rebuild with scripts/dist.sh)", binary.path.display()));
    }
    Ok(Some(binary))
}

/// Installs `binary` at `bin` (by rename, so a running server is never
/// rewritten in place) if given, then runs `bin bootstrap`; returns the
/// port and token it prints.
/// This client's name for its token on hosts: `user@hostname`.
pub fn client_name() -> String {
    let mut buf = [0u8; 256];
    let host = unsafe {
        if libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) == 0 {
            let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
            String::from_utf8_lossy(&buf[..end]).into_owned()
        } else {
            "unknown".into()
        }
    };
    let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
    format!("{user}@{host}")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "@._-".contains(*c))
        .take(64)
        .collect()
}

pub fn install_and_bootstrap(ssh: &Ssh, bin: &str, binary: Option<&Binary>) -> Result<(u16, String), String> {
    let client = shell_quote(&client_name());
    // The upload goes to a name of its own and replaces rsh only when it
    // arrived whole: an interrupted copy must never become the binary.
    let script = match binary {
        Some(binary) => format!(
            "set -e; b={bin}; t=\"$b.new.$$\"; mkdir -p \"$(dirname \"$b\")\"; cat > \"$t\"; \
             if [ \"$(wc -c < \"$t\" | tr -d ' ')\" != {size} ]; then rm -f \"$t\"; echo 'rsh: upload incomplete' >&2; exit 1; fi; \
             chmod 755 \"$t\"; mv -f \"$t\" \"$b\"; exec \"$b\" bootstrap --client {client}",
            size = binary.bytes.len()
        ),
        None => format!("exec {bin} bootstrap --client {client}"),
    };
    let output = ssh
        .run(&script, binary.map(|b| b.bytes.as_slice()))
        .map_err(|err| format!("ssh: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    for note in text.lines().filter_map(|line| line.strip_prefix("RSH NOTE ")) {
        eprintln!("rsh: {}: {}", ssh.host(), crate::cli::printable(note));
    }
    parse_connect(&text).ok_or_else(|| format!("rsh bootstrap on the host failed ({})", output.status))
}

pub fn parse_connect(text: &str) -> Option<(u16, String)> {
    text.lines().find_map(|line| {
        let mut words = line.strip_prefix("RSH CONNECT ")?.split_whitespace();
        let port = words.next()?.parse::<u16>().ok()?;
        let token = words.next()?.to_string();
        Some((port, token))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_plan_and_quoting() {
        let probe = parse_probe(&format!("Welcome!\nLinux box\n{PROBE_MARK}\nLinux x86_64\n{}\n", version::describe())).unwrap();
        assert_eq!(probe.platform, "linux-x86_64");
        assert_eq!(version::need(probe.installed.as_ref()), Need::Nothing);
        assert!(parse_probe("Linux x86_64\n").is_none(), "no mark, no answer");
        let empty = parse_probe(&format!("{PROBE_MARK}\nLinux aarch64\n")).unwrap();
        assert_eq!(empty.installed, None);
        let home = std::env::temp_dir().join(format!("rsh-remote-{}", std::process::id()));
        let local = Probe {
            platform: version::local_platform(),
            installed: version::parse("rsh 0.0.1"),
        };
        let binary = plan(&local, &home).unwrap().expect("old build is replaced");
        assert!(binary.this_build);
        assert!(plan(&Probe { platform: "plan9-mips".into(), installed: None }, &home).is_err());
        assert!(binary_for("../x-y", &home).is_err());
        assert!(valid_platform("linux-x86_64") && !valid_platform("linux-x86_64/..") && !valid_platform("a-b-c"));
        assert!(client_name().contains('@'));
        let same = Probe {
            platform: "plan9-mips".into(),
            installed: version::parse(&version::describe()),
        };
        assert!(plan(&same, &home).unwrap().is_none(), "nothing to do needs no binary");
        assert_eq!(parse_connect("noise\nRSH CONNECT 2222 abc\n"), Some((2222, "abc".into())));
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        assert_eq!(shell_quote(""), "''");
        let ssh = Ssh::new("ssh -p 22", "me@box", Some(PathBuf::from("/tmp/x")));
        let args = ssh.args("echo hi");
        assert_eq!(args[args.len() - 2], "me@box");
        assert_eq!(args[args.len() - 1], "sh -c 'echo hi'");
        assert!(args.contains(&"ControlPath=/tmp/x/%C".to_string()));
        assert!(!Ssh::new("ssh", "h", None).args("x").iter().any(|a| a.contains("Control")));
        let dir = control_dir().expect("private socket dir");
        assert!(dir.join("0123456789012345678901234567890123456789").to_string_lossy().len() + 20 < 104);
        assert_eq!(remote_bin(None), DEFAULT_REMOTE_BIN);
        assert_eq!(remote_bin(Some("~/bin/my rsh")), "\"$HOME\"/'bin/my rsh'");
        assert_eq!(remote_bin(Some("/opt/rsh")), "/opt/rsh");
        let named = Ssh::new("ssh -F /dev/null -o HostName=10.1.2.3", "alias", None);
        assert_eq!(named.hostname().as_deref(), Some("10.1.2.3"));
    }
}
