// Live upgrade of `rsh serve`: on SIGUSR2 the server writes its state to an
// unlinked file, keeps the UDP socket, every pty master and that file open
// across execve(2), and becomes the binary now at its path. The pid stays
// the same, so the shells remain its children; clients see one repaint.

use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Environment variable naming the fd that holds the handed-over state.
pub const ENV_FD: &str = "RSH_HANDOFF_FD";

static REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn on_usr2(_: libc::c_int) {
    REQUESTED.store(true, Ordering::Relaxed);
}

/// SIGUSR2 asks the server to upgrade; no SA_RESTART so poll() wakes.
pub fn install_handler() {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = on_usr2 as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        libc::sigaction(libc::SIGUSR2, &action, std::ptr::null_mut());
    }
}

pub fn take_request() -> bool {
    REQUESTED.swap(false, Ordering::Relaxed)
}

fn mask_usr2(how: libc::c_int) {
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, libc::SIGUSR2);
        libc::pthread_sigmask(how, &set, std::ptr::null_mut());
    }
}

/// Lets SIGUSR2 in again once the handler is installed. An upgrade blocks
/// it across execve, where its default action would kill the server.
pub fn unblock_request() {
    mask_usr2(libc::SIG_UNBLOCK);
}

/// `serve-<port>.pid` next to the auth file: one server per port.
pub fn pidfile(auth: &Path, port: u16) -> PathBuf {
    auth.parent().unwrap_or(Path::new(".")).join(format!("serve-{port}.pid"))
}

/// Records this process, when it (re)started and the binary an upgrade
/// will exec; an upgrade rewrites it.
pub fn write_pidfile(path: &Path, exe: &Path) -> io::Result<()> {
    let started = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    std::fs::write(path, format!("{} {started} {}\n", std::process::id(), exe.display()))
}

pub struct Running {
    pub pid: i32,
    pub text: String,
    pub exe: PathBuf,
}

/// The server a pidfile names, if that pid is still an rsh process; a stale
/// file whose pid now belongs to something else is ignored.
pub fn running(path: &Path) -> Option<Running> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut words = text.trim_end().splitn(3, ' ');
    let pid: i32 = words.next()?.parse().ok()?;
    let _started = words.next()?;
    let exe = PathBuf::from(words.next().unwrap_or(""));
    if unsafe { libc::kill(pid, 0) } != 0 {
        return None;
    }
    is_rsh_serve(&process_args(pid)?).then_some(Running { pid, text, exe })
}

/// The command line of a process: `/proc` where there is one (no procps
/// needed), `ps` elsewhere.
pub fn process_args(pid: i32) -> Option<Vec<String>> {
    if let Ok(raw) = std::fs::read(format!("/proc/{pid}/cmdline")) {
        let args: Vec<String> = raw
            .split(|b| *b == 0)
            .filter(|part| !part.is_empty())
            .map(|part| String::from_utf8_lossy(part).into_owned())
            .collect();
        return (!args.is_empty()).then_some(args);
    }
    let out = Command::new("ps").args(["-o", "args=", "-p", &pid.to_string()]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!text.is_empty()).then(|| vec![text])
}

/// Pids of this user's processes.
pub fn own_pids() -> Vec<i32> {
    use std::os::unix::fs::MetadataExt;
    let uid = unsafe { libc::getuid() };
    if let Ok(entries) = std::fs::read_dir("/proc") {
        return entries
            .flatten()
            .filter_map(|entry| {
                let pid = entry.file_name().to_str()?.parse::<i32>().ok()?;
                (entry.metadata().ok()?.uid() == uid).then_some(pid)
            })
            .collect();
    }
    Command::new("ps")
        .args(["-U", &uid.to_string(), "-o", "pid="])
        .output()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .filter_map(|pid| pid.parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// True for the command line of an `rsh serve` process itself (not of a
/// shell or ssh that merely mentions it).
pub fn is_rsh_serve(args: &[String]) -> bool {
    let joined = args.join(" ");
    let mut words = joined.split_whitespace();
    let program = words.next().map(|w| Path::new(w).file_name().unwrap_or_default().to_string_lossy().into_owned());
    program.is_some_and(|name| name.starts_with("rsh")) && words.next() == Some("serve")
}

/// The port an `rsh serve` command line listens on.
pub fn serve_port(args: &[String], default: u16) -> u16 {
    let joined = args.join(" ");
    let words: Vec<&str> = joined.split_whitespace().collect();
    words
        .windows(2)
        .find(|pair| pair[0] == "--bind")
        .and_then(|pair| pair[1].rsplit(':').next()?.parse().ok())
        .unwrap_or(default)
}

/// `--version` of the binary a process runs (on Linux the very file it was
/// started from, even if replaced since), or None if it does not answer
/// like an rsh within three seconds.
pub fn process_version(pid: i32, args: &[String]) -> Option<String> {
    let proc_exe = PathBuf::from(format!("/proc/{pid}/exe"));
    let exe = if proc_exe.exists() {
        proc_exe
    } else {
        PathBuf::from(args.join(" ").split_whitespace().next()?)
    };
    let (ok, text) = run_with_timeout(Command::new(exe).arg("--version"), 3)?;
    let line = text.lines().next()?.trim().to_string();
    (ok && line.starts_with("rsh ")).then_some(line)
}

/// Runs a command with no stdin and returns its success and stdout, or
/// None if it could not start or did not finish within `secs`.
pub fn run_with_timeout(command: &mut Command, secs: u64) -> Option<(bool, String)> {
    let mut child = command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    let status = loop {
        if let Some(status) = child.try_wait().ok()? {
            break status;
        }
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let mut text = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut text);
    }
    Some((status.success(), text))
}

fn set_cloexec(fd: i32, on: bool) {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFD);
        let flags = if on { flags | libc::FD_CLOEXEC } else { flags & !libc::FD_CLOEXEC };
        libc::fcntl(fd, libc::F_SETFD, flags);
    }
}

/// Checks that `exe` starts, within a few seconds, and reads the state
/// format this process writes; otherwise the upgrade is refused, because
/// once exec'd a binary that cannot take over would end every session.
pub fn preflight(exe: &Path, format: u32) -> io::Result<()> {
    let (ok, text) = run_with_timeout(Command::new(exe).arg("--handoff-format"), 3)
        .ok_or_else(|| io::Error::other(format!("{} --handoff-format did not run or finish", exe.display())))?;
    let want = format!("rsh-handoff {format}");
    if !ok || text.trim() != want {
        return Err(io::Error::other(format!(
            "{} cannot take over (says {:?}, need {want:?})",
            exe.display(),
            text.trim()
        )));
    }
    Ok(())
}

/// Replaces this process with `exe`, keeping `fds` open and passing `state`
/// in an unlinked file under `dir`. Returns only if that failed, with every
/// fd as it was, so the caller keeps serving.
pub fn exec(exe: &Path, format: u32, state: &[u8], fds: &[i32], dir: &Path) -> io::Error {
    match prepare(exe, format, state, dir) {
        Err(err) => err,
        Ok((file, argv, envp)) => {
            for &fd in fds {
                set_cloexec(fd, false);
            }
            mask_usr2(libc::SIG_BLOCK);
            let path = CString::new(exe.as_os_str().as_bytes()).unwrap_or_default();
            let mut argv_ptrs: Vec<*const libc::c_char> = argv.iter().map(|a| a.as_ptr()).collect();
            argv_ptrs.push(std::ptr::null());
            let mut envp_ptrs: Vec<*const libc::c_char> = envp.iter().map(|e| e.as_ptr()).collect();
            envp_ptrs.push(std::ptr::null());
            unsafe {
                libc::execve(path.as_ptr(), argv_ptrs.as_ptr(), envp_ptrs.as_ptr());
            }
            let err = io::Error::last_os_error();
            mask_usr2(libc::SIG_UNBLOCK);
            for &fd in fds {
                set_cloexec(fd, true);
            }
            drop(file);
            err
        }
    }
}

type Prepared = (File, Vec<CString>, Vec<CString>);

fn prepare(exe: &Path, format: u32, state: &[u8], dir: &Path) -> io::Result<Prepared> {
    preflight(exe, format)?;
    use std::os::unix::fs::OpenOptionsExt;
    let path = dir.join(format!(".handoff-{}", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    std::fs::remove_file(&path)?;
    file.write_all(state)?;
    file.seek(SeekFrom::Start(0))?;
    set_cloexec(file.as_raw_fd(), false);
    let argv: Vec<CString> = std::env::args_os()
        .map(|arg| CString::new(arg.as_bytes()).unwrap_or_default())
        .collect();
    let mut envp: Vec<CString> = std::env::vars_os()
        .filter(|(key, _)| key != ENV_FD)
        .filter_map(|(key, value)| {
            let mut item = key;
            item.push("=");
            item.push(value);
            CString::new(item.as_bytes()).ok()
        })
        .collect();
    envp.push(CString::new(format!("{ENV_FD}={}", file.as_raw_fd())).expect("env"));
    Ok((file, argv, envp))
}

/// The state a previous process handed over, if this process is the
/// result of an upgrade. Clears the variable so shells never see it.
pub fn take_handoff() -> Option<io::Result<Vec<u8>>> {
    let fd: i32 = std::env::var(ENV_FD).ok()?.parse().ok()?;
    std::env::remove_var(ENV_FD);
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut state = Vec::new();
    Some(file.read_to_end(&mut state).map(|_| state))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        vec![text.to_string()]
    }

    #[test]
    fn serve_command_lines() {
        assert!(is_rsh_serve(&words("/home/dev/.rsh/bin/rsh serve --bind 0.0.0.0:2222")));
        assert!(is_rsh_serve(&["./rsh".into(), "serve".into()]));
        assert!(!is_rsh_serve(&words("sh -c ssh box rsh serve")), "a shell that mentions it");
        assert!(!is_rsh_serve(&words("/usr/bin/rsh ls")));
        assert_eq!(serve_port(&words("rsh serve --bind 127.0.0.1:47500 --auth x"), 2222), 47500);
        assert_eq!(serve_port(&words("rsh serve"), 2222), 2222);
        let me = process_version(std::process::id() as i32, &[]);
        assert!(me.is_none(), "a test binary is not an rsh: {me:?}");
        assert!(own_pids().contains(&(std::process::id() as i32)));
    }
}
