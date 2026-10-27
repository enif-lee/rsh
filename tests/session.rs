use rsh::client::{self, Attached, Input, Outcome};
use rsh::crypto::Key;
use rsh::net::{self, HelloReply, HostMux};
use rsh::proto::{self, Action, ClientStream, KIND_STREAM};
use rsh::server::{Server, OUTPUT_BACKLOG};
use rsh::terminal::Terminal;
use std::net::{SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const HOST: &str = "box.example";
const TOKEN: &str = "tok";

fn server() -> Server {
    let server = Server::spawn();
    server.allow(HOST, TOKEN);
    server
}

fn key() -> Key {
    Key::from_token(TOKEN)
}

fn attach(peer: SocketAddr, mux: &mut HostMux, action: Action, command: &[&str]) -> Attached {
    let sock = mux.socket_for(HOST, peer).unwrap();
    let command: Vec<String> = command.iter().map(|s| s.to_string()).collect();
    let hello = net::new_hello(HOST, action, (80, 24), &command);
    match net::hello(&sock, peer, &key(), &hello).unwrap() {
        HelloReply::Ready(ready) => Attached::new(HOST, peer, &key(), *ready),
        HelloReply::Denied(reason) => panic!("denied: {reason}"),
    }
}

fn run_script(peer: SocketAddr, action: Action, script: &str) -> (Attached, Outcome, Vec<u8>) {
    let mut mux = HostMux::new();
    let attached = attach(peer, &mut mux, action, &[]);
    let mut out = Vec::new();
    let outcome = client::run(
        &mut mux,
        &attached,
        Input::Script(script.as_bytes().to_vec()),
        &mut out,
        client::Options {
            size: (80, 24),
            escape: None,
            raw: None,
            forwarder: None,
            allowed_targets: Vec::new(),
            sessions: None,
            frames: None,
            tmux: None,
            clipboard_read: false,
        },
    )
    .unwrap();
    (attached, outcome, out)
}

fn screen(bytes: &[u8]) -> String {
    let mut term = Terminal::with_size(80, 24);
    term.feed(bytes);
    (0..24).map(|row| term.screen_line(row)).collect::<Vec<_>>().join("\n")
}

/// Forwards datagrams both ways, dropping, duplicating and reordering some.
struct LossyProxy {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
}

impl LossyProxy {
    fn start(server: SocketAddr, loss_percent: u64) -> Self {
        let front = UdpSocket::bind("127.0.0.1:0").unwrap();
        let back = UdpSocket::bind("127.0.0.1:0").unwrap();
        front.set_nonblocking(true).unwrap();
        back.set_nonblocking(true).unwrap();
        let addr = front.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let done = Arc::clone(&stop);
        thread::spawn(move || {
            let mut client: Option<SocketAddr> = None;
            let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
            let mut held: Option<(Vec<u8>, bool)> = None;
            let mut buf = vec![0u8; 65536];
            while !done.load(Ordering::Relaxed) {
                let mut fds = [
                    libc::pollfd { fd: front.as_raw_fd(), events: libc::POLLIN, revents: 0 },
                    libc::pollfd { fd: back.as_raw_fd(), events: libc::POLLIN, revents: 0 },
                ];
                unsafe { libc::poll(fds.as_mut_ptr(), 2, 20) };
                for (index, upstream) in [(0, true), (1, false)] {
                    let sock = if upstream { &front } else { &back };
                    while let Ok((n, from)) = sock.recv_from(&mut buf) {
                        let _ = index;
                        if upstream {
                            client = Some(from);
                        }
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        let roll = seed % 100;
                        let data = buf[..n].to_vec();
                        let send = |bytes: &[u8], up: bool| {
                            if up {
                                let _ = back.send_to(bytes, server);
                            } else if let Some(client) = client {
                                let _ = front.send_to(bytes, client);
                            }
                        };
                        if roll < loss_percent {
                            continue;
                        }
                        if roll < loss_percent + 5 {
                            if let Some((old, up)) = held.replace((data, upstream)) {
                                send(&old, up);
                            }
                            continue;
                        }
                        send(&data, upstream);
                        if roll > 95 {
                            send(&data, upstream);
                        }
                        if let Some((old, up)) = held.take() {
                            send(&old, up);
                        }
                    }
                }
            }
        });
        Self { addr, stop }
    }
}

impl Drop for LossyProxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

#[test]
fn shell_output_and_resume_redraws_screen() {
    let server = server();
    let (first, outcome, out) = run_script(server.addr(), Action::Named("work".into()), "echo MARK:$RSH_SESSION\n");
    assert_eq!(outcome, Outcome::Detached);
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains(&format!("MARK:{}", first.session)), "{text}");
    let (second, _, out) = run_script(server.addr(), Action::Named("work".into()), "");
    assert_eq!(second.session, first.session);
    assert_ne!(second.attach, first.attach);
    let view = screen(&out);
    assert!(view.contains(&format!("MARK:{}", first.session)), "{view}");
}

/// A UDP relay that delays every datagram by `delay` each way, counting
/// the bytes that reach the client.
struct DelayProxy {
    addr: SocketAddr,
    /// Bytes that went to the client.
    down_bytes: Arc<std::sync::atomic::AtomicUsize>,
    stop: Arc<AtomicBool>,
}

impl DelayProxy {
    fn start(server: SocketAddr, delay: Duration) -> Self {
        use std::collections::VecDeque;
        use std::sync::Mutex;
        let front = Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap());
        let back = Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap());
        for sock in [&front, &back] {
            sock.set_read_timeout(Some(Duration::from_millis(20))).unwrap();
        }
        let addr = front.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let down_bytes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        // Datagrams due at a time, going up (to the server) or not.
        type Queue = Mutex<VecDeque<(Instant, bool, Vec<u8>)>>;
        let queue: Arc<Queue> = Arc::default();
        let client: Arc<Mutex<Option<SocketAddr>>> = Arc::default();
        for upstream in [true, false] {
            let (sock, queue, client, stop) = (if upstream { Arc::clone(&front) } else { Arc::clone(&back) }, Arc::clone(&queue), Arc::clone(&client), Arc::clone(&stop));
            thread::spawn(move || {
                let mut buf = vec![0u8; 65536];
                while !stop.load(Ordering::Relaxed) {
                    if let Ok((n, from)) = sock.recv_from(&mut buf) {
                        if upstream {
                            *client.lock().unwrap() = Some(from);
                        }
                        queue.lock().unwrap().push_back((Instant::now() + delay, upstream, buf[..n].to_vec()));
                    }
                }
            });
        }
        let (done, counted) = (Arc::clone(&stop), Arc::clone(&down_bytes));
        thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                let due: Vec<(bool, Vec<u8>)> = {
                    let mut queue = queue.lock().unwrap();
                    let now = Instant::now();
                    let mut due = Vec::new();
                    while queue.front().is_some_and(|(at, _, _)| *at <= now) {
                        let (_, up, bytes) = queue.pop_front().unwrap();
                        due.push((up, bytes));
                    }
                    due
                };
                for (up, bytes) in due {
                    if up {
                        let _ = back.send_to(&bytes, server);
                    } else if let Some(client) = *client.lock().unwrap() {
                        counted.fetch_add(bytes.len(), Ordering::Relaxed);
                        let _ = front.send_to(&bytes, client);
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
        Self { addr, down_bytes, stop }
    }
}

impl Drop for DelayProxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

#[test]
fn output_over_a_slow_round_trip_is_compressed_and_exact() {
    let server = server();
    let proxy = DelayProxy::start(server.addr(), Duration::from_millis(25));
    let (_, outcome, out) = run_script(proxy.addr, Action::Named("far".into()), "sleep 1; yes abcdefgh | head -n 2000; echo DONE-$((6*7))\n");
    assert_eq!(outcome, Outcome::Detached);
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("DONE-42"), "{}", &text[text.len().saturating_sub(300)..]);
    assert_eq!(text.matches("abcdefgh\r\n").count(), 2000, "every line, once");
    // 50 ms round trips: the output (20 KB on the terminal) went deflated,
    // in less than its own size although each datagram went twice (as at
    // the start of an attachment); plain it would be over 40 KB. Read in
    // small pieces under load, it does not compress much better.
    let wire = proxy.down_bytes.load(Ordering::Relaxed);
    assert!(wire < 20_000, "{wire} bytes reached the client");
}

#[test]
fn where_udp_is_blocked_the_session_goes_over_tcp() {
    let server = server();
    server.set_drop_udp(true);
    let started = Instant::now();
    // A listing (with less time than a handshake) goes over TCP too.
    assert!(net::list_sessions(server.addr(), &key()).unwrap().is_empty());
    let (first, outcome, out) = run_script(server.addr(), Action::Named("tcp".into()), "seq 1 30000; echo MARK:$RSH_SESSION\n");
    assert_eq!(outcome, Outcome::Detached);
    assert!(first.wire.borrow().is_none(), "run took the connection");
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains(&format!("MARK:{}", first.session)), "tail: {}", &text[text.len().saturating_sub(300)..]);
    assert!(text.contains("\r\n29999\r\n30000\r\n"), "the output, whole");
    // The second time TCP is tried at once.
    let again = Instant::now();
    let (second, _, out) = run_script(server.addr(), Action::Named("tcp".into()), "");
    assert_eq!(second.session, first.session);
    assert!(screen(&out).contains(&format!("MARK:{}", first.session)));
    assert!(again.elapsed() < Duration::from_secs(2), "{:?} then {:?}", again - started, again.elapsed());
    let listed = net::list_sessions(server.addr(), &key()).unwrap();
    assert!(listed.iter().any(|s| s.id == first.session));
}

#[test]
fn idle_tcp_connections_do_not_shut_out_clients() {
    let server = server();
    server.set_drop_udp(true);
    let idle: Vec<_> = (0..300).map(|_| std::net::TcpStream::connect(server.addr()).unwrap()).collect();
    thread::sleep(Duration::from_millis(300));
    let (_, outcome, out) = run_script(server.addr(), Action::Named("crowd".into()), "echo MARK-$((6*7))\n");
    assert_eq!(outcome, Outcome::Detached);
    assert!(String::from_utf8_lossy(&out).contains("MARK-42"));
    drop(idle);
}

#[test]
fn input_past_the_pty_backlog_arrives_whole_over_tcp() {
    let server = server();
    server.set_drop_udp(true);
    // 600 KB sent while the program sleeps: more than the server queues for
    // the pty, so part is dropped there and must come again.
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Named("paste".into()), &["sh", "-c", "stty raw -echo; sleep 2; head -c 600000 | wc -c"]);
    let mut out = Vec::new();
    let outcome = client::run(
        &mut mux,
        &attached,
        Input::Script("x\n".repeat(300_000).into_bytes()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    assert_eq!(outcome, Outcome::Exited(0));
    assert!(screen(&out).contains("600000"), "{}", screen(&out));
}

/// Relays TCP connections to `server`; `cut` breaks the ones open.
struct TcpProxy {
    addr: SocketAddr,
    open: Arc<std::sync::Mutex<Vec<std::net::TcpStream>>>,
    accepted: Arc<std::sync::atomic::AtomicUsize>,
}

impl TcpProxy {
    fn start(server: SocketAddr) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let open = Arc::new(std::sync::Mutex::new(Vec::new()));
        let list = Arc::clone(&open);
        let accepted = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = Arc::clone(&accepted);
        thread::spawn(move || {
            for front in listener.incoming().flatten() {
                count.fetch_add(1, Ordering::Relaxed);
                let back = std::net::TcpStream::connect(server).unwrap();
                list.lock().unwrap().extend([front.try_clone().unwrap(), back.try_clone().unwrap()]);
                for (mut from, mut to) in [(front.try_clone().unwrap(), back.try_clone().unwrap()), (back, front)] {
                    thread::spawn(move || {
                        let _ = std::io::copy(&mut from, &mut to);
                        let _ = to.shutdown(std::net::Shutdown::Both);
                    });
                }
            }
        });
        Self { addr, open, accepted }
    }

    fn cut(&self) {
        for stream in self.open.lock().unwrap().drain(..) {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

#[test]
fn a_broken_tcp_connection_is_made_again() {
    let server = server();
    // Nothing answers UDP at the proxy's port.
    let proxy = TcpProxy::start(server.addr());
    let cutter = {
        let open = Arc::clone(&proxy.open);
        thread::spawn(move || {
            for _ in 0..2 {
                thread::sleep(Duration::from_millis(4500));
                for stream in open.lock().unwrap().drain(..) {
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                }
            }
        })
    };
    // Output with no pause the script would take for its end, across the cuts.
    let script = "for i in $(seq 1 80); do echo L$i; sleep 0.1; done; echo DONE-$((6*7))\n";
    let (_, outcome, out) = run_script(proxy.addr, Action::Named("cut".into()), script);
    cutter.join().unwrap();
    proxy.cut();
    assert_eq!(outcome, Outcome::Detached);
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("DONE-42"), "tail: {}", &text[text.len().saturating_sub(300)..]);
    assert!(text.contains("\r\nL79\r\nL80\r\n"));
    assert_eq!(proxy.accepted.load(Ordering::Relaxed), 3, "connected again after each cut");
}

#[test]
fn compression_asked_for_works_on_a_fast_link_too() {
    let server = server();
    let proxy = DelayProxy::start(server.addr(), Duration::ZERO);
    let mut mux = HostMux::new();
    let sock = mux.socket_for(HOST, proxy.addr).unwrap();
    let mut hello = net::new_hello(HOST, Action::Named("asked".into()), (80, 24), &[]);
    hello.features |= proto::FEATURE_COMPRESS;
    let attached = match net::hello(&sock, proxy.addr, &key(), &hello).unwrap() {
        HelloReply::Ready(ready) => Attached::new(HOST, proxy.addr, &key(), *ready),
        HelloReply::Denied(reason) => panic!("denied: {reason}"),
    };
    let mut out = Vec::new();
    let script = b"sleep 1; yes abcdefgh | head -n 20000; echo DONE-$((6*7))\n".to_vec();
    let options = client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false };
    let outcome = client::run(&mut mux, &attached, Input::Script(script), &mut out, options).unwrap();
    assert_eq!(outcome, Outcome::Detached);
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("DONE-42"), "{}", &text[text.len().saturating_sub(300)..]);
    assert_eq!(text.matches("abcdefgh\r\n").count(), 20000, "every line, once");
    // 200 KB on the terminal, a quarter of it on the wire at most: one
    // deflate stream from the first byte (about 15 KB, more when the
    // output is read in small pieces under load; plain it is over 200 KB).
    let wire = proxy.down_bytes.load(Ordering::Relaxed);
    assert!(wire < 50_000, "{wire} bytes reached the client");
}

#[test]
fn output_is_exact_through_loss_duplication_and_reordering() {
    let server = server();
    let proxy = LossyProxy::start(server.addr(), 20);
    let (_, outcome, out) = run_script(proxy.addr, Action::Named("lossy".into()), "seq 1 30000; echo DONE-$((6*7))\n");
    assert_eq!(outcome, Outcome::Detached);
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("DONE-42"), "tail: {}", &text[text.len().saturating_sub(300)..]);
    let numbers: Vec<u32> = text
        .split("\r\n")
        .filter_map(|line| line.rsplit('\x07').next().unwrap_or(line).trim().parse::<u32>().ok())
        .collect();
    let start = numbers.iter().position(|n| *n == 1).unwrap();
    let run: Vec<u32> = numbers[start..start + 30000].to_vec();
    assert_eq!(run, (1..=30000).collect::<Vec<_>>());
}

#[test]
fn command_session_reports_exit_code_and_is_removed() {
    let server = server();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Default, &["echo hi-from-cmd; exit 7"]);
    let mut out = Vec::new();
    let outcome = client::run(
        &mut mux,
        &attached,
        Input::Script(Vec::new()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    assert_eq!(outcome, Outcome::Exited(7));
    assert!(String::from_utf8_lossy(&out).contains("hi-from-cmd"));
    let deadline = Instant::now() + Duration::from_secs(3);
    while server.session_ids().contains(&attached.session) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    assert!(!server.session_ids().contains(&attached.session));
}

#[test]
fn second_client_takes_over_and_first_is_kicked() {
    let server = server();
    let mut mux_a = HostMux::new();
    let a = attach(server.addr(), &mut mux_a, Action::Named("shared".into()), &[]);
    let mut mux_b = HostMux::new();
    let b = attach(server.addr(), &mut mux_b, Action::Named("shared".into()), &[]);
    assert_eq!(a.session, b.session);
    let mut out = Vec::new();
    let outcome = client::run(
        &mut mux_a,
        &a,
        Input::Script(b"echo hi\n".to_vec()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    assert!(matches!(outcome, Outcome::Kicked(ref reason) if reason.contains("another client")), "{outcome:?}");
}

#[test]
fn server_adopts_new_client_address() {
    let server = server();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Named("roam".into()), &[]);
    let first = mux.local_addr(HOST).unwrap();
    let send = |sock: &UdpSocket| {
        let stream = ClientStream {
            flags: 0,
            cols: 80,
            rows: 24,
            out_gen: 0,
            out_ack: 0,
            in_offset: 0,
            data: b"",
        };
        let bytes = proto::seal(&attached.key, proto::Dir::ToServer, KIND_STREAM, attached.attach, &stream.encode());
        sock.send_to(&bytes, server.addr()).unwrap();
    };
    let old = mux.socket(HOST).unwrap();
    let new = mux.rebind(HOST).unwrap();
    assert_ne!(new.local_addr().unwrap(), first);
    send(&new);
    thread::sleep(Duration::from_millis(100));
    assert_eq!(server.last_addr(attached.session).map(|a| a.port()), Some(first.port()), "one packet does not move it");
    send(&new);
    let deadline = Instant::now() + Duration::from_secs(2);
    while server.last_addr(attached.session).map(|a| a.port()) != Some(new.local_addr().unwrap().port()) {
        assert!(Instant::now() < deadline, "server kept {:?}", server.last_addr(attached.session));
        thread::sleep(Duration::from_millis(10));
    }
    let replayed_old_counter = {
        let stream = ClientStream { flags: 0, cols: 80, rows: 24, out_gen: 0, out_ack: 0, in_offset: 0, data: b"" };
        proto::seal(&attached.key, proto::Dir::ToServer, KIND_STREAM, attached.attach, &stream.encode())
    };
    send(&new);
    old.send_to(&replayed_old_counter, server.addr()).unwrap();
    thread::sleep(Duration::from_millis(100));
    assert_eq!(server.last_addr(attached.session).unwrap().port(), new.local_addr().unwrap().port());
}

#[test]
fn reattach_after_server_moves_port() {
    let server = server();
    let (first, _, _) = run_script(server.addr(), Action::Named("move".into()), "export KEEP=survives; echo set\n");
    let moved = server.rebind().unwrap();
    let (again, _, out) = run_script(moved, Action::Attach(first.session), "echo GOT:$KEEP\n");
    assert_eq!(again.session, first.session);
    assert!(String::from_utf8_lossy(&out).contains("GOT:survives"));
}

#[test]
fn backlog_is_bounded_for_silent_client_and_dropped_when_detached() {
    let server = server();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Default, &["yes 0123456789012345678901234567890123456789"]);
    let mut peak = 0;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        peak = peak.max(server.backlog(attached.session).unwrap_or(0));
        thread::sleep(Duration::from_millis(20));
    }
    assert!(peak > 0);
    assert!(peak <= OUTPUT_BACKLOG + 256 * 1024, "peak {peak}");
    let killed = net::kill_sessions(server.addr(), &key(), HOST, &attached.session.to_string()).unwrap();
    assert_eq!(killed, vec![attached.session]);
}

#[test]
fn kill_by_name_and_list() {
    let server = server();
    let (a, _, _) = run_script(server.addr(), Action::Named("one".into()), "");
    let (b, _, _) = run_script(server.addr(), Action::Named("two".into()), "");
    let listed = net::list_sessions(server.addr(), &key()).unwrap();
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().all(|s| s.alive && !s.attached));
    assert_eq!(net::kill_sessions(server.addr(), &key(), HOST, "two").unwrap(), vec![b.session]);
    let deadline = Instant::now() + Duration::from_secs(3);
    while server.session_ids().contains(&b.session) {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    assert!(server.session_ids().contains(&a.session));
    assert!(net::kill_sessions(server.addr(), &Key::from_token("wrong"), HOST, "one").is_err());
}

#[test]
fn wrong_token_gets_no_reply_and_replayed_hello_does_nothing() {
    let server = server();
    let wrong = Key::from_token("nope");
    let err = net::list_sessions_wait(server.addr(), &wrong, Duration::from_millis(400)).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    sock.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    let mut hello = net::new_hello(HOST, Action::Named("once".into()), (80, 24), &[]);
    hello.client_pub = rsh::crypto::ephemeral().1;
    let bytes = proto::seal(&key(), proto::Dir::ToServer, proto::KIND_HELLO, key().id(), &hello.encode());
    let mut buf = vec![0u8; 65536];
    let mut readies = Vec::new();
    for _ in 0..2 {
        sock.send_to(&bytes, server.addr()).unwrap();
        let (n, _) = sock.recv_from(&mut buf).unwrap();
        let packet = proto::open(&key(), proto::Dir::ToClient, &buf[..n]).unwrap();
        assert_eq!(packet.kind, proto::KIND_READY);
        readies.push(proto::Ready::decode(&packet.payload).unwrap());
    }
    assert_eq!(readies[0], readies[1], "a retry gets the same answer");
    let elsewhere = UdpSocket::bind("127.0.0.1:0").unwrap();
    elsewhere.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
    elsewhere.send_to(&bytes, server.addr()).unwrap();
    assert!(elsewhere.recv_from(&mut buf).is_err(), "a replay is not reflected to another address");
    thread::sleep(Duration::from_millis(100));
    assert!(server.session_ids().is_empty(), "nothing happens without CONFIRM");
}

#[test]
fn stale_attachment_is_told_after_server_restart() {
    let server = server();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Default, &[]);
    let addr = server.addr();
    drop(server);
    let fresh = Server::bind(addr, &rsh::net::AuthStore::new()).unwrap();
    fresh.allow(HOST, TOKEN);
    let started = Instant::now();
    let mut out = Vec::new();
    let outcome = client::run(
        &mut mux,
        &attached,
        Input::Script(b"x".to_vec()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    assert!(matches!(outcome, Outcome::Kicked(ref reason) if reason.contains("no longer has")), "{outcome:?}");
    assert!(started.elapsed() < Duration::from_secs(15));
}

#[test]
fn disconnect_timeout_hangs_up_abandoned_sessions() {
    let mut store = rsh::net::AuthStore::new();
    store.insert(HOST, TOKEN);
    let server = Server::start("127.0.0.1:0".parse().unwrap(), &store, None, Some(Duration::from_secs(1))).unwrap();
    let (a, _, _) = run_script(server.addr(), Action::Named("idle".into()), "");
    let deadline = Instant::now() + Duration::from_secs(10);
    while server.session_ids().contains(&a.session) {
        assert!(Instant::now() < deadline, "session kept");
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn ctrl_c_drops_a_large_unacked_backlog() {
    let server = server();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Named("flood".into()), &[]);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut offset = 0u64;
    let mut type_bytes = |bytes: &[u8]| {
        let stream = ClientStream { flags: 0, cols: 80, rows: 24, out_gen: 0, out_ack: 0, in_offset: offset, data: bytes };
        sock.send_to(&proto::seal(&attached.key, proto::Dir::ToServer, KIND_STREAM, attached.attach, &stream.encode()), server.addr()).unwrap();
        offset += bytes.len() as u64;
    };
    type_bytes(b"yes 0123456789012345678901234567890123456789\r");
    let deadline = Instant::now() + Duration::from_secs(5);
    while server.backlog(attached.session).unwrap() < 256 * 1024 {
        assert!(Instant::now() < deadline, "no flood");
        thread::sleep(Duration::from_millis(20));
    }
    type_bytes(b"\x03");
    thread::sleep(Duration::from_millis(500));
    let left = server.backlog(attached.session).unwrap();
    assert!(left < 64 * 1024, "backlog after Ctrl-C: {left}");
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn echo_server() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut conn) = conn else { continue };
            thread::spawn(move || {
                use std::io::{Read, Write};
                let mut buf = [0u8; 4096];
                while let Ok(n) = conn.read(&mut buf) {
                    if n == 0 || conn.write_all(&buf[..n]).is_err() {
                        break;
                    }
                }
            });
        }
    });
    addr
}

fn round_trip(port: u16, text: &[u8]) -> Vec<u8> {
    use std::io::{Read, Write};
    let mut conn = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    conn.write_all(text).unwrap();
    let mut back = vec![0u8; text.len()];
    conn.read_exact(&mut back).unwrap();
    back
}

#[test]
fn local_and_remote_port_forwarding() {
    use rsh::forward::{Forwarder, Spec};
    let server = server();
    let echo = echo_server();
    let local_port = free_port();
    let remote_port = free_port();
    let remote = Spec { bind: "127.0.0.1".into(), port: remote_port, target: echo.to_string() };
    let mut mux = HostMux::new();
    let sock = mux.socket(HOST).unwrap();
    let hello = net::new_hello_forwarding(HOST, Action::Named("fwd".into()), (80, 24), &[], &[remote.encode()]);
    let ready = match net::hello(&sock, server.addr(), &key(), &hello).unwrap() {
        HelloReply::Ready(ready) => *ready,
        HelloReply::Denied(reason) => panic!("{reason}"),
    };
    assert!(ready.attached.notes.is_empty(), "{:?}", ready.attached.notes);
    let attached = Attached::new(HOST, server.addr(), &key(), ready);
    let mut forwarder = Forwarder::new(true);
    forwarder.listen(&Spec { bind: "127.0.0.1".into(), port: local_port, target: echo.to_string() }).unwrap();
    let runner = thread::spawn(move || {
        let mut out = Vec::new();
        client::run(
            &mut mux,
            &attached,
            Input::Script(b"for i in 1 2 3 4 5 6 7 8; do echo tick; sleep 0.5; done\n".to_vec()),
            &mut out,
            client::Options { size: (80, 24), escape: None, raw: None, forwarder: Some(forwarder), allowed_targets: vec![echo.to_string()], sessions: None, frames: None, tmux: None, clipboard_read: false },
        )
        .unwrap()
    });
    thread::sleep(Duration::from_millis(300));
    let big: Vec<u8> = (0..200_000u32).map(|i| (i % 253) as u8).collect();
    assert_eq!(round_trip(local_port, b"via-L"), b"via-L");
    assert_eq!(round_trip(local_port, &big), big);
    assert_eq!(round_trip(remote_port, b"via-R"), b"via-R");
    runner.join().unwrap();
}

#[test]
fn ipv6_peer_works() {
    let mut store = rsh::net::AuthStore::new();
    store.insert(HOST, TOKEN);
    let Ok(server) = Server::bind("[::1]:0".parse().unwrap(), &store) else {
        return;
    };
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Named("v6".into()), &[]);
    let _ = attached;
    let sock = mux.socket_for(HOST, server.addr()).unwrap();
    assert!(sock.local_addr().unwrap().is_ipv6());
    let listed = net::list_sessions(server.addr(), &key()).unwrap();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].id >= 100_000 && listed[0].id < 1_000_000);
}

#[test]
fn one_udp_socket_per_host() {
    let server = server();
    server.allow("other.example", "tok-b");
    let mut mux = HostMux::new();
    let first = attach(server.addr(), &mut mux, Action::Named("a".into()), &[]);
    let addr = mux.local_addr(HOST);
    let second = attach(server.addr(), &mut mux, Action::Named("b".into()), &[]);
    assert_ne!(first.session, second.session);
    assert_eq!(mux.local_addr(HOST), addr, "both sessions share the host socket");
    assert_eq!(mux.socket_count(), 1);
    let other = mux.socket_for("other.example", server.addr()).unwrap();
    assert_ne!(Some(other.local_addr().unwrap()), addr);
    assert_eq!(mux.socket_count(), 2);
    assert_eq!(server.last_addr(first.session), server.last_addr(second.session));
}

#[test]
fn reflected_server_packets_are_not_input() {
    let server = server();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Named("reflect".into()), &[]);
    let sock = mux.socket_for(HOST, server.addr()).unwrap();
    sock.set_nonblocking(false).unwrap();
    sock.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut buf = vec![0u8; 65536];
    let (n, _) = sock.recv_from(&mut buf).unwrap();
    assert_eq!(proto::peek(&buf[..n]).map(|(kind, _)| kind), Some(KIND_STREAM));
    let before = server.backlog(attached.session).unwrap();
    for _ in 0..3 {
        sock.send_to(&buf[..n], server.addr()).unwrap();
    }
    thread::sleep(Duration::from_millis(300));
    let screen = server.session_screen(attached.session).unwrap().join("\n");
    assert!(!screen.contains("2026h"), "{screen}");
    assert!(server.backlog(attached.session).unwrap() >= before, "no ack or input came from the echo");
}

#[test]
fn end_offset_follows_a_repaint_after_exit() {
    use rsh::proto::{ServerStream, FLAG_EXITED, FLAG_REDRAW};
    let server = server();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Default, &["printf done; exit 7"]);
    let sock = mux.socket_for(HOST, server.addr()).unwrap();
    sock.set_nonblocking(false).unwrap();
    sock.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
    let mut buf = vec![0u8; 65536];
    let mut exited_gen = None;
    let deadline = Instant::now() + Duration::from_secs(5);
    while exited_gen.is_none() && Instant::now() < deadline {
        if let Ok((n, _)) = sock.recv_from(&mut buf) {
            if let Some(p) = proto::open(&attached.key, proto::Dir::ToClient, &buf[..n]) {
                if let Some(s) = ServerStream::decode(&p.payload) {
                    if s.flags & FLAG_EXITED != 0 {
                        exited_gen = Some(s.out_gen);
                    }
                }
            }
        }
    }
    let gen = exited_gen.expect("exit seen");
    let redraw = ClientStream { flags: FLAG_REDRAW, cols: 80, rows: 24, out_gen: gen, out_ack: 0, in_offset: 0, data: b"" };
    sock.send_to(&proto::seal(&attached.key, proto::Dir::ToServer, KIND_STREAM, attached.attach, &redraw.encode()), server.addr()).unwrap();
    let mut seen_end = 0u64;
    let mut advertised = None;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Ok((n, _)) = sock.recv_from(&mut buf) {
            if let Some(p) = proto::open(&attached.key, proto::Dir::ToClient, &buf[..n]) {
                if let Some(s) = ServerStream::decode(&p.payload) {
                    if s.out_gen == gen + 1 {
                        seen_end = seen_end.max(s.out_offset + s.data.len() as u64);
                        advertised = Some(s.end_offset);
                    }
                }
            }
        }
        if advertised.is_some() && advertised == Some(seen_end) {
            break;
        }
    }
    assert_eq!(advertised, Some(seen_end), "end offset of the repaint generation");
}

#[test]
fn attach_in_the_middle_of_an_escape_sequence() {
    let server = server();
    let mut mux = HostMux::new();
    let first = attach(server.addr(), &mut mux, Action::Default, &["printf 'ok \\033[3'; sleep 1; printf '1mRED\\033[0m\\n'"]);
    thread::sleep(Duration::from_millis(500));
    let mut mux2 = HostMux::new();
    let second = attach(server.addr(), &mut mux2, Action::Attach(first.session), &[]);
    let mut out = Vec::new();
    let outcome = client::run(
        &mut mux2,
        &second,
        Input::Script(Vec::new()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    assert_eq!(outcome, Outcome::Exited(0));
    let mut term = Terminal::with_size(80, 24);
    term.feed(&out);
    let row = (0..24).map(|r| term.screen_line(r)).find(|l| l.contains("RED")).unwrap_or_default();
    assert_eq!(row, "ok RED", "the split sequence must not show as text");
    let col = row.find("RED").unwrap();
    assert_eq!(term.cell(0, col).map(|c| c.attr.fg), Some(rsh::terminal::Color::Named(1)));
}

#[test]
fn revoked_token_ends_its_attachment_and_bad_permissions_fail_closed() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("rsh-revoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let auth = dir.join("authorized");
    let mut store = rsh::net::AuthStore::new();
    store.insert(HOST, TOKEN);
    store.save(&auth).unwrap();
    let server = Server::start("127.0.0.1:0".parse().unwrap(), &store, Some(auth.clone()), None).unwrap();
    let mut mux = HostMux::new();
    let attached = attach(server.addr(), &mut mux, Action::Default, &[]);
    rsh::net::AuthStore::new().save(&auth).unwrap();
    let mut out = Vec::new();
    let outcome = client::run(
        &mut mux,
        &attached,
        Input::Script(b"while :; do echo tick; sleep 0.2; done\n".to_vec()),
        &mut out,
        client::Options { size: (80, 24), escape: None, raw: None, forwarder: None, allowed_targets: Vec::new(), sessions: None, frames: None, tmux: None, clipboard_read: false },
    )
    .unwrap();
    assert!(matches!(outcome, Outcome::Kicked(ref reason) if reason.contains("revoked")), "{outcome:?}");

    // A file others could write grants nothing until it is fixed, and a
    // chmod alone is noticed.
    store.save(&auth).unwrap();
    std::fs::set_permissions(&auth, std::fs::Permissions::from_mode(0o644)).unwrap();
    thread::sleep(Duration::from_millis(2600));
    assert!(net::list_sessions_wait(server.addr(), &key(), Duration::from_millis(500)).is_err());
    std::fs::set_permissions(&auth, std::fs::Permissions::from_mode(0o600)).unwrap();
    thread::sleep(Duration::from_millis(2600));
    assert!(net::list_sessions_wait(server.addr(), &key(), Duration::from_millis(1500)).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}
