// TCP port forwarding over the session (ET's -L and -R). Each forwarded
// connection is a channel with its own pair of reliable byte streams; a
// FIN carries the final offset so either side can half-close.

use crate::proto::{Reader, Receiver, Sender, Writer, CHUNK};
use std::collections::{BTreeMap, VecDeque};
use std::io::{self, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::os::fd::AsRawFd;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Stream bytes and acks; the first one from the far side also means the
/// channel is open.
pub const OP_DATA: u8 = 0;
pub const OP_OPEN: u8 = 1;
pub const OP_RESET: u8 = 3;

pub const FLAG_FIN_SEEN: u8 = 1;

const NO_FIN: u64 = u64::MAX;
/// Bytes buffered per channel in each direction before reading stops.
const CHANNEL_BUFFER: usize = 256 * 1024;
const OPEN_RETRY: Duration = Duration::from_millis(300);
const OPEN_TIMEOUT: Duration = Duration::from_secs(15);
pub const MAX_CHANNELS: usize = 256;

/// A forward request: listen on `listen` and connect each accepted
/// connection to `target` on the other side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub bind: String,
    pub port: u16,
    pub target: String,
}

impl Spec {
    /// `[bind:]port:host:hostport`, as ssh's -L and -R take.
    pub fn parse(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.split(':').collect();
        let (bind, port, host, target_port) = match parts.as_slice() {
            [port, host, target_port] => ("127.0.0.1", *port, *host, *target_port),
            [bind, port, host, target_port] => (*bind, *port, *host, *target_port),
            _ => return Err(format!("bad forward {text}; use [bind:]port:host:hostport")),
        };
        let port: u16 = port.parse().map_err(|_| format!("bad port in {text}"))?;
        let target_port: u16 = target_port.parse().map_err(|_| format!("bad port in {text}"))?;
        if host.is_empty() {
            return Err(format!("bad host in {text}"));
        }
        let bind = if bind.is_empty() || bind == "localhost" { "127.0.0.1" } else { bind };
        Ok(Self {
            bind: bind.to_string(),
            port,
            target: format!("{host}:{target_port}"),
        })
    }

    pub fn encode(&self) -> String {
        format!("{}:{}:{}", self.bind, self.port, self.target)
    }

    pub fn decode(text: &str) -> Option<Self> {
        let (bind, rest) = text.split_once(':')?;
        let (port, target) = rest.split_once(':')?;
        Some(Self {
            bind: bind.to_string(),
            port: port.parse().ok()?,
            target: target.to_string(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message<'a> {
    pub channel: u32,
    pub op: u8,
    pub flags: u8,
    pub offset: u64,
    pub ack: u64,
    pub fin: u64,
    pub data: &'a [u8],
}

impl<'a> Message<'a> {
    pub fn encode(&self) -> Vec<u8> {
        Writer::new()
            .u32(self.channel)
            .u8(self.op)
            .u8(self.flags)
            .u64(self.offset)
            .u64(self.ack)
            .u64(self.fin)
            .rest(self.data)
            .finish()
    }

    pub fn decode(buf: &'a [u8]) -> Option<Self> {
        let mut r = Reader::new(buf);
        Some(Self {
            channel: r.u32()?,
            op: r.u8()?,
            flags: r.u8()?,
            offset: r.u64()?,
            ack: r.u64()?,
            fin: r.u64()?,
            data: r.rest(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// We asked the peer to open; waiting for OPENED.
    Opening,
    /// The peer asked us; our TCP connect is in progress.
    Connecting,
    Open,
    Closed,
}

struct Channel {
    tcp: Option<TcpStream>,
    connecting: Option<mpsc::Receiver<io::Result<TcpStream>>>,
    state: State,
    target: String,
    tx: Sender,
    rx: Receiver,
    to_tcp: VecDeque<u8>,
    fin_out: Option<u64>,
    fin_out_seen: bool,
    fin_in: Option<u64>,
    fin_in_done: bool,
    reset_sent: bool,
    last_open: Instant,
    started: Instant,
    reply_due: bool,
}

impl Channel {
    fn new(tcp: Option<TcpStream>, state: State, target: String, now: Instant) -> Self {
        Self {
            tcp,
            connecting: None,
            state,
            target,
            tx: Sender::new(0),
            rx: Receiver::new(0),
            to_tcp: VecDeque::new(),
            fin_out: None,
            fin_out_seen: false,
            fin_in: None,
            fin_in_done: false,
            reset_sent: false,
            last_open: now - OPEN_RETRY,
            started: now,
            reply_due: false,
        }
    }

    fn finished(&self) -> bool {
        self.fin_out_seen && self.fin_in_done && self.tx.pending() == 0 && self.to_tcp.is_empty()
    }
}

/// The forwarding state of one end of a session: listeners that open
/// channels, and the channels themselves.
pub struct Forwarder {
    /// Each listener, its target and whether poll found it ready.
    listeners: Vec<(TcpListener, String, bool)>,
    channels: BTreeMap<u32, Channel>,
    next_id: u32,
    /// Recently finished channel ids, so a late duplicate OPEN is not
    /// mistaken for a new connection.
    closed: VecDeque<u32>,
    resets: Vec<u32>,
    /// Read buffer for the channels' sockets, made on first use.
    buf: Vec<u8>,
}

impl Forwarder {
    /// `odd` ids for the client's channels, even for the server's, so the
    /// two sides never pick the same id.
    pub fn new(odd: bool) -> Self {
        Self {
            listeners: Vec::new(),
            channels: BTreeMap::new(),
            next_id: if odd { 1 } else { 2 },
            closed: VecDeque::new(),
            resets: Vec::new(),
            buf: Vec::new(),
        }
    }

    /// The id the next locally opened channel gets; handed over in an
    /// upgrade so ids the peer still remembers are not reused.
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    pub fn set_next_id(&mut self, id: u32) {
        self.next_id = id;
    }

    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty() && self.channels.is_empty()
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn listen(&mut self, spec: &Spec) -> io::Result<SocketAddr> {
        // An address literal only: a name lookup here would block the loop.
        let ip: std::net::IpAddr = spec
            .bind
            .parse()
            .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "bind address must be an IP address"))?;
        let listener = TcpListener::bind((ip, spec.port))?;
        listener.set_nonblocking(true)?;
        let addr = listener.local_addr()?;
        self.listeners.push((listener, spec.target.clone(), false));
        Ok(addr)
    }

    pub fn pollfds(&self, fds: &mut Vec<libc::pollfd>) {
        for (listener, _, _) in &self.listeners {
            fds.push(libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        }
        for channel in self.channels.values() {
            let Some(tcp) = &channel.tcp else {
                continue;
            };
            let mut events = 0;
            if !channel.to_tcp.is_empty() {
                events |= libc::POLLOUT;
            }
            if channel.fin_out.is_none() && channel.tx.pending() < CHANNEL_BUFFER {
                events |= libc::POLLIN;
            }
            fds.push(libc::pollfd {
                fd: tcp.as_raw_fd(),
                events,
                revents: 0,
            });
        }
    }

    /// Notes which listeners `poll` found ready, so `service` accepts on
    /// those only.
    pub fn mark_ready(&mut self, fds: &[libc::pollfd]) {
        for (listener, _, ready) in &mut self.listeners {
            let fd = listener.as_raw_fd();
            *ready |= fds.iter().any(|p| p.fd == fd && p.revents != 0);
        }
    }

    /// Accepts, connects, reads and writes whatever is ready.
    pub fn service(&mut self, now: Instant) {
        let mut accepted = Vec::new();
        for (listener, target, ready) in &mut self.listeners {
            if !std::mem::take(ready) {
                continue;
            }
            while let Ok((tcp, _)) = listener.accept() {
                accepted.push((tcp, target.clone()));
            }
        }
        for (tcp, target) in accepted {
            if self.channels.len() >= MAX_CHANNELS {
                continue;
            }
            let _ = tcp.set_nonblocking(true);
            let _ = tcp.set_nodelay(true);
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(2);
            self.channels.insert(id, Channel::new(Some(tcp), State::Opening, target, now));
        }
        if self.buf.is_empty() && !self.channels.is_empty() {
            self.buf = vec![0u8; 16384];
        }
        let buf = &mut self.buf;
        for channel in self.channels.values_mut() {
            if channel.state == State::Connecting {
                let outcome = channel.connecting.as_ref().map(mpsc::Receiver::try_recv);
                match outcome {
                    Some(Ok(Ok(tcp))) => {
                        let _ = tcp.set_nonblocking(true);
                        let _ = tcp.set_nodelay(true);
                        channel.tcp = Some(tcp);
                        channel.connecting = None;
                        channel.state = State::Open;
                        channel.reply_due = true;
                    }
                    Some(Err(mpsc::TryRecvError::Empty)) => {}
                    _ => channel.state = State::Closed,
                }
                continue;
            }
            let Some(tcp) = channel.tcp.as_mut() else {
                continue;
            };
            while !channel.to_tcp.is_empty() {
                let (front, _) = channel.to_tcp.as_slices();
                match tcp.write(front) {
                    Ok(0) => break,
                    Ok(n) => {
                        channel.to_tcp.drain(..n);
                    }
                    Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                    Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                    Err(_) => {
                        channel.state = State::Closed;
                        break;
                    }
                }
            }
            if channel.to_tcp.is_empty() && channel.fin_in.is_some_and(|fin| channel.rx.next >= fin) && !channel.fin_in_done {
                let _ = tcp.shutdown(std::net::Shutdown::Write);
                channel.fin_in_done = true;
                channel.reply_due = true;
            }
            if channel.state == State::Closed || channel.fin_out.is_some() {
                continue;
            }
            while channel.tx.pending() < CHANNEL_BUFFER {
                match tcp.read(buf) {
                    Ok(0) => {
                        channel.fin_out = Some(channel.tx.end());
                        channel.reply_due = true;
                        break;
                    }
                    Ok(n) => channel.tx.push(&buf[..n]),
                    Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                    Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                    Err(_) => {
                        channel.state = State::Closed;
                        break;
                    }
                }
            }
        }
    }

    /// Applies a message from the peer. `connect` is whether this side may
    /// open outgoing connections for the peer (the server, for -L).
    pub fn receive(&mut self, msg: &Message, now: Instant, allowed: &dyn Fn(&str) -> bool) {
        match msg.op {
            OP_OPEN => {
                if let Some(channel) = self.channels.get_mut(&msg.channel) {
                    channel.reply_due = true;
                    return;
                }
                if self.closed.contains(&msg.channel) {
                    self.resets.push(msg.channel);
                    return;
                }
                let target = String::from_utf8_lossy(msg.data).into_owned();
                let mut channel = Channel::new(None, State::Connecting, target.clone(), now);
                if self.channels.len() >= MAX_CHANNELS || !allowed(&target) {
                    channel.state = State::Closed;
                } else {
                    channel.connecting = Some(connect_in_background(target));
                }
                channel.reply_due = true;
                self.channels.insert(msg.channel, channel);
            }
            OP_DATA | OP_RESET => {
                let Some(channel) = self.channels.get_mut(&msg.channel) else {
                    if msg.op == OP_DATA {
                        self.resets.push(msg.channel);
                    }
                    return;
                };
                if msg.op == OP_RESET {
                    if channel.fin_out.is_some() && channel.tx.pending() == 0 {
                        // The peer already finished and forgot the channel.
                        channel.fin_out_seen = true;
                        channel.fin_in_done = true;
                    } else {
                        channel.state = State::Closed;
                    }
                    channel.reset_sent = true;
                    return;
                }
                if channel.state == State::Opening {
                    channel.state = State::Open;
                }
                channel.tx.on_ack(0, msg.ack, now, !msg.data.is_empty());
                if channel.fin_out.is_some() && msg.flags & FLAG_FIN_SEEN != 0 {
                    channel.fin_out_seen = true;
                }
                if msg.fin != NO_FIN {
                    channel.fin_in = Some(msg.fin);
                }
                let room = CHANNEL_BUFFER.saturating_sub(channel.to_tcp.len());
                let data = channel.rx.accept(0, msg.offset, msg.data, room);
                channel.to_tcp.extend(data);
                if !msg.data.is_empty() || msg.fin != NO_FIN {
                    channel.reply_due = true;
                }
            }
            _ => {}
        }
    }

    /// Messages to send now.
    pub fn outgoing(&mut self, now: Instant) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut done = Vec::new();
        for (&id, channel) in self.channels.iter_mut() {
            if channel.state == State::Closed {
                if !channel.reset_sent {
                    channel.reset_sent = true;
                    out.push(message(id, OP_RESET, 0, channel, 0, &[]));
                }
                done.push(id);
                continue;
            }
            match channel.state {
                State::Opening => {
                    if now.duration_since(channel.started) > OPEN_TIMEOUT {
                        channel.state = State::Closed;
                        continue;
                    }
                    if now.duration_since(channel.last_open) >= OPEN_RETRY {
                        channel.last_open = now;
                        let target = channel.target.clone();
                        out.push(message(id, OP_OPEN, 0, channel, 0, target.as_bytes()));
                    }
                    continue;
                }
                State::Connecting => continue,
                _ => {}
            }
            let flags = if channel.fin_in.is_some_and(|fin| channel.rx.next >= fin) { FLAG_FIN_SEEN } else { 0 };
            let mut sent = false;
            while let Some((offset, data)) = channel.tx.next_chunk(now) {
                out.push(message(id, OP_DATA, flags, channel, offset, &data));
                sent = true;
            }
            let fin_unacked = channel.fin_out.is_some() && !channel.fin_out_seen && channel.tx.pending() == 0;
            if !sent && (channel.reply_due || (fin_unacked && now.duration_since(channel.last_open) >= OPEN_RETRY)) {
                if fin_unacked {
                    channel.last_open = now;
                }
                let offset = channel.tx.end();
                out.push(message(id, OP_DATA, flags, channel, offset, &[]));
            }
            channel.reply_due = false;
            if channel.finished() {
                done.push(id);
            }
        }
        for id in done {
            self.channels.remove(&id);
            self.closed.push_back(id);
            if self.closed.len() > 1024 {
                self.closed.pop_front();
            }
        }
        for id in self.resets.drain(..) {
            out.push(
                Message {
                    channel: id,
                    op: OP_RESET,
                    flags: 0,
                    offset: 0,
                    ack: 0,
                    fin: NO_FIN,
                    data: &[],
                }
                .encode(),
            );
        }
        out
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.channels
            .values()
            .filter_map(|channel| channel.tx.deadline())
            .min()
    }

    /// True if a channel still waits on something only the timer can do.
    pub fn needs_timer(&self) -> bool {
        self.channels
            .values()
            .any(|c| c.state == State::Opening || c.state == State::Connecting || (c.fin_out.is_some() && !c.fin_out_seen))
    }

    pub fn close_all(&mut self) {
        self.listeners.clear();
        self.channels.clear();
    }
}

fn message(id: u32, op: u8, flags: u8, channel: &Channel, offset: u64, data: &[u8]) -> Vec<u8> {
    Message {
        channel: id,
        op,
        flags,
        offset,
        ack: channel.rx.next,
        fin: channel.fin_out.unwrap_or(NO_FIN),
        data: &data[..data.len().min(CHUNK)],
    }
    .encode()
}

/// Resolves and connects on a helper thread, so a slow DNS answer or SYN
/// never stalls the poll loop that serves every session.
fn connect_in_background(target: String) -> mpsc::Receiver<io::Result<TcpStream>> {
    let (done, result) = mpsc::channel();
    std::thread::spawn(move || {
        let attempt = (|| {
            let mut last = io::Error::new(ErrorKind::NotFound, "no address");
            for addr in target.to_socket_addrs()? {
                match TcpStream::connect_timeout(&addr, OPEN_TIMEOUT) {
                    Ok(tcp) => return Ok(tcp),
                    Err(err) => last = err,
                }
            }
            Err(last)
        })();
        let _ = done.send(attempt);
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    /// What the event loops do before `service`: poll, then mark what is ready.
    fn poll_ready(forwarder: &mut Forwarder) {
        let mut fds = Vec::new();
        forwarder.pollfds(&mut fds);
        unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, 0) };
        forwarder.mark_ready(&fds);
    }

    #[test]
    fn spec_parse() {
        let spec = Spec::parse("8080:localhost:3000").unwrap();
        assert_eq!(spec, Spec { bind: "127.0.0.1".into(), port: 8080, target: "localhost:3000".into() });
        assert_eq!(Spec::decode(&spec.encode()), Some(spec));
        assert_eq!(Spec::parse("0.0.0.0:1:h:2").unwrap().bind, "0.0.0.0");
        assert!(Spec::parse("x:y").is_err());
        assert!(Spec::parse("1:h:z").is_err());
    }

    /// Two forwarders wired back to back: a listener on one side, the
    /// target echo server reached through the other.
    #[test]
    fn channel_round_trip_with_half_close() {
        let echo = TcpListener::bind("127.0.0.1:0").unwrap();
        let echo_addr = echo.local_addr().unwrap();
        thread::spawn(move || {
            let (mut conn, _) = echo.accept().unwrap();
            let mut all = Vec::new();
            conn.read_to_end(&mut all).unwrap();
            conn.write_all(&all).unwrap();
            conn.write_all(b"|done").unwrap();
        });
        let mut near = Forwarder::new(true);
        let mut far = Forwarder::new(false);
        let local = near
            .listen(&Spec { bind: "127.0.0.1".into(), port: 0, target: echo_addr.to_string() })
            .unwrap();
        let payload: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        let sent = payload.clone();
        let client = thread::spawn(move || {
            let mut conn = TcpStream::connect(local).unwrap();
            conn.write_all(&sent).unwrap();
            conn.shutdown(std::net::Shutdown::Write).unwrap();
            let mut back = Vec::new();
            conn.read_to_end(&mut back).unwrap();
            back
        });
        let start = Instant::now();
        while !client.is_finished() {
            assert!(start.elapsed() < Duration::from_secs(10), "stalled");
            let now = Instant::now();
            poll_ready(&mut near);
            near.service(now);
            far.service(now);
            for bytes in near.outgoing(now) {
                far.receive(&Message::decode(&bytes).unwrap(), now, &|_| true);
            }
            for bytes in far.outgoing(now) {
                near.receive(&Message::decode(&bytes).unwrap(), now, &|_| true);
            }
            thread::sleep(Duration::from_millis(1));
        }
        let back = client.join().unwrap();
        assert_eq!(&back[..payload.len()], &payload[..]);
        assert_eq!(&back[payload.len()..], b"|done");
        for _ in 0..50 {
            let now = Instant::now();
            poll_ready(&mut near);
            near.service(now);
            far.service(now);
            for bytes in near.outgoing(now) {
                far.receive(&Message::decode(&bytes).unwrap(), now, &|_| true);
            }
            for bytes in far.outgoing(now) {
                near.receive(&Message::decode(&bytes).unwrap(), now, &|_| true);
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(near.channel_count(), 0);
        assert_eq!(far.channel_count(), 0);
    }

    #[test]
    fn refused_target_resets_channel() {
        let closed = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
        let mut near = Forwarder::new(true);
        let mut far = Forwarder::new(false);
        let local = near
            .listen(&Spec { bind: "127.0.0.1".into(), port: 0, target: closed.to_string() })
            .unwrap();
        let mut conn = TcpStream::connect(local).unwrap();
        conn.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut seen = false;
        for _ in 0..400 {
            let now = Instant::now();
            poll_ready(&mut near);
            near.service(now);
            far.service(now);
            for bytes in near.outgoing(now) {
                far.receive(&Message::decode(&bytes).unwrap(), now, &|_| true);
            }
            for bytes in far.outgoing(now) {
                near.receive(&Message::decode(&bytes).unwrap(), now, &|_| true);
            }
            seen |= near.channel_count() > 0;
            if seen && near.channel_count() == 0 && far.channel_count() == 0 {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(seen);
        let mut buf = [0u8; 8];
        assert_eq!(conn.read(&mut buf).unwrap_or(0), 0, "local connection closed");
    }
}
