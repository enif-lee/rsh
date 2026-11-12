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
        (0..usize::from(self.screen.size().1))
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
        // A PTY read may split a synchronized update between its erase and
        // redraw. Assert only after the complete frame reaches the terminal.
        while self.screen.synchronized() || !self.screen.at_ground() || !ready(&self.text()) {
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
    fn attr(&self, row_text: &str, token: &str) -> rsh::terminal::Attr {
        let row = (0..usize::from(self.screen.size().1))
            .find(|&row| self.screen.screen_line(row).contains(row_text))
            .unwrap_or_else(|| panic!("missing row {row_text:?}: {}", self.text()));
        let line = self.screen.screen_line(row);
        let offset = line.find(token).unwrap_or_else(|| panic!("missing {token:?} in {line:?}"));
        let col = unicode_width::UnicodeWidthStr::width(&line[..offset]);
        self.screen.cell(row, col).unwrap().attr
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
    let mut forced = Menu::start(&args, &dir.0, true, "xterm-256color");
    forced.wait("after");
    let bytes = String::from_utf8_lossy(&forced.bytes);
    assert!(
        bytes.contains("\x1b[36m"),
        "NO_COLOR cannot disable menu colors"
    );
}

#[test]
fn help_always_uses_colors() {
    let dir = Temp::new("help");
    for (no_color, term, colored) in [
        (false, "xterm-256color", true),
        (true, "xterm-256color", true),
        (false, "dumb", true),
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
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        output.stdout.contains(&0x1b),
        "redirected help keeps colors too"
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
    assert_eq!(active.attr("tmux   after", "tmux").fg, rsh::terminal::Color::Indexed(5));
    assert_eq!(active.attr("tmux   after", "after").fg, rsh::terminal::Color::Indexed(6));
    assert_eq!(active.attr("tmux   after", "attached").fg, rsh::terminal::Color::Indexed(2));
    assert_ne!(active.attr("tmux   after", "idle").flags & rsh::terminal::DIM, 0);
    assert!(
        tmux_clients() >= 2,
        "native and rsh tmux clients are both actually attached"
    );
    active.send(b"\x1b[A\x1b[Bm");
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
            let detach = rsh::proto::ClientStream {
                flags: rsh::proto::FLAG_DETACH, cols: 80, rows: 24,
                out_gen: 0, out_ack: 0, in_offset: 0, feedback: &[], data: &[],
            };
            let bytes = rsh::proto::seal(&done.key, rsh::crypto::Dir::ToServer, rsh::proto::KIND_STREAM, done.attach, &detach.encode());
            socket.send_to(&bytes, server.addr()).unwrap();
        }
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while !net::list_sessions_wait(server.addr(), &key, Duration::from_secs(1)).unwrap()
        .iter().any(|s| s.id == other && !s.attached) {
        assert!(Instant::now() < deadline, "fixture session did not detach");
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
    use rsh::terminal::{Color, DIM, REVERSE};
    assert_eq!(menu.attr("│ fixture.example", "fixture.example").fg, Color::Indexed(6));
    assert_eq!(menu.attr("rsh    before", "rsh").fg, Color::Indexed(5));
    let selected_name = menu.attr("rsh    before", "before");
    assert_eq!(selected_name.fg, Color::Indexed(6));
    assert_eq!(selected_name.bg, Color::Indexed(236));
    assert_eq!(selected_name.flags & REVERSE, 0, "selection preserves foreground colors");
    assert_eq!(menu.attr("rsh    before", "attached").fg, Color::Indexed(2));
    assert_eq!(menu.attr("rsh    before", "idle").fg, Color::Indexed(15));
    assert_eq!(menu.attr("rsh    other", "detached").fg, Color::Indexed(3));
    assert_ne!(menu.attr("rsh    other", "idle").flags & DIM, 0);
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
    assert_eq!(menu.attr("rsh    renamed", "renamed").fg, Color::Indexed(6));
    assert_eq!(menu.attr("rsh    renamed", "renamed").bg, Color::Indexed(236));
    assert!(
        menu.text().contains("rsh sessions"),
        "success keeps the list open"
    );
    // The list has a nonselectable host heading. Up from its first
    // session wraps to the last, Down wraps back, and Up+Enter switches.
    menu.send(b"\x1b[A\x1b[B\x1b[A\r");
    menu.wait("other:fixture.example");
    menu.send(b"printf 'CHOICE:%s\\n' \"$RSH_SESSION\"\r");
    menu.wait(&format!("CHOICE:{other}"));
}

#[test]
fn list_arrows_wrap_with_filtered_single_and_empty_results() {
    let server = Server::spawn();
    server.allow("fixture.example", "token");
    let dir = Temp::new("list-wrap");
    let auth = dir.0.join("known");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "token");
    store.save(&auth).unwrap();
    let key = Key::from_token("token");
    let socket = net::bind_for(server.addr()).unwrap();
    for name in ["wrap-a", "wrap-b"] {
        let hello = net::new_hello("fixture.example", Action::Named(name.into()), (80, 24), &[]);
        assert!(matches!(
            net::hello(&socket, server.addr(), &key, &hello).unwrap(),
            HelloReply::Ready(_)
        ));
    }
    let address = server.addr().to_string();
    let mut menu = Menu::start(
        &["ls", "--auth", auth.to_str().unwrap(), "--peer", &address],
        &dir.0,
        false,
        "xterm-256color",
    );
    menu.wait("> [1] rsh wrap-a");
    // The host row is selectable too: it is the first unfiltered choice.
    menu.send(b"\x1b[A");
    menu.wait("> fixture.example");
    menu.send(b"\x1b[A");
    menu.wait("> [2] rsh wrap-b");
    menu.send(b"\x1b[B");
    menu.wait("> fixture.example");
    menu.send(b"wrap-");
    menu.wait("filter: wrap-");
    menu.wait("> [1] rsh wrap-a");
    menu.send(b"\x1b[A");
    menu.wait("> [2] rsh wrap-b");
    menu.send(b"\x1b[B");
    menu.wait("> [1] rsh wrap-a");
    menu.send(b"a");
    menu.wait("filter: wrap-a");
    // A single choice wraps to itself; then an empty result must remain
    // responsive and avoid modulo-by-zero.
    menu.send(b"\x1b[A\x1b[Bz");
    menu.wait("filter: wrap-az");
    menu.send(b"\x1b[A\x1b[B\x7f");
    menu.wait_gone("filter: wrap-az");
    menu.wait("> [1] rsh wrap-a");
    menu.send(b"\r");
    menu.wait("wrap-a:fixture.example");
}

#[test]
fn ordinary_session_scrolls_its_body_with_a_fixed_header() {
    let server = Server::spawn();
    server.allow("fixture.example", "token");
    let dir = Temp::new("local-scrollback");
    let auth = dir.0.join("known");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "token");
    store.save(&auth).unwrap();
    let address = server.addr().to_string();
    let command = "n=0; while [ $n -lt 100 ]; do printf 'tree-node-%03d\\n' $n; n=$((n+1)); done; printf 'TREE_DONE\\nTERM:%s:%s\\n' \"$TERM\" \"$TERM_PROGRAM\"; read reply; printf 'INPUT:%s\\nSIZE:' \"$reply\"; stty size; read finish";
    let mut menu = Menu::start(
        &["--auth", auth.to_str().unwrap(), "--peer", &address, "fixture.example", "scroll-header", "--", command],
        &dir.0, false, "xterm-256color",
    );
    menu.wait("TREE_DONE");
    menu.wait("TERM:xterm-256color:rsh");
    assert!(menu.screen.alt_screen());
    assert!(menu.screen.screen_line(0).contains("scroll-header:fixture.example"));
    let latest = menu.screen.screen_line(1);
    let history = menu.screen.scrollback_len();
    menu.send(b"\x1b[<64;10;");
    menu.send(b"5M");
    menu.wait("[scroll +3]");
    assert_ne!(menu.screen.screen_line(1), latest);
    assert!(menu.screen.screen_line(0).contains("scroll-header:fixture.example"));
    menu.send(b"\x1b[5;2~");
    menu.wait("[scroll +31]");
    assert!(menu.screen.screen_line(0).contains("scroll-header:fixture.example"));
    // Resizing while viewing history must neither scroll the header away
    // nor transfer it into the outer terminal's scrollback.
    menu.screen.resize(100, 20);
    menu.pty.resize(100, 20);
    menu.send(b"back\r");
    menu.wait("INPUT:back");
    menu.wait_for("resized body", |text| text.lines().any(|line| line.strip_prefix("SIZE:").is_some_and(|size| size.split_whitespace().collect::<Vec<_>>() == ["19", "100"])));
    assert!(menu.screen.screen_line(0).contains("scroll-header:fixture.example"));
    menu.wait_gone("[scroll +");
    assert_eq!(menu.screen.scrollback_len(), history);
    menu.send(b"~.");
    menu.wait("detached");
    assert!(!menu.screen.alt_screen());
    assert!((0..menu.screen.scrollback_len()).all(|n| !menu.screen.scrollback_line(n).contains("scroll-header:fixture.example")));
}

#[test]
fn header_stats_toggle_live_and_persist_as_global_defaults() {
    let server = Server::spawn();
    server.allow("fixture.example", "token");
    let dir = Temp::new("header-stats");
    let auth = dir.0.join("known");
    let config = dir.0.join(".rsh/config.yaml");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "token");
    store.save(&auth).unwrap();
    let address = server.addr().to_string();
    let args = ["--auth", auth.to_str().unwrap(), "--peer", &address, "fixture.example", "metrics"];
    let mut menu = Menu::start(&args, &dir.0, true, "xterm-256color");
    menu.wait("metrics:fixture.example");
    assert!(!menu.text().contains("KB/s]"));
    menu.send(b"~,\x1b[B\x1b[B\x1b[B\x1b[C");
    menu.wait("KB/s): true");
    assert!(!config.exists(), "session changes stay local");
    menu.send(b"\x1b");
    menu.wait_gone("rsh settings");
    menu.wait("KB/s] metrics:fixture.example");
    menu.wait_for("measured RTT", |text| text.contains("KB/s]") && !text.contains("--ms"));
    // Drive real remote output, so the header must measure nonzero FPS and traffic.
    let render = format!("{} test render --seconds 2.5 --fps 60\r", rsh::remote::shell_quote(env!("CARGO_BIN_EXE_rsh")));
    menu.send(render.as_bytes());
    menu.wait_for("active output statistics", |text| {
        let top = text.lines().next().unwrap_or("");
        let Some((_, stats)) = top.rsplit_once('[') else { return false; };
        let Some((fps, rest)) = stats.split_once("fps, ") else { return false; };
        let Some((_, rate)) = rest.split_once("ms, ") else { return false; };
        let rate = rate.split("KB/s]").next().unwrap_or("");
        fps.parse::<u64>().is_ok_and(|n| n > 0) && rate.parse::<u64>().is_ok_and(|n| n > 0)
    });
    menu.wait("written into the session:");
    // With no remote output, the header's own periodic redraws do not count.
    menu.wait("[0fps,");
    // Save the global default, leaving this session's setting independent.
    menu.send(b"~,\x1b[C\x1b[B\x1b[B\x1b[B\x1b[C");
    menu.wait("Saved for new attachments");
    assert!(rsh::config::Config::load(&config).unwrap().show_stats);
    menu.send(b"\x1b");
    menu.wait_gone("rsh settings");
    // Rename through settings is still reachable after inserting the new row.
    menu.send(b"~,\x1b[B\x1b[B\x1b[B\x1b[B\r");
    menu.wait("Rename session");
    menu.send(b"\x15"); // Rename input ignores control characters.
    menu.send(b"\x7f\x7f\x7f\x7f\x7f\x7f\x7frenamed\r");
    menu.wait_gone("Rename session");
    menu.wait("KB/s] renamed:fixture.example");
    // Disable only for this attachment and ensure the whole old badge clears.
    menu.send(b"~,\x1b[B\x1b[B\x1b[B\x1b[D");
    menu.wait("KB/s): false");
    menu.send(b"\x1b");
    menu.wait_gone("rsh settings");
    menu.wait_gone("KB/s]");
    assert!(menu.text().contains("renamed:fixture.example"));
    assert!(rsh::config::Config::load(&config).unwrap().show_stats);
    menu.send(b"~.");
    menu.wait("detached");
    drop(menu);
    let args = ["--auth", auth.to_str().unwrap(), "--peer", &address, "fixture.example", "renamed"];
    let mut next = Menu::start(&args, &dir.0, true, "xterm-256color");
    next.wait("KB/s] renamed:fixture.example");
    next.send(b"~.");
    next.wait("detached");
}

#[test]
fn unfinished_synchronized_frame_is_released_before_the_heartbeat() {
    let server = Server::spawn();
    server.allow("fixture.example", "token");
    let dir = Temp::new("sync-timeout");
    let auth = dir.0.join("known");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "token");
    store.save(&auth).unwrap();
    let address = server.addr().to_string();
    // A TUI can leave a frame open when interrupted or resized mid-draw.
    // The bottom row has arrived; it must not wait for more remote output.
    let command = r#"trap 'printf "\033[?2026h\033[2J\033[999;1HBOTTOM_READY"; sleep 2; printf "\033[?2026l"' WINCH; printf '\033[?1049hREADY'; while :; do sleep 0.01; done"#;
    let mut menu = Menu::start(
        &["--auth", auth.to_str().unwrap(), "--peer", &address, "fixture.example", "sync-timeout", "--", command],
        &dir.0, false, "xterm-256color",
    );
    menu.wait("READY");
    // The user's terminal grows before its SIGWINCH reaches rsh.
    menu.screen.resize(110, 36);
    menu.pty.resize(110, 36);
    let deadline = Instant::now() + Duration::from_secs(6);
    let mut received = None;
    loop {
        assert!(Instant::now() < deadline, "bottom row did not arrive");
        if let Some(at) = received {
            assert!(Instant::now().duration_since(at) < Duration::from_millis(500), "received bottom row still held after 500 ms");
        }
        let mut bytes = [0; 65536];
        match menu.pty.read(&mut bytes) {
            Ok(n) if n > 0 => {
                menu.screen.feed(&bytes[..n]);
                menu.bytes.extend_from_slice(&bytes[..n]);
                if menu.screen.screen_line(35).contains("BOTTOM_READY") {
                    received.get_or_insert_with(Instant::now);
                    if !menu.screen.synchronized() && menu.screen.at_ground() {
                        eprintln!("bottom received-to-visible after resize: {:?}", received.unwrap().elapsed());
                        break;
                    }
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(5)),
            other => panic!("child stopped: {other:?}"),
        }
    }
    menu.send(b"~.");
    menu.wait("detached");
}
