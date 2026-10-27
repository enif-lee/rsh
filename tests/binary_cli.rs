use std::io::Write;
use std::process::Command;

use rsh::net::{AuthStore, Server};

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rsh"));
    cmd.env_remove("RSH_CATALOG");
    cmd
}

fn marker(text: &str) -> u64 {
    let bytes = text.as_bytes();
    let mut index = 0;
    while let Some(found) = text[index..].find("MARK:") {
        let start = index + found + 5;
        if start < bytes.len() && bytes[start].is_ascii_digit() {
            let mut cursor = start;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if let Ok(id) = text[start..cursor].parse::<u64>() {
                return id;
            }
        }
        index += found + 5;
    }
    panic!("no marker in {text}");
}

fn listed_id(text: &str, index: usize) -> u64 {
    text.lines()
        .find(|line| line.starts_with(&format!("[{index}] ")))
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap()
}

fn spawn_with(args: &[&str], auth: &std::path::Path, peer: &str, script: &[u8]) -> String {
    let mut child = bin()
        .args(args)
        .env("RSH_AUTH", auth)
        .env("RSH_PEER", peer)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(script).unwrap();
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stderr={stderr} stdout={stdout}");
    assert!(!stdout.contains("authentication required"), "{stdout}");
    assert!(!stderr.contains("authentication required"), "{stderr}");
    stdout
}

#[test]
fn binary_host_and_ls_repeat_same_session() {
    let peer = Server::spawn();
    peer.allow("fixture.example", "secret-token");
    let dir = std::env::temp_dir().join(format!("rsh-bin-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let auth = dir.join("auth");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "secret-token");
    store.save(&auth).unwrap();
    let peer_addr = peer.addr().to_string();

    let script = b"echo MARK:$RSH_SESSION\n";
    let first = spawn_with(&["fixture.example"], &auth, &peer_addr, script);
    let second = spawn_with(&["fixture.example"], &auth, &peer_addr, script);
    assert!(!first.to_lowercase().contains("password"), "{first}");
    assert_eq!(marker(&first), marker(&second), "{first}\n{second}");

    let pick = b"1\necho MARK:$RSH_SESSION\n";
    let listed = spawn_with(&["ls"], &auth, &peer_addr, pick);
    let only = listed_id(&listed, 1);
    assert_eq!(listed.lines().filter(|line| line.starts_with('[') && !line.contains("] tmux ")).count(), 1, "{listed}");
    assert_eq!(marker(&listed), only, "{listed}");
    let again = spawn_with(&["ls"], &auth, &peer_addr, pick);
    assert_eq!(marker(&again), only, "{again}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn piped_stdin_streams_into_the_session() {
    let peer = Server::spawn();
    peer.allow("fixture.example", "secret-token");
    let dir = std::env::temp_dir().join(format!("rsh-pipe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let auth = dir.join("auth");
    let mut store = AuthStore::new();
    store.insert("fixture.example", "secret-token");
    store.save(&auth).unwrap();
    let mut child = bin()
        .args(["fixture.example", "--name", "piped"])
        .env("RSH_AUTH", &auth)
        .env("RSH_PEER", peer.addr().to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"echo first-$((1+1))\n").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(3));
    stdin.write_all(b"echo second-$((2+2))\n").unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(text.contains("first-2"), "{text}");
    assert!(text.contains("second-4"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn live_upgrade_keeps_sessions_and_attached_clients() {
    use rsh::client::{self, Attached, Input, Outcome};
    use rsh::crypto::Key;
    use rsh::net::{self, HelloReply, HostMux};
    use rsh::proto::Action;
    use std::time::{Duration, Instant};

    let dir = std::env::temp_dir().join(format!("rsh-upgrade-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("rsh");
    std::fs::copy(env!("CARGO_BIN_EXE_rsh"), &exe).unwrap();
    let auth = dir.join("auth");
    let mut store = AuthStore::new();
    store.insert("up.example", "tok");
    store.save(&auth).unwrap();
    let port = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let log = std::fs::File::create(dir.join("serve.log")).unwrap();
    let mut server = Command::new(&exe)
        .args(["serve", "--bind", &format!("127.0.0.1:{port}"), "--authorized"])
        .arg(&auth)
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .spawn()
        .unwrap();
    let peer: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let key = Key::from_token("tok");
    let deadline = Instant::now() + Duration::from_secs(5);
    while net::list_sessions_wait(peer, &key, Duration::from_millis(200)).is_err() {
        assert!(Instant::now() < deadline, "server did not start");
    }
    let pidfile = dir.join(format!("serve-{port}.pid"));
    let before = std::fs::read_to_string(&pidfile).unwrap();

    let attach = |action: Action| {
        let mut mux = HostMux::new();
        let sock = mux.socket_for("up.example", peer).unwrap();
        let hello = net::new_hello("up.example", action, (80, 24), &[]);
        let HelloReply::Ready(ready) = net::hello(&sock, peer, &key, &hello).unwrap() else { panic!("denied") };
        (mux, Attached::new("up.example", peer, &key, *ready))
    };
    let (mut mux, attached) = attach(Action::Named("keep".into()));
    let session = attached.session;
    let script = b"export KEEP=yes; for i in $(seq 1 14); do echo tick-$i; sleep 0.5; done\n".to_vec();
    let client = std::thread::spawn(move || {
        let mut out = Vec::new();
        let outcome = client::run(
            &mut mux,
            &attached,
            Input::Script(script),
            &mut out,
            client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
        )
        .unwrap();
        (outcome, String::from_utf8_lossy(&out).into_owned())
    });
    std::thread::sleep(Duration::from_millis(1500));

    // A command whose shell ends before the upgrade, with its client not yet
    // told: the exit status must survive the upgrade.
    let (mut ended_mux, ended) = {
        let mut mux = HostMux::new();
        let sock = mux.socket_for("up.example", peer).unwrap();
        let hello = net::new_hello("up.example", Action::Default, (80, 24), &["printf bye; exit 5".to_string()]);
        let HelloReply::Ready(ready) = net::hello(&sock, peer, &key, &hello).unwrap() else { panic!("denied") };
        (mux, Attached::new("up.example", peer, &key, *ready))
    };

    // A binary that is not an rsh, or reads another state format, is refused
    // and the old server keeps going.
    use std::os::unix::fs::PermissionsExt;
    for fake in [&b"not a program"[..], &b"#!/bin/sh\necho rsh-handoff 999\n"[..]] {
        std::fs::write(dir.join("fake"), fake).unwrap();
        std::fs::set_permissions(dir.join("fake"), std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::rename(dir.join("fake"), &exe).unwrap();
        unsafe { libc::kill(server.id() as i32, libc::SIGUSR2) };
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(std::fs::read_to_string(&pidfile).unwrap(), before, "refused");
    }

    // The real update: a new file at the same path, then SIGUSR2.
    std::fs::copy(env!("CARGO_BIN_EXE_rsh"), dir.join("next")).unwrap();
    std::fs::rename(dir.join("next"), &exe).unwrap();
    unsafe { libc::kill(server.id() as i32, libc::SIGUSR2) };
    let deadline = Instant::now() + Duration::from_secs(10);
    while std::fs::read_to_string(&pidfile).unwrap_or_default() == before {
        assert!(Instant::now() < deadline, "server did not upgrade");
        std::thread::sleep(Duration::from_millis(50));
    }
    let after = std::fs::read_to_string(&pidfile).unwrap();
    assert_eq!(after.split_whitespace().next(), Some(server.id().to_string().as_str()), "same pid");
    // Requests arriving while it execs again must not kill it.
    for _ in 0..20 {
        unsafe { libc::kill(server.id() as i32, libc::SIGUSR2) };
        std::thread::sleep(Duration::from_millis(5));
    }
    std::thread::sleep(Duration::from_millis(1500));
    assert!(server.try_wait().unwrap().is_none(), "server survived repeated upgrade requests");

    let (outcome, text) = client.join().unwrap();
    assert_eq!(outcome, Outcome::Detached, "{text}");
    assert!(text.contains("tick-14"), "the attached client kept receiving: {}", &text[text.len().saturating_sub(400)..]);

    let (mut mux, again) = attach(Action::Named("keep".into()));
    assert_eq!(again.session, session);
    let mut out = Vec::new();
    client::run(
        &mut mux,
        &again,
        Input::Script(b"echo GOT:$KEEP\n".to_vec()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("GOT:yes"), "same shell after the upgrade: {text}");
    assert!(text.contains("tick-14"), "screen carried over");
    let mut out = Vec::new();
    let outcome = client::run(
        &mut ended_mux,
        &ended,
        Input::Script(Vec::new()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    assert_eq!(outcome, Outcome::Exited(5), "{}", String::from_utf8_lossy(&out));
    assert!(String::from_utf8_lossy(&out).contains("bye"));

    let log = std::fs::read_to_string(dir.join("serve.log")).unwrap();
    assert!(log.contains("upgrade failed, still serving"), "{log}");
    assert!(log.contains("cannot take over"), "{log}");
    assert!(log.contains("kept 2 sessions"), "{log}");

    let _ = server.kill();
    let _ = server.wait();
    let _ = std::fs::remove_dir_all(&dir);
}

/// tmux sessions through `rsh serve`: on a tmux server of the test's own
/// (TMUX_TMPDIR), never the user's. Skipped without tmux.
#[test]
fn tmux_sessions_are_joined_listed_and_left() {
    use rsh::client::{self, Attached, Input, Outcome};
    use rsh::crypto::Key;
    use rsh::net::{self, HelloReply, HostMux};
    use rsh::proto::Action;
    use std::time::{Duration, Instant};

    if Command::new("tmux").arg("-V").output().map_or(true, |out| !out.status.success()) {
        eprintln!("no tmux; skipped");
        return;
    }
    let dir = std::env::temp_dir().join(format!("rsh-tmux-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let tmux = |args: &[&str]| {
        let out = Command::new("tmux").args(args).env("TMUX_TMPDIR", &dir).env_remove("TMUX").output().unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    struct Cleanup(Box<dyn Fn()>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            (self.0)();
        }
    }
    let kill_tmux = {
        let dir = dir.clone();
        move || {
            let _ = Command::new("tmux").arg("kill-server").env("TMUX_TMPDIR", &dir).env_remove("TMUX").output();
        }
    };
    let tmux_server = Cleanup(Box::new(kill_tmux));
    tmux(&["new-session", "-d", "-s", "outside", "-x", "90", "-y", "20"]);

    let auth = dir.join("auth");
    let mut store = AuthStore::new();
    store.insert("t.example", "tok");
    store.save(&auth).unwrap();
    let port = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let mut server = Command::new(env!("CARGO_BIN_EXE_rsh"))
        .args(["serve", "--bind", &format!("127.0.0.1:{port}"), "--authorized"])
        .arg(&auth)
        .env("TMUX_TMPDIR", &dir)
        .env_remove("TMUX")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let server_pid = server.id() as i32;
    let stop_server = Cleanup(Box::new(move || unsafe {
        libc::kill(server_pid, libc::SIGTERM);
    }));
    let peer: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let key = Key::from_token("tok");
    let deadline = Instant::now() + Duration::from_secs(5);
    while net::list_sessions_wait(peer, &key, Duration::from_millis(200)).is_err() {
        assert!(Instant::now() < deadline, "server did not start");
    }
    let listed = net::list(peer, &key, Duration::from_secs(5), true).unwrap();
    let outside = listed.tmux.iter().find(|t| t.name == "outside").expect("the tmux session made outside rsh is listed");
    assert_eq!((outside.cols, outside.rows, outside.session), (90, 20, 0));
    assert!(net::list(peer, &key, Duration::from_secs(5), false).unwrap().tmux.is_empty(), "only when asked");

    let attach = |action: Action, command: Vec<String>| {
        let mut mux = HostMux::new();
        let sock = mux.socket_for("t.example", peer).unwrap();
        let hello = net::new_hello("t.example", action, (80, 24), &command);
        let HelloReply::Ready(ready) = net::hello(&sock, peer, &key, &hello).unwrap() else { panic!("denied") };
        let created = ready.attached.created;
        (mux, Attached::new("t.example", peer, &key, *ready), created)
    };
    let run = |mux: &mut HostMux, attached: &Attached, script: &[u8]| {
        let mut out = Vec::new();
        let outcome = client::run(
            mux,
            attached,
            Input::Script(script.to_vec()),
            &mut out,
            client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: Some("work".into()), clipboard_read: false },
        )
        .unwrap();
        (outcome, String::from_utf8_lossy(&out).into_owned())
    };
    let join = || attach(Action::Named("work".into()), vec![rsh::tmux::command("work")]);

    // Joining makes the tmux session; the rsh session is listed on its row.
    let (mut mux, first, created) = join();
    assert!(created);
    assert_eq!(first.label, "work");
    let (outcome, text) = run(&mut mux, &first, b"echo from-tmux-$((6*7))\n");
    assert_eq!(outcome, Outcome::Detached, "{text}");
    assert!(text.contains("from-tmux-42"), "{text}");
    assert!(tmux(&["list-sessions", "-F", "#{session_name}"]).lines().any(|name| name == "work"));
    std::thread::sleep(Duration::from_millis(2100));
    let listed = net::list(peer, &key, Duration::from_secs(5), true).unwrap();
    let work = listed.tmux.iter().find(|t| t.name == "work").expect("work listed");
    assert_eq!((work.session, work.clients), (first.session, 1));

    // Joining again reaches the same rsh session; a plain session never does.
    let (_, again, created) = join();
    assert_eq!(again.session, first.session);
    assert!(!created);
    let (_, plain, _) = attach(Action::Default, Vec::new());
    assert_ne!(plain.session, first.session);
    let (_, named, _) = attach(Action::Named("work".into()), Vec::new());
    assert_ne!(named.session, first.session, "the plain session called work is another one");

    // Ending the rsh session (what ~. does) detaches tmux; the tmux session stays.
    let killed = net::kill_sessions(peer, &key, "t.example", &first.session.to_string()).unwrap();
    assert!(killed.contains(&first.session));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        std::thread::sleep(Duration::from_millis(300));
        let clients = tmux(&["list-sessions", "-F", "#{session_name} #{session_attached}"]);
        if clients.lines().any(|line| line == "work 0") {
            break;
        }
        assert!(Instant::now() < deadline, "tmux client still attached: {clients}");
    }

    // Detaching inside tmux (C-b d) ends the new rsh session with status 0.
    let (mut mux, second, created) = join();
    assert!(created, "a new rsh session for the tmux session left");
    let (outcome, text) = run(&mut mux, &second, b"\x02d");
    assert_eq!(outcome, Outcome::Exited(0), "{text}");
    assert!(tmux(&["list-sessions", "-F", "#{session_name}"]).lines().any(|name| name == "work"), "tmux session kept");

    // The tmux server goes while its socket is still there to reach it.
    drop(stop_server);
    drop(tmux_server);
    let _ = server.wait();
    let _ = std::fs::remove_dir_all(&dir);
}
