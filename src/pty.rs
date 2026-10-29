use std::ffi::{CStr, CString};
use std::io::{self};

#[repr(C)]
struct Winsize {
    row: u16,
    col: u16,
    xpixel: u16,
    ypixel: u16,
}

use libc::{TIOCGWINSZ, TIOCSCTTY, TIOCSWINSZ};

/// Environment names a client may set in the remote shell.
pub const FORWARDED_ENV: &[&str] = &["TERM", "COLORTERM", "LANG", "LC_ALL", "LC_CTYPE", "LC_MESSAGES", "TERM_PROGRAM"];

pub struct SpawnOptions<'a> {
    pub session_id: u64,
    pub cols: u16,
    pub rows: u16,
    /// `NAME=value` pairs from the client, limited to [`FORWARDED_ENV`].
    pub env: &'a [String],
    /// Run `$SHELL -c` with these words joined by spaces instead of a login shell.
    pub command: &'a [String],
}

/// A child process on a pseudo terminal. The master fd is non-blocking and
/// read by the server's poll loop; nothing is buffered here.
pub struct Pty {
    fd: i32,
    pid: i32,
}

impl Pty {
    pub fn spawn(opts: &SpawnOptions) -> io::Result<Self> {
        unsafe { spawn_pty(opts) }
    }

    pub fn fd(&self) -> i32 {
        self.fd
    }

    /// Takes over a master fd and child handed over by a previous process.
    pub fn adopt(fd: i32, pid: i32) -> Self {
        unsafe {
            libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC);
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
        Self { fd, pid }
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// Ok(0) means the slave side is gone (EIO on Linux, 0 on macOS).
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let n = unsafe { libc::read(self.fd, buf.as_mut_ptr().cast(), buf.len()) };
        if n >= 0 {
            return Ok(n as usize);
        }
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EIO) {
            return Ok(0);
        }
        Err(err)
    }

    pub fn write(&self, bytes: &[u8]) -> io::Result<usize> {
        let n = unsafe { libc::write(self.fd, bytes.as_ptr().cast(), bytes.len()) };
        if n >= 0 {
            Ok(n as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        let size = Winsize {
            row: rows.max(1),
            col: cols.max(1),
            xpixel: 0,
            ypixel: 0,
        };
        unsafe {
            libc::ioctl(self.fd, TIOCSWINSZ as _, &size);
        }
    }

    /// Sends SIGHUP to the session's process group, as a closing terminal would.
    pub fn hangup(&self) {
        unsafe {
            libc::kill(-self.pid, libc::SIGHUP);
            libc::kill(self.pid, libc::SIGHUP);
        }
    }

    pub fn kill(&self) {
        kill_group(self.pid);
    }
}

pub fn kill_group(pid: i32) {
    unsafe {
        libc::kill(-pid, libc::SIGKILL);
        libc::kill(pid, libc::SIGKILL);
    }
}

/// Exit status of a finished child without blocking: the exit code, or
/// 128 + signal number like a shell reports it.
pub fn try_wait(pid: i32) -> Option<i32> {
    let mut status = 0;
    let waited = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if waited == pid {
        if libc::WIFEXITED(status) {
            return Some(libc::WEXITSTATUS(status));
        }
        if libc::WIFSIGNALED(status) {
            return Some(128 + libc::WTERMSIG(status));
        }
        return Some(-1);
    }
    if waited < 0 {
        return Some(-1);
    }
    None
}

impl Drop for Pty {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

unsafe fn spawn_pty(opts: &SpawnOptions) -> io::Result<Pty> {
    let master = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
    if master < 0 || libc::grantpt(master) != 0 || libc::unlockpt(master) != 0 {
        if master >= 0 {
            libc::close(master);
        }
        return Err(io::Error::other("open pty"));
    }
    let name_ptr = libc::ptsname(master);
    if name_ptr.is_null() {
        libc::close(master);
        return Err(io::Error::other("ptsname"));
    }
    let slave_name = CString::new(CStr::from_ptr(name_ptr).to_bytes()).map_err(|_| {
        libc::close(master);
        io::Error::other("slave name")
    })?;
    let account = user_account();
    let shell_c = CString::new(account.shell.clone()).unwrap_or_else(|_| CString::new("/bin/sh").expect("sh"));
    let home_c = CString::new(account.home.clone()).unwrap_or_else(|_| CString::new("/").expect("root"));
    let shell_name = std::path::Path::new(&account.shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sh")
        .to_string();
    let args: Vec<CString> = if opts.command.is_empty() {
        vec![CString::new(format!("-{shell_name}")).expect("login argv")]
    } else {
        let joined = opts.command.join(" ").replace('\0', "");
        vec![
            CString::new(shell_name).expect("argv0"),
            CString::new("-c").expect("-c"),
            CString::new(joined).expect("command"),
        ]
    };
    let mut argv: Vec<*const libc::c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
    argv.push(std::ptr::null());
    let env_owned = child_env(&account, opts);
    let mut envp: Vec<*const libc::c_char> = env_owned.iter().map(|item| item.as_ptr()).collect();
    envp.push(std::ptr::null());
    let flags = libc::fcntl(master, libc::F_GETFL);
    libc::fcntl(master, libc::F_SETFL, flags | libc::O_NONBLOCK);
    libc::fcntl(master, libc::F_SETFD, libc::FD_CLOEXEC);
    let (cols, rows) = (opts.cols.max(1), opts.rows.max(1));
    let pid = libc::fork();
    if pid < 0 {
        libc::close(master);
        return Err(io::Error::other("fork"));
    }
    if pid == 0 {
        libc::setsid();
        let slave = libc::open(slave_name.as_ptr(), libc::O_RDWR);
        if slave >= 0 {
            prepare_slave(slave, cols, rows);
            libc::ioctl(slave, TIOCSCTTY as _, 0);
            libc::dup2(slave, 0);
            libc::dup2(slave, 1);
            libc::dup2(slave, 2);
            if slave > 2 {
                libc::close(slave);
            }
        }
        libc::close(master);
        for signal in [libc::SIGPIPE, libc::SIGHUP, libc::SIGINT, libc::SIGQUIT, libc::SIGTERM, libc::SIGCHLD, libc::SIGWINCH, libc::SIGTSTP, libc::SIGTTIN, libc::SIGTTOU] {
            libc::signal(signal, libc::SIG_DFL);
        }
        let mut empty: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut empty);
        libc::sigprocmask(libc::SIG_SETMASK, &empty, std::ptr::null_mut());
        libc::chdir(home_c.as_ptr());
        libc::execve(shell_c.as_ptr(), argv.as_ptr(), envp.as_ptr());
        libc::_exit(127);
    }
    Ok(Pty { fd: master, pid })
}

fn child_env(account: &Account, opts: &SpawnOptions) -> Vec<CString> {
    let mut vars: Vec<(String, String)> = std::env::vars()
        .filter(|(key, _)| {
            !matches!(key.as_str(), "HOME" | "USER" | "LOGNAME" | "SHELL" | "PWD" | "OLDPWD" | "RSH_SESSION" | "TMUX" | "TMUX_PANE" | "STY")
                && !FORWARDED_ENV.contains(&key.as_str())
        })
        .collect();
    if !vars.iter().any(|(key, _)| key == "PATH") {
        vars.push(("PATH".into(), "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".into()));
    }
    for (key, value) in [
        ("HOME", account.home.as_str()),
        ("PWD", account.home.as_str()),
        ("USER", account.name.as_str()),
        ("LOGNAME", account.name.as_str()),
        ("SHELL", account.shell.as_str()),
    ] {
        vars.push((key.into(), value.into()));
    }
    let mut term = None;
    for item in opts.env {
        if let Some((key, value)) = item.split_once('=') {
            if FORWARDED_ENV.contains(&key) && !value.contains('\0') {
                if key == "TERM" {
                    term = Some(value.to_string());
                } else {
                    vars.push((key.into(), value.into()));
                }
            }
        }
    }
    let term = term.filter(|t| !t.is_empty() && t != "dumb").unwrap_or_else(|| "xterm-256color".into());
    vars.push(("TERM".into(), term));
    vars.push(("RSH_SESSION".into(), opts.session_id.to_string()));
    vars.into_iter()
        .filter_map(|(key, value)| CString::new(format!("{key}={value}")).ok())
        .collect()
}

struct Account {
    name: String,
    home: String,
    shell: String,
}

unsafe fn prepare_slave(slave: i32, cols: u16, rows: u16) {
    let mut tio: libc::termios = std::mem::zeroed();
    if libc::tcgetattr(slave, &mut tio) == 0 {
        tio.c_iflag |= libc::ICRNL | libc::IXON | libc::IUTF8;
        tio.c_oflag |= libc::OPOST | libc::ONLCR;
        tio.c_lflag |= libc::ISIG | libc::ICANON | libc::ECHO | libc::ECHOE | libc::ECHOK | libc::ECHOCTL | libc::ECHOKE | libc::IEXTEN;
        tio.c_cc[libc::VMIN] = 1;
        tio.c_cc[libc::VTIME] = 0;
        libc::tcsetattr(slave, libc::TCSANOW, &tio);
    }
    let size = Winsize {
        row: rows,
        col: cols,
        xpixel: 0,
        ypixel: 0,
    };
    libc::ioctl(slave, TIOCSWINSZ as _, &size);
}

/// The user's login shell, as sessions run it.
pub fn login_shell() -> String {
    user_account().shell
}

fn user_account() -> Account {
    let mut name = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let mut home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let mut shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    unsafe {
        let pw = libc::getpwuid(libc::geteuid());
        if !pw.is_null() {
            if let Some(value) = c_string((*pw).pw_name) {
                name = value;
            }
            if let Some(value) = c_string((*pw).pw_dir) {
                home = value;
            }
            if let Some(value) = c_string((*pw).pw_shell) {
                if !value.is_empty() {
                    shell = value;
                }
            }
        }
    }
    if shell.is_empty() || !std::path::Path::new(&shell).exists() {
        shell = "/bin/sh".to_string();
    }
    if home.is_empty() {
        home = "/".to_string();
    }
    Account { name, home, shell }
}

fn c_string(ptr: *const libc::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(str::to_string) }
}

pub fn is_tty(fd: i32) -> bool {
    unsafe { libc::isatty(fd) == 1 }
}

pub fn window_size(fd: i32) -> (u16, u16) {
    let mut size = Winsize {
        row: 24,
        col: 80,
        xpixel: 0,
        ypixel: 0,
    };
    unsafe {
        if libc::ioctl(fd, TIOCGWINSZ as _, &mut size) == 0 && size.col > 0 && size.row > 0 {
            return (size.col, size.row);
        }
    }
    (80, 24)
}

pub struct RawTerm {
    saved: libc::termios,
    fd: i32,
}

impl RawTerm {
    pub fn enter(fd: i32) -> io::Result<Self> {
        unsafe {
            let mut saved: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut saved) != 0 {
                return Err(io::Error::last_os_error());
            }
            let raw = Self { saved, fd };
            raw.reenter()?;
            Ok(raw)
        }
    }

    /// Puts the tty back into raw mode, e.g. after the process was resumed.
    pub fn reenter(&self) -> io::Result<()> {
        unsafe {
            let mut raw = self.saved;
            libc::cfmakeraw(&mut raw);
            raw.c_iflag |= libc::IUTF8;
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;
            if libc::tcsetattr(self.fd, libc::TCSADRAIN, &raw) != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub fn restore(&self) {
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSADRAIN, &self.saved);
        }
    }
}

impl Drop for RawTerm {
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn collect(pty: &Pty, until: impl Fn(&str) -> bool) -> String {
        let start = Instant::now();
        let mut all = Vec::new();
        let mut buf = [0u8; 4096];
        while start.elapsed() < Duration::from_secs(5) {
            match pty.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => all.extend_from_slice(&buf[..n]),
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
            if until(&String::from_utf8_lossy(&all)) {
                break;
            }
        }
        String::from_utf8_lossy(&all).into_owned()
    }

    #[test]
    fn pty_runs_login_shell_in_home() {
        let env = vec!["TERM=screen".to_string(), "LANG=C.UTF-8".to_string(), "EVIL=1".to_string()];
        let pty = Pty::spawn(&SpawnOptions {
            session_id: 7,
            cols: 100,
            rows: 40,
            env: &env,
            command: &[],
        })
        .expect("spawn");
        let home = user_account().home;
        std::thread::sleep(Duration::from_millis(200));
        pty.write(b"echo MARK:$RSH_SESSION SIZE:$(stty size) PWD:$PWD T:$TERM E:${EVIL:-none} L:$LANG\n").unwrap();
        let text = collect(&pty, |t| t.contains("L:C.UTF-8"));
        assert!(text.contains("MARK:7"), "{text}");
        assert!(text.contains("SIZE:40 100"), "{text}");
        assert!(text.contains(&format!("PWD:{home}")), "{text}");
        assert!(text.contains("T:screen"), "{text}");
        assert!(text.contains("E:none"), "{text}");
        pty.hangup();
        let start = Instant::now();
        let mut code = None;
        let mut buf = [0u8; 4096];
        while code.is_none() && start.elapsed() < Duration::from_secs(3) {
            while matches!(pty.read(&mut buf), Ok(n) if n > 0) {}
            code = try_wait(pty.pid());
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(code.is_some());
    }

    #[test]
    fn command_exit_code_and_sigpipe_default() {
        let command = vec!["yes | head -n 2; exit 3".to_string()];
        let pty = Pty::spawn(&SpawnOptions {
            session_id: 1,
            cols: 80,
            rows: 24,
            env: &[],
            command: &command,
        })
        .unwrap();
        let text = collect(&pty, |_| false);
        assert_eq!(text.matches('y').count(), 2, "{text}");
        let start = Instant::now();
        let mut code = None;
        while code.is_none() && start.elapsed() < Duration::from_secs(3) {
            code = try_wait(pty.pid());
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(code, Some(3));
    }
}
