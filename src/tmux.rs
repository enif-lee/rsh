//! tmux sessions on the server's host. An rsh session joins one by running
//! `command(name)`; the server lists the host's tmux sessions for `rsh ls`
//! and `~l`.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

/// The command of an rsh session joined to a tmux session: tmux through the
/// user's login shell, so a PATH set in its rc files finds it. The name
/// goes in an environment variable, never into the quoted program, and the
/// same line works in sh, bash, zsh and fish.
const PREFIX: &str = "exec env RSH_TMUX=";
const SUFFIX: &str = " \"$SHELL\" -lc 'exec tmux new-session -A -s \"$RSH_TMUX\"'";

pub fn command(name: &str) -> String {
    format!("{PREFIX}{}{SUFFIX}", crate::remote::shell_quote(name))
}

/// Whether an rsh session's command is `command` of some name.
pub fn is_wrapper(command: &str) -> bool {
    command.starts_with(PREFIX) && command.ends_with(SUFFIX)
}

/// A session name tmux can be joined by again and the server keeps as it
/// is: `:` and `.` (target separators, which `new-session -A` parses) and
/// `#` (format expansion) become `_`; no control characters, at most 64
/// characters, trimmed (as `server::sanitize_name`).
pub fn clean_name(name: &str) -> String {
    name.trim()
        .chars()
        .filter(|ch| !ch.is_control())
        .map(|ch| if matches!(ch, ':' | '.' | '#') { '_' } else { ch })
        .take(64)
        .collect::<String>()
        .trim()
        .to_string()
}

/// One tmux session as `tmux list-sessions` reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub name: String,
    /// tmux clients attached to it, rsh's or not.
    pub clients: u32,
    /// Last activity, seconds since the epoch.
    pub activity: u64,
    /// Size of its current window.
    pub cols: u16,
    pub rows: u16,
}

/// The name last: a tab in it stays part of it.
const FORMAT: &str = "#{session_attached}\t#{session_activity}\t#{window_width}\t#{window_height}\t#{session_name}";

/// The host's tmux sessions, none when tmux is missing, runs no server or
/// takes over a second. Blocks; the server runs it on a thread.
pub fn list() -> Vec<Found> {
    let Some(tmux) = locate() else {
        return Vec::new();
    };
    let mut command = Command::new(tmux);
    // The server's own TMUX (started inside tmux) would point at that
    // server; sessions use the default one (`pty::child_env` drops it too).
    command.args(["list-sessions", "-F", FORMAT]).env_remove("TMUX").env_remove("TMUX_PANE");
    parse(&run(command, Duration::from_secs(1), |_| false))
}

fn parse(lines: &[String]) -> Vec<Found> {
    lines
        .iter()
        .filter_map(|line| {
            let mut fields = line.splitn(5, '\t');
            let clients = fields.next()?.parse().ok()?;
            let activity = fields.next()?.parse().unwrap_or(0);
            let cols = fields.next()?.parse().unwrap_or(0);
            let rows = fields.next()?.parse().unwrap_or(0);
            let name = fields.next()?.to_string();
            Some(Found { name, clients, activity, cols, rows })
        })
        .collect()
}

/// tmux as a login shell finds it; looked up again after a minute when it
/// was missing, so installing tmux needs no server restart.
fn locate() -> Option<PathBuf> {
    static FOUND: Mutex<Option<(Instant, Option<PathBuf>)>> = Mutex::new(None);
    let mut found = FOUND.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((at, path)) = &*found {
        if path.is_some() || at.elapsed() < Duration::from_secs(60) {
            return path.clone();
        }
    }
    let shell = crate::pty::login_shell();
    let mut command = Command::new(&shell);
    // Started clean, as `cli::link_on_path` does: only the login variables.
    command.args(["-lc", "command -v tmux"]).env_clear().env("SHELL", &shell).env("TERM", "dumb").env("PATH", "/usr/local/bin:/usr/bin:/bin");
    for name in ["HOME", "USER", "LOGNAME"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    let is_path = |line: &str| line.starts_with('/') && line.ends_with("/tmux");
    let path = run(command, Duration::from_secs(3), is_path).into_iter().rev().find(|line| is_path(line)).map(PathBuf::from);
    *found = Some((Instant::now(), path.clone()));
    path
}

/// The lines a command prints until it closes its output, `done` accepts a
/// line, or `limit` passes. It runs in a session of its own and is killed
/// with all it started, so rc files cannot take a terminal or linger.
fn run(mut command: Command, limit: Duration, done: impl Fn(&str) -> bool) -> Vec<String> {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    unsafe {
        use std::os::unix::process::CommandExt;
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    let Ok(mut child) = command.spawn() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let (send, receive) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if send.send(line).is_err() {
                    return;
                }
            }
        });
        let deadline = Instant::now() + limit;
        while let Ok(line) = receive.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            let stop = done(&line);
            lines.push(line);
            if stop {
                break;
            }
        }
    }
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.wait();
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_command_quotes_and_is_recognised() {
        let line = command("it's a:b.c");
        assert_eq!(line, "exec env RSH_TMUX='it'\\''s a:b.c' \"$SHELL\" -lc 'exec tmux new-session -A -s \"$RSH_TMUX\"'");
        assert!(is_wrapper(&line));
        assert!(!is_wrapper("exec env RSH_TMUX=x sh"));
        assert!(!is_wrapper("vim"));
        assert_eq!(clean_name("  work\x07 \n"), "work");
        assert_eq!(clean_name("a.b:c#{pid}"), "a_b_c_{pid}");
        // The name goes through sh untouched.
        let out = Command::new("/bin/sh").args(["-c", &line.replace("exec tmux new-session -A -s", "printf %s")]).env("SHELL", "/bin/sh").output().unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), "it's a:b.c");
    }

    #[test]
    fn list_lines_parse() {
        let lines = vec!["1\t1792944475\t100\t30\ta b".to_string(), "0\t5\t80\t24\tx\ty".to_string(), "junk".to_string()];
        assert_eq!(
            parse(&lines),
            vec![
                Found { name: "a b".into(), clients: 1, activity: 1792944475, cols: 100, rows: 30 },
                Found { name: "x\ty".into(), clients: 0, activity: 5, cols: 80, rows: 24 },
            ]
        );
    }
}
