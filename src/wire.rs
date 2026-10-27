//! How packets travel between a client and a server: UDP datagrams, or,
//! where UDP does not get through (TCP-only networks, some VPNs), a TCP
//! connection to the same port carrying the same sealed packets, each
//! after a 2-byte length. Everything above (keys, stream, sessions) is the
//! same either way; the server sees each TCP connection as an address of
//! its own (`fd72:7368:7463:7000::/64`).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::os::fd::{AsRawFd, RawFd};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// Bytes kept for a TCP connection the kernel takes no more from. Past
/// this the connection is given up: over TCP the stream resends nothing
/// (`Sender::set_lossless`), so a dropped packet would stall it, while a
/// new connection makes both ends resend what is unacked.
const TCP_BACKLOG: usize = 1 << 20;
/// How long making a TCP connection may take.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
/// The pause after a failed attempt.
const RECONNECT: Duration = Duration::from_secs(1);
/// Connections a server keeps at once. One that brought no packet the
/// server could open is closed after `UNKNOWN_IDLE` (or for a newer one
/// when all are taken); one that did, after `IDLE` without another
/// (clients send at least every second).
const MAX_CONNS: usize = 256;
const IDLE: Duration = Duration::from_secs(120);
const UNKNOWN_IDLE: Duration = Duration::from_secs(10);
/// How long closing waits for the last packets (a detach) to go out.
const LINGER: Duration = Duration::from_millis(300);
const PSEUDO: u128 = 0xfd72_7368_7463_7000 << 64;

/// Servers that answered over TCP when UDP did not, in this process.
static TCP_PEERS: Mutex<BTreeSet<SocketAddr>> = Mutex::new(BTreeSet::new());

pub fn remember_tcp(peer: SocketAddr) {
    TCP_PEERS.lock().unwrap_or_else(|p| p.into_inner()).insert(peer);
}

pub fn prefers_tcp(peer: SocketAddr) -> bool {
    TCP_PEERS.lock().unwrap_or_else(|p| p.into_inner()).contains(&peer)
}

/// Whether the server's `addr` stands for a TCP connection.
pub fn is_tcp_addr(addr: SocketAddr) -> bool {
    matches!(addr.ip(), IpAddr::V6(ip) if u128::from(ip) >> 64 == PSEUDO >> 64)
}

/// A client's way to its server.
pub enum Wire {
    Udp { sock: Arc<UdpSocket>, peer: SocketAddr },
    Tcp(TcpWire),
}

pub struct TcpWire {
    peer: SocketAddr,
    stream: Option<TcpStream>,
    rbuf: Vec<u8>,
    wbuf: Vec<u8>,
    connecting: Option<mpsc::Receiver<io::Result<TcpStream>>>,
    retry_at: Instant,
    /// When the connection came up: one that breaks soon after (a server
    /// with every slot taken) is not made again at once.
    up_since: Instant,
}

fn prepare(stream: &TcpStream) -> io::Result<()> {
    stream.set_nodelay(true)?;
    stream.set_nonblocking(true)
}

/// Appends one framed packet; false when the backlog is full.
fn frame(wbuf: &mut Vec<u8>, packet: &[u8]) -> bool {
    if packet.len() > u16::MAX as usize || wbuf.len() + 2 + packet.len() > TCP_BACKLOG {
        return false;
    }
    wbuf.extend_from_slice(&(packet.len() as u16).to_be_bytes());
    wbuf.extend_from_slice(packet);
    true
}

/// Writes what the kernel takes; false once the connection is broken.
fn flush(stream: &mut TcpStream, wbuf: &mut Vec<u8>) -> bool {
    while !wbuf.is_empty() {
        match stream.write(wbuf) {
            Ok(0) => return false,
            Ok(n) => {
                wbuf.drain(..n);
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => {}
            Err(err) if err.kind() == ErrorKind::WouldBlock => return true,
            Err(_) => return false,
        }
    }
    true
}

/// Reads what is there into `rbuf`; false once the connection is closed.
fn fill(stream: &mut TcpStream, rbuf: &mut Vec<u8>) -> bool {
    let mut chunk = [0u8; 16384];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return false,
            Ok(n) => rbuf.extend_from_slice(&chunk[..n]),
            Err(err) if err.kind() == ErrorKind::Interrupted => {}
            Err(err) if err.kind() == ErrorKind::WouldBlock => return true,
            Err(_) => return false,
        }
    }
}

/// Takes the first whole packet off `rbuf`.
fn unframe(rbuf: &mut Vec<u8>) -> Option<Vec<u8>> {
    let len = u16::from_be_bytes([*rbuf.first()?, *rbuf.get(1)?]) as usize;
    if rbuf.len() < 2 + len {
        return None;
    }
    let packet = rbuf[2..2 + len].to_vec();
    rbuf.drain(..2 + len);
    Some(packet)
}

impl Wire {
    pub fn udp(sock: Arc<UdpSocket>, peer: SocketAddr) -> io::Result<Self> {
        sock.set_nonblocking(true)?;
        Ok(Wire::Udp { sock, peer })
    }

    /// A TCP connection to `peer`, made now.
    pub fn tcp(peer: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect_timeout(&peer, CONNECT_TIMEOUT)?;
        prepare(&stream)?;
        Ok(Wire::Tcp(TcpWire {
            peer,
            stream: Some(stream),
            rbuf: Vec::new(),
            wbuf: Vec::new(),
            connecting: None,
            retry_at: Instant::now(),
            up_since: Instant::now(),
        }))
    }

    pub fn is_tcp(&self) -> bool {
        matches!(self, Wire::Tcp(_))
    }

    pub fn peer(&self) -> SocketAddr {
        match self {
            Wire::Udp { peer, .. } => *peer,
            Wire::Tcp(tcp) => tcp.peer,
        }
    }

    /// What to poll: the descriptor (-1 while a TCP connection is down)
    /// and the events.
    pub fn pollfd(&self) -> libc::pollfd {
        let (fd, events): (RawFd, i16) = match self {
            Wire::Udp { sock, .. } => (sock.as_raw_fd(), libc::POLLIN),
            Wire::Tcp(tcp) => (
                tcp.stream.as_ref().map_or(-1, AsRawFd::as_raw_fd),
                if tcp.wbuf.is_empty() { libc::POLLIN } else { libc::POLLIN | libc::POLLOUT },
            ),
        };
        libc::pollfd { fd, events, revents: 0 }
    }

    /// Sends one packet; over a TCP connection that is down it is dropped.
    pub fn send(&mut self, packet: &[u8]) -> io::Result<()> {
        match self {
            Wire::Udp { sock, peer } => sock.send_to(packet, *peer).map(drop),
            Wire::Tcp(tcp) => {
                if tcp.stream.is_some() {
                    if frame(&mut tcp.wbuf, packet) {
                        tcp.flush();
                    } else {
                        tcp.broke(Instant::now());
                    }
                }
                Ok(())
            }
        }
    }

    /// Whether a TCP connection is down (and being made again).
    pub fn down(&self) -> bool {
        matches!(self, Wire::Tcp(tcp) if tcp.stream.is_none())
    }

    /// The next packet from the server into `buf`, if one is here.
    pub fn recv(&mut self, buf: &mut [u8]) -> Option<usize> {
        match self {
            Wire::Udp { sock, peer } => loop {
                match sock.recv_from(buf) {
                    // The peer's address, or its port on another address
                    // of the host (an answer from another interface).
                    Ok((n, from)) if from == *peer || from.port() == peer.port() => return Some(n),
                    Ok(_) => {}
                    Err(err) if err.kind() == ErrorKind::Interrupted => {}
                    Err(_) => return None,
                }
            },
            Wire::Tcp(tcp) => {
                if let Some(n) = unframe_into(&mut tcp.rbuf, buf) {
                    return Some(n);
                }
                let alive = tcp.stream.as_mut().is_some_and(|stream| fill(stream, &mut tcp.rbuf));
                if !alive && tcp.stream.is_some() {
                    tcp.broke(Instant::now());
                }
                unframe_into(&mut tcp.rbuf, buf)
            }
        }
    }

    /// Writes what waits and makes a lost TCP connection again; true when
    /// a new connection is up.
    pub fn service(&mut self, now: Instant) -> bool {
        match self {
            Wire::Tcp(tcp) => {
                tcp.flush();
                let down = tcp.stream.is_none();
                tcp.reconnect(now);
                down && tcp.stream.is_some()
            }
            Wire::Udp { .. } => false,
        }
    }

    /// When `service` should run again while a connection is being made.
    pub fn next_service(&self, now: Instant) -> Option<Instant> {
        match self {
            Wire::Tcp(tcp) if tcp.stream.is_none() => {
                Some(if tcp.connecting.is_some() { now + Duration::from_millis(50) } else { tcp.retry_at })
            }
            _ => None,
        }
    }

    /// Drops a TCP connection that may be dead (nothing heard for long,
    /// as after the client's address changed) and makes a new one.
    pub fn reset(&mut self, now: Instant) {
        if let Wire::Tcp(tcp) = self {
            if tcp.stream.is_some() {
                tcp.broke(now);
            }
            tcp.reconnect(now);
        }
    }
}

fn unframe_into(rbuf: &mut Vec<u8>, buf: &mut [u8]) -> Option<usize> {
    let packet = unframe(rbuf)?;
    let n = packet.len().min(buf.len());
    buf[..n].copy_from_slice(&packet[..n]);
    Some(n)
}

impl TcpWire {
    fn broke(&mut self, now: Instant) {
        self.stream = None;
        self.rbuf.clear();
        self.wbuf.clear();
        self.retry_at = if now.duration_since(self.up_since) < RECONNECT { now + RECONNECT } else { now };
    }

    fn flush(&mut self) {
        if let Some(stream) = self.stream.as_mut() {
            if !flush(stream, &mut self.wbuf) {
                self.broke(Instant::now());
            }
        }
    }

    /// Sends what waits and closes, reading until the server closes too:
    /// closing with unread data would reset the connection and lose the
    /// last packets (a detach).
    fn close(&mut self) {
        let Some(mut stream) = self.stream.take() else {
            return;
        };
        let end = Instant::now() + LINGER;
        let wait = |stream: &TcpStream, events| {
            let mut fd = libc::pollfd { fd: stream.as_raw_fd(), events, revents: 0 };
            let ms = end.saturating_duration_since(Instant::now()).as_millis() as i32;
            ms > 0 && unsafe { libc::poll(&mut fd, 1, ms) } > 0
        };
        while !self.wbuf.is_empty() && flush(&mut stream, &mut self.wbuf) && wait(&stream, libc::POLLOUT) {}
        let _ = stream.shutdown(std::net::Shutdown::Write);
        // Until the server's end, which follows ours, or nothing is unread.
        let mut sink = Vec::new();
        while fill(&mut stream, &mut sink) && !sink.is_empty() && wait(&stream, libc::POLLIN) {
            sink.clear();
        }
    }

    /// Connects on a thread, so the caller's loop never waits on it.
    fn reconnect(&mut self, now: Instant) {
        if self.stream.is_some() {
            return;
        }
        if let Some(pending) = &self.connecting {
            match pending.try_recv() {
                Err(mpsc::TryRecvError::Empty) => return,
                Ok(Ok(stream)) if prepare(&stream).is_ok() => {
                    self.stream = Some(stream);
                    self.connecting = None;
                    self.up_since = now;
                    return;
                }
                _ => {
                    self.connecting = None;
                    self.retry_at = now + RECONNECT;
                }
            }
        }
        if now >= self.retry_at {
            let (send, receive) = mpsc::channel();
            let peer = self.peer;
            std::thread::spawn(move || {
                let _ = send.send(TcpStream::connect_timeout(&peer, CONNECT_TIMEOUT));
            });
            self.connecting = Some(receive);
        }
    }
}

impl Drop for TcpWire {
    fn drop(&mut self) {
        self.close();
    }
}

/// The server's side: a listener on the UDP port, and its connections by
/// the address each stands for.
pub struct TcpSide {
    listener: Option<TcpListener>,
    conns: RefCell<BTreeMap<SocketAddr, Conn>>,
    next: u64,
}

struct Conn {
    stream: TcpStream,
    rbuf: Vec<u8>,
    wbuf: Vec<u8>,
    /// When it was made, then when it last brought a packet that opened.
    heard: Instant,
    known: bool,
    broken: bool,
}

impl TcpSide {
    /// Listens on `addr`.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self::with(Some(listener)))
    }

    /// No TCP: the port is taken, and the server serves UDP only.
    pub fn none() -> Self {
        Self::with(None)
    }

    fn with(listener: Option<TcpListener>) -> Self {
        Self {
            listener,
            conns: RefCell::new(BTreeMap::new()),
            // Unlike the addresses a previous process (before an upgrade)
            // handed out, which sessions may still name.
            next: crate::server::now_ms() << 16,
        }
    }

    /// The listener's and every connection's descriptor.
    pub fn pollfds(&self, fds: &mut Vec<libc::pollfd>) {
        if let Some(listener) = &self.listener {
            fds.push(libc::pollfd { fd: listener.as_raw_fd(), events: libc::POLLIN, revents: 0 });
        }
        for conn in self.conns.borrow().values() {
            let events = if conn.wbuf.is_empty() { libc::POLLIN } else { libc::POLLIN | libc::POLLOUT };
            fds.push(libc::pollfd { fd: conn.stream.as_raw_fd(), events, revents: 0 });
        }
    }

    /// Whether the connection `addr` stands for is open.
    pub fn has(&self, addr: SocketAddr) -> bool {
        self.conns.borrow().get(&addr).is_some_and(|conn| !conn.broken)
    }

    /// A packet from `addr` opened with a key the server holds.
    pub fn known(&self, addr: SocketAddr, now: Instant) {
        if let Some(conn) = self.conns.borrow_mut().get_mut(&addr) {
            conn.known = true;
            conn.heard = now;
        }
    }

    /// Sends one packet to the connection `addr` stands for, if it is still there.
    pub fn send(&self, packet: &[u8], addr: SocketAddr) {
        if let Some(conn) = self.conns.borrow_mut().get_mut(&addr) {
            conn.broken |= !frame(&mut conn.wbuf, packet) || !flush(&mut conn.stream, &mut conn.wbuf);
        }
    }

    /// Takes new connections, writes what waits, and returns the packets
    /// that arrived with the address of their connection.
    pub fn receive(&mut self, now: Instant) -> Vec<(Vec<u8>, SocketAddr)> {
        let conns = self.conns.get_mut();
        while let Some(listener) = &self.listener {
            match listener.accept() {
                Ok((stream, _)) => {
                    if conns.len() >= MAX_CONNS {
                        // The oldest unknown one makes room, else the new one goes.
                        let Some(oldest) = conns.iter().filter(|(_, c)| !c.known).min_by_key(|(_, c)| c.heard).map(|(a, _)| *a) else {
                            continue;
                        };
                        conns.remove(&oldest);
                    }
                    if prepare(&stream).is_err() {
                        continue;
                    }
                    self.next += 1;
                    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::from(PSEUDO | u128::from(self.next))), 0);
                    conns.insert(addr, Conn { stream, rbuf: Vec::new(), wbuf: Vec::new(), heard: now, known: false, broken: false });
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
        let mut packets = Vec::new();
        for (addr, conn) in conns.iter_mut() {
            if !fill(&mut conn.stream, &mut conn.rbuf) || !flush(&mut conn.stream, &mut conn.wbuf) {
                conn.broken = true;
            }
            while let Some(packet) = unframe(&mut conn.rbuf) {
                packets.push((packet, *addr));
            }
        }
        conns.retain(|_, conn| !conn.broken && now.duration_since(conn.heard) < if conn.known { IDLE } else { UNKNOWN_IDLE });
        packets
    }
}

/// Waits until `wire` may have a packet (or can take what waits), at most
/// `timeout`.
pub fn wait_readable(wire: &Wire, timeout: Duration) {
    let mut fd = wire.pollfd();
    if fd.fd < 0 {
        std::thread::sleep(timeout.min(Duration::from_millis(50)));
        return;
    }
    let ms = timeout.as_millis().clamp(1, i32::MAX as u128) as i32;
    unsafe { libc::poll(&mut fd, 1, ms) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_that_arrive_together_are_each_received() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut wire = Wire::tcp(listener.local_addr().unwrap()).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        let mut bytes = Vec::new();
        for packet in [&b"one"[..], b"two", b"three", b"four"] {
            assert!(frame(&mut bytes, packet));
        }
        // The last frame arrives cut in two.
        server.write_all(&bytes[..bytes.len() - 2]).unwrap();
        let mut buf = [0u8; 64];
        let mut got = Vec::new();
        let end = Instant::now() + Duration::from_secs(2);
        while got.len() < 3 && Instant::now() < end {
            match wire.recv(&mut buf) {
                Some(n) => got.push(buf[..n].to_vec()),
                None => wait_readable(&wire, Duration::from_millis(50)),
            }
        }
        assert_eq!(got, [b"one".to_vec(), b"two".to_vec(), b"three".to_vec()]);
        assert_eq!(wire.recv(&mut buf), None, "half a frame waits");
        server.write_all(&bytes[bytes.len() - 2..]).unwrap();
        wait_readable(&wire, Duration::from_secs(1));
        assert_eq!(wire.recv(&mut buf).map(|n| buf[..n].to_vec()), Some(b"four".to_vec()));
    }
}
