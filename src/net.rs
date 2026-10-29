use crate::crypto::{Dir, Key};
use crate::proto::{
    self, Attached, Hello, Packet, Ready, SessionInfo, KIND_ATTACHED, KIND_CONFIRM, KIND_DENIED, KIND_HELLO,
    KIND_LIST, KIND_LIST_REPLY, KIND_READY,
};
use crate::server::now_ms;
use crate::wire::{self, Wire};
use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::server::Server;

/// `host token [port]` lines. The client keeps one per host it can reach;
/// the server accepts any token in its file.
#[derive(Clone, Debug, Default)]
pub struct AuthStore {
    tokens: BTreeMap<String, (String, Option<u16>)>,
}

impl AuthStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, host: impl Into<String>, token: impl Into<String>) {
        self.tokens.insert(host.into(), (token.into(), None));
    }

    pub fn insert_with_port(&mut self, host: impl Into<String>, token: impl Into<String>, port: Option<u16>) {
        self.tokens.insert(host.into(), (token.into(), port));
    }

    pub fn token(&self, host: &str) -> Option<&str> {
        self.tokens.get(host).map(|(token, _)| token.as_str())
    }

    pub fn port(&self, host: &str) -> Option<u16> {
        self.tokens.get(host).and_then(|(_, port)| *port)
    }

    pub fn hosts(&self) -> Vec<String> {
        self.tokens.keys().cloned().collect()
    }

    pub fn entries(&self) -> Vec<(String, String)> {
        self.tokens
            .iter()
            .map(|(host, (token, _))| (host.clone(), token.clone()))
            .collect()
    }

    /// Reads a token file, refusing one that others may read or write: its
    /// tokens are shell access, as an ssh key is.
    pub fn load(path: &Path) -> io::Result<Self> {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)?.permissions().mode();
        if mode & 0o077 != 0 {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                format!("{} is accessible by others (mode {:o}); run chmod 600 on it", path.display(), mode & 0o777),
            ));
        }
        Self::load_any(path)
    }

    /// Reads a token file whatever its permissions (only to migrate it).
    pub fn load_any(path: &Path) -> io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let mut store = Self::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let host = parts.next().unwrap_or("");
            let token = parts.next().unwrap_or("");
            let port = parts.next().and_then(|p| p.parse().ok());
            if !host.is_empty() && !token.is_empty() {
                store.insert_with_port(host, token, port);
            }
        }
        Ok(store)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut text = String::new();
        for (host, (token, port)) in &self.tokens {
            text.push_str(host);
            text.push(' ');
            text.push_str(token);
            if let Some(port) = port {
                text.push_str(&format!(" {port}"));
            }
            text.push('\n');
        }
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        if let Some(parent) = path.parent() {
            private_dir(parent)?;
        }
        let tmp = path.with_extension("tmp");
        let _ = std::fs::remove_file(&tmp);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&tmp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)
    }
}

/// Creates `dir` as 0700, or narrows it to 0700 if this user owns it: it
/// holds tokens, logs and the server's pidfile.
pub fn private_dir(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    let meta = std::fs::metadata(dir)?;
    if meta.uid() == unsafe { libc::getuid() } && meta.permissions().mode() & 0o077 != 0 {
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// One UDP socket per host, shared by every session to that host in a process.
#[derive(Debug, Default)]
pub struct HostMux {
    sockets: BTreeMap<String, Arc<UdpSocket>>,
}

impl HostMux {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn socket(&mut self, host: &str) -> io::Result<Arc<UdpSocket>> {
        self.socket_for(host, SocketAddr::from(([127, 0, 0, 1], 0)))
    }

    /// The host's socket, bound for the address family of `peer`.
    pub fn socket_for(&mut self, host: &str, peer: SocketAddr) -> io::Result<Arc<UdpSocket>> {
        if let Some(sock) = self.sockets.get(host) {
            return Ok(Arc::clone(sock));
        }
        self.rebind_for(host, peer)
    }

    /// Replaces the host's socket with one on a fresh local port.
    pub fn rebind(&mut self, host: &str) -> io::Result<Arc<UdpSocket>> {
        self.rebind_for(host, SocketAddr::from(([127, 0, 0, 1], 0)))
    }

    pub fn rebind_for(&mut self, host: &str, peer: SocketAddr) -> io::Result<Arc<UdpSocket>> {
        let sock = Arc::new(bind_for(peer)?);
        self.sockets.insert(host.to_string(), Arc::clone(&sock));
        Ok(sock)
    }

    pub fn local_addr(&self, host: &str) -> Option<SocketAddr> {
        self.sockets.get(host).and_then(|s| s.local_addr().ok())
    }

    pub fn socket_count(&self) -> usize {
        self.sockets.len()
    }
}

/// An unbound-port UDP socket that can reach `peer`.
pub fn bind_for(peer: SocketAddr) -> io::Result<UdpSocket> {
    let local = if peer.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let sock = UdpSocket::bind(local)?;
    crate::server::set_buffers(&sock);
    Ok(sock)
}

/// Sends datagrams to one address, as few system calls as the platform
/// allows (`sendmmsg` on Linux). A full send buffer is waited out for a
/// moment; past that, like `send_to`, the rest is dropped and the
/// stream's retransmission covers it.
pub fn send_all(sock: &UdpSocket, peer: SocketAddr, datagrams: &[Vec<u8>]) {
    let mut waited = Duration::ZERO;
    #[cfg(target_os = "linux")]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: zeroed sockaddr storage is valid; the address written
        // below fits it and the declared length matches what was written.
        let mut addr: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
        let len = match peer {
            SocketAddr::V4(v4) => {
                let sin = libc::sockaddr_in {
                    sin_family: libc::AF_INET as libc::sa_family_t,
                    sin_port: v4.port().to_be(),
                    sin_addr: libc::in_addr { s_addr: u32::from_ne_bytes(v4.ip().octets()) },
                    sin_zero: [0; 8],
                };
                unsafe { std::ptr::write(std::ptr::addr_of_mut!(addr).cast(), sin) };
                std::mem::size_of::<libc::sockaddr_in>()
            }
            SocketAddr::V6(v6) => {
                let sin6 = libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as libc::sa_family_t,
                    sin6_port: v6.port().to_be(),
                    sin6_flowinfo: v6.flowinfo(),
                    sin6_addr: libc::in6_addr { s6_addr: v6.ip().octets() },
                    sin6_scope_id: v6.scope_id(),
                };
                unsafe { std::ptr::write(std::ptr::addr_of_mut!(addr).cast(), sin6) };
                std::mem::size_of::<libc::sockaddr_in6>()
            }
        };
        let mut iovs: Vec<libc::iovec> = datagrams
            .iter()
            .map(|d| libc::iovec { iov_base: d.as_ptr() as *mut libc::c_void, iov_len: d.len() })
            .collect();
        let mut msgs: Vec<libc::mmsghdr> = iovs
            .iter_mut()
            .map(|iov| {
                // SAFETY: an all-zero mmsghdr is valid; the pointers set
                // here outlive the sendmmsg calls below.
                let mut msg: libc::mmsghdr = unsafe { std::mem::zeroed() };
                msg.msg_hdr.msg_name = std::ptr::addr_of_mut!(addr).cast();
                msg.msg_hdr.msg_namelen = len as libc::socklen_t;
                msg.msg_hdr.msg_iov = iov;
                msg.msg_hdr.msg_iovlen = 1;
                msg
            })
            .collect();
        let mut done = 0;
        while done < msgs.len() {
            let rest = &mut msgs[done..];
            let n = unsafe { libc::sendmmsg(sock.as_raw_fd(), rest.as_mut_ptr(), rest.len() as _, 0) };
            if n > 0 {
                done += n as usize;
                continue;
            }
            match io::Error::last_os_error().kind() {
                ErrorKind::Interrupted => {}
                ErrorKind::WouldBlock if wait_writable(sock, &mut waited) => {}
                _ => break,
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    for datagram in datagrams {
        while let Err(err) = sock.send_to(datagram, peer) {
            if err.kind() != ErrorKind::Interrupted && !(err.kind() == ErrorKind::WouldBlock && wait_writable(sock, &mut waited)) {
                return;
            }
        }
    }
}

/// Waits until the socket can take more, within 10 ms in all per batch.
fn wait_writable(sock: &UdpSocket, waited: &mut Duration) -> bool {
    use std::os::fd::AsRawFd;
    const LIMIT: Duration = Duration::from_millis(10);
    if *waited >= LIMIT {
        return false;
    }
    let start = Instant::now();
    let mut fd = libc::pollfd { fd: sock.as_raw_fd(), events: libc::POLLOUT, revents: 0 };
    let ms = (LIMIT - *waited).as_millis().max(1) as i32;
    let ready = unsafe { libc::poll(&mut fd, 1, ms) } > 0;
    *waited += start.elapsed();
    ready
}

/// How long a request waits for UDP before it tries TCP: half its time,
/// at most this.
const UDP_TRY: Duration = Duration::from_secs(3);

/// Over TCP for a server that answered only that way before (while it
/// still takes TCP connections), else UDP.
fn wire_for(sock: Arc<UdpSocket>, peer: SocketAddr) -> io::Result<Wire> {
    if wire::prefers_tcp(peer) {
        if let Ok(tcp) = Wire::tcp(peer) {
            return Ok(tcp);
        }
    }
    Wire::udp(sock, peer)
}

/// Sends one sealed request and waits for a reply of an accepted kind,
/// resending the same bytes over UDP so the server can recognise the
/// retry. The request is sealed with `key` and carries `id` in its header;
/// replies are opened with the same key. Returns the reply and the
/// request's nonce.
#[allow(clippy::too_many_arguments)]
fn request(
    wire: &mut Wire,
    key: &Key,
    kind: u8,
    id: u64,
    payload: &[u8],
    accept: &[u8],
    wait: Duration,
) -> io::Result<(Packet, [u8; 24])> {
    let bytes = proto::seal(key, Dir::ToServer, kind, id, payload);
    let nonce: [u8; 24] = bytes[13..proto::HEADER_LEN].try_into().expect("nonce");
    let peer = wire.peer();
    let over = if wire.is_tcp() { " over TCP" } else { "" };
    let started = Instant::now();
    let deadline = started + wait;
    let mut buf = vec![0u8; 65536];
    let mut interval = Duration::from_millis(200);
    // A spinner once a reply takes over a second; gone when this returns.
    let mut waiting: Option<Spinner> = None;
    let mut sent = false;
    while Instant::now() < deadline {
        if waiting.is_none() && started.elapsed() >= Duration::from_secs(1) && crate::pty::is_tty(2) {
            waiting = Spinner::new(format!("waiting for a reply from {peer}{over}"));
        }
        // TCP delivers the first.
        if !sent || !wire.is_tcp() {
            wire.send(&bytes)?;
            sent = true;
        }
        let resend = Instant::now() + interval;
        interval = (interval * 2).min(Duration::from_secs(1));
        loop {
            let left = resend.min(deadline).saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            wire.service(Instant::now());
            if wire.down() {
                return Err(io::Error::new(ErrorKind::ConnectionReset, format!("{peer} closed the TCP connection")));
            }
            let Some((n, _)) = wire.recv(&mut buf) else {
                wire::wait_readable(wire, left);
                continue;
            };
            if let Some(packet) = proto::open(key, Dir::ToClient, &buf[..n]) {
                if accept.contains(&packet.kind) {
                    return Ok((packet, nonce));
                }
            }
        }
    }
    Err(io::Error::new(
        ErrorKind::TimedOut,
        format!("no reply from {peer} (server not running, port blocked, or wrong token)"),
    ))
}

/// `request` over UDP, and when no answer comes for a while, over TCP to
/// the same port (a network that lets only TCP through).
#[allow(clippy::too_many_arguments)]
fn request_or_tcp(
    wire: &mut Wire,
    key: &Key,
    kind: u8,
    id: u64,
    payload: &[u8],
    accept: &[u8],
    wait: Duration,
) -> io::Result<(Packet, [u8; 24])> {
    let started = Instant::now();
    // A short wait is a poll (a server starting up): no TCP for it, or one
    // lost or early datagram would put the process on TCP for good.
    let first = if wire.is_tcp() || wait < Duration::from_secs(1) { wait } else { (wait / 2).min(UDP_TRY) };
    // No answer, or UDP may not even be sent (a firewall's EPERM).
    match request(wire, key, kind, id, payload, accept, first) {
        Err(_) if !wire.is_tcp() && started.elapsed() < wait => {}
        done => return done,
    }
    let peer = wire.peer();
    let Ok(tcp) = Wire::tcp(peer) else {
        // No TCP there: UDP for the rest of the time.
        return request(wire, key, kind, id, payload, accept, wait.saturating_sub(started.elapsed()));
    };
    let udp = std::mem::replace(wire, tcp);
    match request(wire, key, kind, id, payload, accept, wait.saturating_sub(started.elapsed())) {
        Ok(done) => {
            wire::remember_tcp(peer);
            Ok(done)
        }
        Err(err) => {
            *wire = udp;
            Err(err)
        }
    }
}

/// An indicatif spinner on stderr, cleared when dropped. None while a
/// session owns the terminal or another spinner is up.
struct Spinner(indicatif::ProgressBar);

impl Spinner {
    fn new(message: String) -> Option<Self> {
        use std::sync::atomic::Ordering;
        if QUIET.load(Ordering::Relaxed) || SPINNING.swap(true, Ordering::Relaxed) {
            return None;
        }
        let bar = indicatif::ProgressBar::new_spinner().with_message(message);
        if let Ok(style) = indicatif::ProgressStyle::with_template("{spinner} rsh: {msg} {elapsed}") {
            bar.set_style(style);
        }
        bar.enable_steady_tick(Duration::from_millis(100));
        Some(Spinner(bar))
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.0.finish_and_clear();
        SPINNING.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

/// An attachment set up by the handshake: its id, its own keys, and what
/// the server did.
pub struct Handshake {
    pub attach: u64,
    pub key: Key,
    pub attached: Attached,
    /// The server's `rsh --version` line.
    pub server: String,
    /// The TCP connection the handshake went over, when UDP did not get
    /// through; the session goes on over it.
    pub wire: Option<Wire>,
}

pub enum HelloReply {
    Ready(Box<Handshake>),
    Denied(String),
}

fn bad(what: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, what.to_string())
}

/// HELLO/READY with the token key, then CONFIRM/ATTACHED with the keys of
/// an X25519 exchange: only after CONFIRM does the server act.
pub fn hello(sock: &UdpSocket, peer: SocketAddr, key: &Key, hello: &Hello) -> io::Result<HelloReply> {
    let mut wire = wire_for(Arc::new(sock.try_clone()?), peer)?;
    let (secret, client_pub) = crate::crypto::ephemeral();
    let mut hello = hello.clone();
    hello.client_pub = client_pub;
    let payload = hello.encode();
    if payload.len() + proto::HEADER_LEN + crate::crypto::TAG_LEN > proto::MAX_REQUEST {
        return Err(io::Error::new(ErrorKind::InvalidInput, "command or environment too long"));
    }
    let (packet, nonce) = request_or_tcp(&mut wire, key, KIND_HELLO, key.id(), &payload, &[KIND_READY, KIND_DENIED], Duration::from_secs(8))?;
    if packet.kind == KIND_DENIED {
        let (echo, reason) = proto::decode_denied(&packet.payload).ok_or_else(|| bad("bad denial"))?;
        if echo != nonce {
            return Err(bad("denial for another request"));
        }
        return Ok(HelloReply::Denied(reason));
    }
    let ready = Ready::decode(&packet.payload).ok_or_else(|| bad("bad ready"))?;
    if ready.echo != nonce {
        return Err(bad("ready for another request"));
    }
    let shared = crate::crypto::shared(&secret, &ready.server_pub).ok_or_else(|| bad("bad key exchange"))?;
    let attach_key = key.attach(&shared, &client_pub, &ready.server_pub);
    let (packet, _) = request(&mut wire, &attach_key, KIND_CONFIRM, ready.attach, &[], &[KIND_ATTACHED], Duration::from_secs(8))?;
    let attached = Attached::decode(&packet.payload).ok_or_else(|| bad("bad attach"))?;
    if !attached.denied.is_empty() {
        return Ok(HelloReply::Denied(attached.denied));
    }
    Ok(HelloReply::Ready(Box::new(Handshake {
        attach: ready.attach,
        key: attach_key,
        attached,
        server: ready.server,
        wire: wire.is_tcp().then_some(wire),
    })))
}

pub fn list_sessions(peer: SocketAddr, key: &Key) -> io::Result<Vec<SessionInfo>> {
    list_sessions_wait(peer, key, Duration::from_secs(3))
}

/// A session owns the terminal: no spinner may draw on it.
static QUIET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// A spinner is on screen: one at a time.
static SPINNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// No "waiting for a reply" spinner from any thread while `on`.
pub fn quiet(on: bool) {
    QUIET.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn list_sessions_wait(peer: SocketAddr, key: &Key, wait: Duration) -> io::Result<Vec<SessionInfo>> {
    list_with_server(peer, key, wait).map(|(sessions, _)| sessions)
}

/// The sessions and the server's `rsh --version` line.
pub fn list_with_server(peer: SocketAddr, key: &Key, wait: Duration) -> io::Result<(Vec<SessionInfo>, String)> {
    list(peer, key, wait, false).map(|listing| (listing.sessions, listing.server))
}

/// The server's answer to LIST; with `tmux`, the host's tmux sessions too.
pub fn list(peer: SocketAddr, key: &Key, wait: Duration, tmux: bool) -> io::Result<proto::Listing> {
    let mut wire = wire_for(Arc::new(bind_for(peer)?), peer)?;
    let mut payload = proto::Writer::new();
    payload.u64(now_ms());
    if tmux {
        payload.u8(proto::LIST_TMUX);
    }
    let payload = payload.finish();
    let (packet, nonce) = request_or_tcp(&mut wire, key, KIND_LIST, key.id(), &payload, &[KIND_LIST_REPLY, KIND_DENIED], wait)?;
    if packet.kind == KIND_DENIED {
        let (echo, reason) = proto::decode_denied(&packet.payload).ok_or_else(|| bad("bad denial"))?;
        if echo != nonce {
            return Err(bad("denial for another request"));
        }
        return Err(io::Error::new(ErrorKind::PermissionDenied, reason));
    }
    let (echo, listing) = proto::decode_list(&packet.payload).ok_or_else(|| bad("bad list"))?;
    if echo != nonce {
        return Err(bad("list for another request"));
    }
    Ok(listing)
}

/// Hangs up sessions of `host` whose id or name is `target`; returns their
/// ids. Goes through the full handshake, so a recorded request cannot be
/// replayed to end sessions.
pub fn kill_sessions(peer: SocketAddr, key: &Key, host: &str, target: &str) -> io::Result<Vec<u64>> {
    let sock = bind_for(peer)?;
    let request = new_hello(host, proto::Action::Kill(target.to_string()), (80, 24), &[]);
    match hello(&sock, peer, key, &request)? {
        HelloReply::Ready(done) => Ok(done.attached.killed),
        HelloReply::Denied(reason) => Err(io::Error::new(ErrorKind::PermissionDenied, reason)),
    }
}

pub fn new_hello(host: &str, action: proto::Action, size: (u16, u16), command: &[String]) -> Hello {
    new_hello_forwarding(host, action, size, command, &[])
}

pub fn new_hello_forwarding(
    host: &str,
    action: proto::Action,
    size: (u16, u16),
    command: &[String],
    forwards: &[String],
) -> Hello {
    let env = crate::pty::FORWARDED_ENV
        .iter()
        .filter_map(|name| std::env::var(name).ok().map(|value| format!("{name}={value}")))
        .collect();
    Hello {
        time_ms: now_ms(),
        host: host.to_string(),
        action,
        cols: size.0,
        rows: size.1,
        env,
        command: command.to_vec(),
        forwards: forwards.to_vec(),
        client_pub: [0; 32],
        features: proto::FEATURE_DEFLATE | proto::FEATURE_ZSTREAM,
    }
}
