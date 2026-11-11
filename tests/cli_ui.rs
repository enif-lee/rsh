//! Exercise the actual CLI in a child PTY; output bytes are also parsed so
//! colors cannot hide a stale name or change the menu's visible contents.
use rsh::crypto::Key;
use rsh::net::{self, AuthStore, HelloReply, Server};
use rsh::proto::Action;
use rsh::pty::{Pty, SpawnOptions};
use rsh::terminal::Terminal;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

struct Menu {
    pty: Pty,
    screen: Terminal,
    bytes: Vec<u8>,
}
impl Menu {
    fn start(args: &[&str], dir: &Path, no_color: bool, term: &str) -> Self {
        let mut words = vec![
            "exec".to_string(),
            "env".into(),
            "-u".into(),
            "NO_COLOR".into(),
            "RSH_NO_UPDATE=1".into(),
            format!("HOME={}", dir.display()),
            format!("TERM={term}"),
        ];
        if no_color {
            words.push("NO_COLOR=1".into());
        }
        words.push(env!("CARGO_BIN_EXE_rsh").into());
        words.extend(args.iter().map(|s| s.to_string()));
        let command = words
            .iter()
            .map(|s| rsh::remote::shell_quote(s))
            .collect::<Vec<_>>()
            .join(" ");
        let pty = Pty::spawn(&SpawnOptions {
            session_id: 0,
            cols: 130,
            rows: 30,
            env: &[],
            command: &[command],
        })
        .unwrap();
        Self {
            pty,
            screen: Terminal::with_size(130, 30),
            bytes: Vec::new(),
        }
    }
    fn text(&self) -> String {
        (0..30)
            .map(|row| self.screen.screen_line(row))
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn wait(&mut self, expected: &str) {
        self.wait_for(expected, |text| text.contains(expected));
    }
    fn wait_gone(&mut self, expected: &str) {
        self.wait_for(expected, |text| !text.contains(expected));
    }
    fn wait_for(&mut self, expected: &str, ready: impl Fn(&str) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(6);
        while !ready(&self.text()) {
            assert!(
                Instant::now() < deadline,
                "missing {expected:?}: {}",
                self.text()
            );
            let mut bytes = [0u8; 65536];
            match self.pty.read(&mut bytes) {
                Ok(n) if n > 0 => {
                    self.screen.feed(&bytes[..n]);
                    self.bytes.extend_from_slice(&bytes[..n]);
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                other => panic!("child stopped ({other:?}): {}", self.text()),
            }
        }
    }
    fn send(&self, text: &[u8]) {
        assert_eq!(self.pty.write(text).unwrap(), text.len());
    }
}
impl Drop for Menu {
    fn drop(&mut self) {
        self.pty.kill();
        unsafe {
            libc::waitpid(self.pty.pid(), std::ptr::null_mut(), 0);
        }
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("rsh-ui-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn menu_rename_survives_filter_and_a_fresh_listing() {
    let server = Server::spawn();
    server.allow("fixture.example", "token");
    let dir = Temp::new("rename");
    let auth = dir.0.join("known");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "token");
    store.save(&auth).unwrap();
    let key = Key::from_token("token");
    let socket = net::bind_for(server.addr()).unwrap();
    let hello = net::new_hello(
        "fixture.example",
        Action::Named("before".into()),
        (80, 24),
        &[],
    );
    let HelloReply::Ready(done) = net::hello(&socket, server.addr(), &key, &hello).unwrap() else {
        panic!("denied")
    };
    let id = done.attached.session;
    let address = server.addr().to_string();
    let args = ["ls", "--auth", auth.to_str().unwrap(), "--peer", &address];
    let mut menu = Menu::start(&args, &dir.0, false, "xterm-256color");
    menu.wait("before");
    assert!(menu.text().contains("[1] rsh before"));
    menu.send(b"before");
    menu.wait("filter: before");
    menu.send(b"m");
    menu.wait("session name");
    menu.send(b"\x15after\r");
    menu.wait("Renamed: before → after");
    let fresh = net::list_sessions_wait(server.addr(), &key, Duration::from_secs(1)).unwrap();
    assert!(fresh.iter().any(|s| s.id == id && s.label == "after"));
    assert!(
        !menu.text().contains("filter: before"),
        "renaming clears the stale filter"
    );
    assert!(
        String::from_utf8_lossy(&menu.bytes).contains("\x1b[36m"),
        "session/host colors"
    );
    menu.send(b"\x1b");
    drop(menu);
    let mut reopened = Menu::start(&args, &dir.0, false, "xterm-256color");
    reopened.wait("after");
    assert!(!reopened.text().contains("before"));
    reopened.send(b"\x1b");
    let mut plain = Menu::start(&args, &dir.0, true, "xterm-256color");
    plain.wait("after");
    let bytes = String::from_utf8_lossy(&plain.bytes);
    assert!(
        !bytes.contains("\x1b[36m") && !bytes.contains("\x1b[32m"),
        "NO_COLOR keeps text plain"
    );
}

#[test]
fn help_colors_follow_terminal_and_no_color() {
    let dir = Temp::new("help");
    for (no_color, term, colored) in [
        (false, "xterm-256color", true),
        (true, "xterm-256color", false),
        (false, "dumb", false),
    ] {
        let mut menu = Menu::start(&["--help"], &dir.0, no_color, term);
        menu.wait("auth file");
        let bytes = String::from_utf8_lossy(&menu.bytes);
        assert_eq!(
            bytes.contains("\x1b["),
            colored,
            "NO_COLOR={no_color}, TERM={term}"
        );
        assert!(bytes.contains("~m") && bytes.contains("--name"));
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rsh"))
        .arg("--help")
        .env_remove("NO_COLOR")
        .env("TERM", "xterm-256color")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        !output.stdout.contains(&0x1b),
        "a pipe contains no color escapes"
    );
}

#[test]
fn tmux_menu_rename_is_visible_in_an_immediate_and_reopened_list() {
    use std::process::{Command, Stdio};
    if Command::new("tmux").arg("-V").output().is_err() {
        return;
    }
    let dir = Temp::new("tmux");
    struct Backend {
        child: std::process::Child,
        dir: PathBuf,
    }
    impl Drop for Backend {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = Command::new("tmux")
                .arg("kill-server")
                .env("TMUX_TMPDIR", &self.dir)
                .env_remove("TMUX")
                .output();
        }
    }
    let tmux = Command::new("tmux")
        .args([
            "-f",
            "/dev/null",
            "new-session",
            "-d",
            "-s",
            "before",
            "sleep 30",
        ])
        .env("TMUX_TMPDIR", &dir.0)
        .env_remove("TMUX")
        .output()
        .unwrap();
    assert!(
        tmux.status.success(),
        "{}",
        String::from_utf8_lossy(&tmux.stderr)
    );
    let auth = dir.0.join("known");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "token");
    store.save(&auth).unwrap();
    let port = std::net::UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let address = format!("127.0.0.1:{port}");
    let child = Command::new(env!("CARGO_BIN_EXE_rsh"))
        .args([
            "serve",
            "--bind",
            &address,
            "--authorized",
            auth.to_str().unwrap(),
        ])
        .env("TMUX_TMPDIR", &dir.0)
        .env_remove("TMUX")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _backend = Backend {
        child,
        dir: dir.0.clone(),
    };
    let peer = address.parse().unwrap();
    let key = Key::from_token("token");
    let deadline = Instant::now() + Duration::from_secs(5);
    while net::list_sessions_wait(peer, &key, Duration::from_millis(100)).is_err() {
        assert!(Instant::now() < deadline);
    }
    // Keep a native tmux client attached throughout. It must not affect
    // rsh's detached/attached labels, colors or sorting.
    let command = format!(
        "exec env TMUX_TMPDIR={} tmux attach-session -t before",
        rsh::remote::shell_quote(dir.0.to_str().unwrap())
    );
    let native_pty = Pty::spawn(&SpawnOptions {
        session_id: 0,
        cols: 130,
        rows: 30,
        env: &["TERM=xterm-256color".into()],
        command: &[command],
    })
    .unwrap();
    let mut native = Menu {
        pty: native_pty,
        screen: Terminal::with_size(130, 30),
        bytes: Vec::new(),
    };
    native.wait("before");
    let tmux_clients = || {
        let out = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_attached}"])
            .env("TMUX_TMPDIR", &dir.0)
            .env_remove("TMUX")
            .output()
            .unwrap();
        String::from_utf8(out.stdout)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap()
    };
    assert_eq!(tmux_clients(), 1);
    let args = ["ls", "--auth", auth.to_str().unwrap(), "--peer", &address];
    let mut menu = Menu::start(&args, &dir.0, false, "xterm-256color");
    menu.wait("[1] tmux before  detached");
    assert!(
        !menu.text().contains("client"),
        "native client counts are not shown"
    );
    menu.send(b"m");
    menu.wait("session name");
    menu.send(b"\x15after\r");
    menu.wait("Renamed: before → after");
    for _ in 0..3 {
        let listing = net::list(peer, &key, Duration::from_secs(2), true).unwrap();
        assert!(listing.tmux.iter().any(|s| s.name == "after"));
        assert!(!listing.tmux.iter().any(|s| s.name == "before"));
    }
    menu.send(b"\x1b");
    drop(menu);
    let mut reopened = Menu::start(&args, &dir.0, false, "xterm-256color");
    reopened.wait("after");
    assert!(!reopened.text().contains("before"));
    reopened.send(b"\x1b");
    drop(reopened);
    let mut active = Menu::start(
        &[
            "--auth",
            auth.to_str().unwrap(),
            "--peer",
            &address,
            "--tmux",
            "fixture.example",
            "after",
        ],
        &dir.0,
        false,
        "xterm-256color",
    );
    active.wait("after:fixture.example");
    active.send(b"~l");
    active.wait("attached 130x");
    assert!(
        tmux_clients() >= 2,
        "native and rsh tmux clients are both actually attached"
    );
    active.send(b"m");
    active.wait("Rename session");
    active.send(b"\x7f\x7f\x7f\x7f\x7flater\r");
    active.wait("tmux   later");
    assert!(
        active.text().contains("rsh sessions"),
        "tmux rename returns to the session list"
    );
    active.send(b"\r");
    active.wait_gone("rsh sessions");
    assert!(active.text().contains("later:fixture.example"));
    active.send(b"~.");
    active.wait("left tmux session later");
    let mut detached = Menu::start(&args, &dir.0, false, "xterm-256color");
    detached.wait("[1] tmux later  detached");
    assert_eq!(
        tmux_clients(),
        1,
        "the native tmux client is still attached"
    );
}

#[test]
fn in_session_rename_returns_to_list_and_can_switch_afterwards() {
    let server = Server::spawn();
    server.allow("fixture.example", "token");
    let dir = Temp::new("list-return");
    let auth = dir.0.join("known");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "token");
    store.save(&auth).unwrap();
    let key = Key::from_token("token");
    let socket = net::bind_for(server.addr()).unwrap();
    let mut other = 0;
    for name in ["before", "other"] {
        let hello = net::new_hello("fixture.example", Action::Named(name.into()), (80, 24), &[]);
        let HelloReply::Ready(done) = net::hello(&socket, server.addr(), &key, &hello).unwrap()
        else {
            panic!("denied")
        };
        if name == "other" {
            other = done.attached.session;
        }
    }
    let address = server.addr().to_string();
    let mut menu = Menu::start(
        &[
            "--auth",
            auth.to_str().unwrap(),
            "--peer",
            &address,
            "fixture.example",
            "before",
        ],
        &dir.0,
        false,
        "xterm-256color",
    );
    menu.wait("before:fixture.example");
    menu.send(b"~l");
    menu.wait("other");
    menu.send(b"m");
    menu.wait("Rename session");
    menu.send(b"\x1b");
    menu.wait_gone("Rename session");
    assert!(
        menu.text().contains("rsh sessions"),
        "cancelling returns to the list"
    );
    menu.send(b"m");
    menu.wait("Rename session");
    menu.send(b"\x7f\x7f\x7f\x7f\x7f\x7frenamed\r");
    menu.wait("rsh    renamed");
    assert!(
        menu.text().contains("rsh sessions"),
        "success keeps the list open"
    );
    menu.send(b"\x1b[B\r");
    menu.wait("other:fixture.example");
    menu.send(b"printf 'CHOICE:%s\\n' \"$RSH_SESSION\"\r");
    menu.wait(&format!("CHOICE:{other}"));
}
