use crate::crypto::{random_u64, Dir, Key};
use crate::forward::{self, Forwarder, Spec};
use crate::net::AuthStore;
use crate::proto::{
    self, encode_list, Action, Attached, ClientStream, Hello, Packet, Ready, Receiver, ReplayWindow, Sender, TmuxInfo,
    ServerStream, SessionInfo, FLAG_DETACH, FLAG_EXITED, FLAG_REDRAW, KIND_ATTACHED, KIND_CHANNEL, KIND_CONFIRM,
    KIND_DENIED, KIND_HELLO, KIND_KICKED, KIND_LIST, KIND_LIST_REPLY, KIND_PING, KIND_PONG, KIND_READY, KIND_STREAM,
};
use crate::pty::{kill_group, try_wait, Pty, SpawnOptions};
use crate::terminal::Terminal;
use crate::wire::{is_tcp_addr, TcpSide};
use std::collections::{BTreeMap, VecDeque};
use std::io::{self, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Unacked output kept per attached client. Past this the client is too far
/// behind, so the stream restarts from a repaint of the current screen.
pub const OUTPUT_BACKLOG: usize = 1 << 20;
/// Scrollback lines replayed into the client's terminal on attach.
pub const ATTACH_HISTORY: usize = 1000;
/// Client input accepted but not yet written to the pty.
const INPUT_BACKLOG: usize = 256 * 1024;
/// Stop sending to a client address not heard from for this long.
const ASSOCIATION_TIMEOUT: Duration = Duration::from_secs(40);
/// A finished session is dropped after its client saw the end, or after this.
const ENDED_LINGER: Duration = Duration::from_secs(60);
/// Accepted clock skew for HELLO, LIST and KILL, which also bounds replays.
const MAX_SKEW_MS: u64 = 120_000;
/// Pty output read per session per loop turn, so one busy session does not
/// hold up the others' input.
const READ_BUDGET: usize = 64 * 1024;
/// Output that would take longer than this to drain at a window per round
/// trip is dropped for a repaint, so a slow link shows the current screen.
/// Responsiveness first: more output is skipped (and missing from the
/// local scrollback) than with a longer limit.
const LAG_LIMIT: Duration = Duration::from_millis(100);
/// After falling behind, output goes as screens (as mosh sends state) until
/// updates have kept up (no queue on the path, nothing waiting to go) this
/// long: the link stays free of queued frames, so what shows is fresh.
const SYNC_SPAN: Duration = Duration::from_secs(1);
/// Output read at once from which, over a round trip of `DEFLATE_RTT` or
/// more, the screens-only mode starts at once.
const BURST: usize = 32 * 1024;
/// A program animates once this many writes in a row of `FRAME_BYTES` or
/// more were each followed by the next within `FRAME_GAP`: then the
/// screens-only mode may start before the link's rate is known, and
/// updates follow its step. A command's output (one burst) or keys'
/// echoes (small) are not frames; more than one in a row would hold the
/// mode back on a slow link (a frame read in pieces resets the count).
const ANIMATION_FRAMES: u32 = 1;
const FRAME_GAP: Duration = Duration::from_millis(100);
const FRAME_BYTES: usize = 512;
/// The least time between updates in the screens-only mode (mosh's least
/// frame interval), unless the program draws faster: then its step, down
/// to `MIN_SCREEN_GAP`.
const SCREEN_GAP: Duration = Duration::from_millis(20);
const MIN_SCREEN_GAP: Duration = Duration::from_millis(8);
/// On a slow link, how long output that came while the link was busy waits
/// for newer output before it goes (most of a frame at 30 a second).
const FRESH_WAIT: Duration = Duration::from_millis(20);
/// In the screens-only mode, how long an update waits for the program to
/// finish the escape sequence it is in the middle of (the rest of a frame
/// read in pieces comes within a few ms): taken mid-sequence, the update
/// is a repaint, a new generation whose start the ack clock holds back (a
/// frame split over a round trip, a stall at RTT 300 ms).
const TAIL_WAIT: Duration = Duration::from_millis(20);
/// On a slow link, how much later than the link is through with the last
/// update the next may go: a little room keeps a queue from forming on the
/// link, which would age every update after it.
const SLOW_ROOM: Duration = Duration::from_millis(5);
/// Resend share above which output goes twice while the link has room (a
/// lossy path then rarely holds a frame back for a retransmission), and
/// below which it stops: in between it keeps what it was doing, since
/// sending twice itself hides the loss.
const DOUBLE_ON: f64 = 0.005;
const DOUBLE_OFF: f64 = 0.001;
/// Round trip from which output is compressed (when the client can take
/// it): below it the link is not what holds output up, CPU would be.
const DEFLATE_RTT: Duration = Duration::from_millis(20);
/// Client input is acked by the echo that usually follows it within this
/// time, or by an empty packet when none does.
const INPUT_ACK_DELAY: Duration = Duration::from_millis(5);
/// Largest terminal a client may ask for; bounds the emulator's memory.
const MAX_COLS: u16 = 512;
const MAX_ROWS: u16 = 256;
/// Unacked output above which a Ctrl-C from the client drops the backlog
/// and repaints instead, so the interrupt shows up at once (as in ET).
const INTERRUPT_FLUSH: usize = 64 * 1024;
/// Handshakes waiting for CONFIRM, and how long one may wait.
const MAX_PENDING: usize = 256;
const PENDING_LIFETIME: Duration = Duration::from_secs(30);

pub struct Server {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

struct Shared {
    state: Mutex<State>,
    stop: AtomicBool,
}

struct State {
    socket: UdpSocket,
    tcp: TcpSide,
    /// Token keys by the id clients put in the header.
    keys: BTreeMap<u64, Key>,
    auth_path: Option<PathBuf>,
    auth_stamp: Option<(SystemTime, SystemTime, u64, u32)>,
    auth_checked: Instant,
    sessions: BTreeMap<u64, Session>,
    /// Attachment id → session, for packets sealed with attachment keys.
    attach_index: BTreeMap<u64, u64>,
    pending: BTreeMap<u64, Pending>,
    /// Answers to HELLO, LIST and PING by request nonce, with the address
    /// they went to.
    replies: VecDeque<([u8; 24], SocketAddr, Instant, Vec<u8>)>,
    zombies: Vec<(i32, Instant)>,
    drop_incoming: bool,
    drop_udp: bool,
    disconnect_timeout: Option<Duration>,
    /// Binary to exec and directory for the state file on SIGUSR2.
    upgrade: Option<(PathBuf, PathBuf)>,
    tmux: TmuxList,
}

/// The host's tmux sessions for LIST. tmux runs on a thread so the loop
/// never waits on it; a LIST that wants them waits for the answer (at
/// most `TMUX_WAIT`), and an answer serves LISTs for `TMUX_FRESH`.
#[derive(Default)]
struct TmuxList {
    known: Option<(Instant, Vec<crate::tmux::Found>)>,
    asking: Option<(Instant, std::sync::mpsc::Receiver<Vec<crate::tmux::Found>>)>,
    /// LISTs waiting for it: key, nonce, address.
    waiting: Vec<(Key, [u8; 24], SocketAddr)>,
}

const TMUX_FRESH: Duration = Duration::from_secs(2);
const TMUX_WAIT: Duration = Duration::from_secs(1);

/// A handshake between READY and CONFIRM.
struct Pending {
    key: Key,
    /// Id of the token key the HELLO came with.
    token: u64,
    hello: Hello,
    expires: Instant,
    /// The ATTACHED answer once done and where it went; a retried CONFIRM
    /// from that address gets it again.
    done: Option<(SocketAddr, Vec<u8>)>,
}

struct Session {
    id: u64,
    host: String,
    label: String,
    command: String,
    created: SystemTime,
    pty: Option<Pty>,
    pid: i32,
    closed_at: Option<Instant>,
    exit_code: Option<i32>,
    ended_at: Option<Instant>,
    term: Terminal,
    /// Output after the last point where no escape sequence was half done;
    /// sent after every repaint so the program's next bytes still fit.
    tail: Vec<u8>,
    pty_in: VecDeque<u8>,
    attach: Option<Attachment>,
    last_active: Instant,
    last_client: Instant,
}

struct Attachment {
    id: u64,
    /// This attachment's own keys, from the X25519 handshake.
    key: Key,
    /// Id of the token key it was made with; removing that token from the
    /// authorized file ends the attachment.
    token: u64,
    addr: SocketAddr,
    last_heard: Instant,
    /// When the current address was last heard from, and a new address
    /// seen once (see `State::heard`).
    heard_at_addr: Instant,
    candidate: Option<(SocketAddr, u8)>,
    /// Nonces seen from this client; pinned to its process by the HELLO.
    window: ReplayWindow,
    input: Receiver,
    output: Sender,
    reply_due: bool,
    /// When client input received is acked if no output carries the ack first.
    ack_at: Option<Instant>,
    /// The client fell behind before it had the last repaint: output is
    /// dropped until that repaint is acked, then a new one goes out.
    repaint_due: bool,
    /// Until when the client counts as behind the link: output then goes
    /// as the latest screen only, one repaint in flight at a time.
    syncing_until: Option<Instant>,
    /// When the last screen went in that mode.
    screen_at: Option<Instant>,
    /// Since when the program's output stops in the middle of an escape
    /// sequence (see `TAIL_WAIT`).
    mid_sequence: Option<Instant>,
    /// In that mode, the screen the client will have once it has all that
    /// was sent: what the next update is the difference from.
    shadow: Option<Terminal>,
    /// The client inflates `FLAG_DEFLATE` datagrams (HELLO features).
    deflate: bool,
    /// The client writes to a terminal, so screen updates may stand in
    /// for the bytes (HELLO features); a pipe gets every byte.
    screens: bool,
    /// The client inflates a generation sent as one deflate stream, and
    /// whether it asked for that on any link (HELLO features).
    zstream: bool,
    compress: bool,
    /// The generation under way is one deflate stream: its compressor.
    z: Option<proto::Deflater>,
    /// Output not yet in the stream: for the compressor (one flush per
    /// pump), or until the plain generation before is all acked.
    zraw: Vec<u8>,
    /// The generation `stage` opened to start compressing, which repaints
    /// nothing.
    switched: Option<u32>,
    /// Bytes of the last screen update as sent, and how many went.
    update_len: usize,
    updates: u32,
    /// When the program last wrote, and the usual time between its writes
    /// lately.
    output_at: Option<Instant>,
    output_gap: Duration,
    /// Writes in a row that look like an animation's frames (see
    /// `animating`), and the bytes of the write under way.
    frames: u32,
    step_bytes: usize,
    /// Output the program wrote: since when the span under way runs, its
    /// bytes, and the last whole span's.
    made: (Instant, usize, usize),
    /// Stream bytes per deflated byte lately: the first slice to try.
    ratio: f64,
    /// Output goes twice (see `DOUBLE_ON`); from the start, until the
    /// path shows it need not.
    doubling: bool,
    end_offset: Option<u64>,
    forward: Forwarder,
    /// Listeners are closed while the client is away and reopened when it
    /// is heard again.
    forwards_closed: bool,
    /// The client's `-R` specs, kept to listen again.
    remote_forwards: Vec<String>,
}

impl Server {
    /// A server on a loopback port for tests, with no tokens until `allow`.
    pub fn spawn() -> Self {
        Self::bind("127.0.0.1:0".parse().expect("addr"), &AuthStore::new()).expect("server bind")
    }

    pub fn bind(addr: SocketAddr, store: &AuthStore) -> io::Result<Self> {
        Self::start(addr, store, None, None)
    }

    /// Binds and serves; `auth_path` is re-read when it changes, and
    /// sessions without a client for `disconnect_timeout` are hung up.
    pub fn start(
        addr: SocketAddr,
        store: &AuthStore,
        auth_path: Option<PathBuf>,
        disconnect_timeout: Option<Duration>,
    ) -> io::Result<Self> {
        let (socket, tcp) = bind_both(addr)?;
        let auth_stamp = auth_path.as_ref().and_then(modified);
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                socket,
                tcp,
                keys: keys_of(store),
                auth_path,
                auth_stamp,
                auth_checked: Instant::now(),
                sessions: BTreeMap::new(),
                attach_index: BTreeMap::new(),
                pending: BTreeMap::new(),
                replies: VecDeque::new(),
                zombies: Vec::new(),
                drop_incoming: false,
                drop_udp: false,
                disconnect_timeout,
                upgrade: None,
                tmux: TmuxList::default(),
            }),
            stop: AtomicBool::new(false),
        });
        Self::run(shared)
    }

    fn run(shared: Arc<Shared>) -> io::Result<Self> {
        let bg = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("rsh-server".into())
            .spawn(move || serve(bg))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    /// The `rsh serve` process: continues the sessions a previous binary
    /// handed over through an upgrade, or binds `addr` afresh. On SIGUSR2
    /// it execs `exe`, keeping every session. Returns how many sessions
    /// were taken over.
    pub fn serve_process(
        addr: SocketAddr,
        store: &AuthStore,
        auth_path: PathBuf,
        disconnect_timeout: Option<Duration>,
        exe: PathBuf,
    ) -> io::Result<(Self, usize)> {
        let dir = auth_path.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let handoff = crate::upgrade::take_handoff().transpose()?;
        let Handoff {
            socket,
            sessions,
            zombies,
            replies,
        } = match handoff {
            Some(bytes) => import(&bytes).ok_or_else(|| io::Error::other("unreadable upgrade state"))?,
            None => Handoff {
                socket: bind_udp(addr)?,
                sessions: BTreeMap::new(),
                zombies: Vec::new(),
                replies: VecDeque::new(),
            },
        };
        let resumed = sessions.len();
        // TCP connections are not handed over: their clients connect again.
        // (`rsh serve` has a given port: no picking again.)
        let tcp = bind_tcp(socket.local_addr()?);
        let attach_index = sessions
            .values()
            .filter_map(|s| s.attach.as_ref().map(|a| (a.id, s.id)))
            .collect();
        let auth_stamp = modified(&auth_path);
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                socket,
                tcp,
                keys: keys_of(store),
                auth_path: Some(auth_path),
                auth_stamp,
                auth_checked: Instant::now(),
                sessions,
                attach_index,
                pending: BTreeMap::new(),
                replies,
                zombies,
                drop_incoming: false,
                drop_udp: false,
                disconnect_timeout,
                upgrade: Some((exe, dir)),
                tmux: TmuxList::default(),
            }),
            stop: AtomicBool::new(false),
        });
        Ok((Self::run(shared)?, resumed))
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.shared.state.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    pub fn addr(&self) -> SocketAddr {
        self.state().socket.local_addr().expect("addr")
    }

    pub fn allow(&self, host: &str, token: &str) {
        let _ = host;
        let key = Key::from_token(token);
        self.state().keys.insert(key.id(), key);
    }

    pub fn set_drop_incoming(&self, drop: bool) {
        self.state().drop_incoming = drop;
    }

    /// Drops what comes over UDP only, as a network that lets only TCP through.
    pub fn set_drop_udp(&self, drop: bool) {
        self.state().drop_udp = drop;
    }

    pub fn session_title(&self, id: u64) -> Option<String> {
        self.state().sessions.get(&id).map(|s| s.term.title().to_string())
    }

    pub fn session_screen(&self, id: u64) -> Option<Vec<String>> {
        self.state().sessions.get(&id).map(|s| {
            let rows = s.term.size().1 as usize;
            (0..rows).map(|row| s.term.screen_line(row)).collect()
        })
    }

    pub fn session_ids(&self) -> Vec<u64> {
        self.state().sessions.keys().copied().collect()
    }

    pub fn last_addr(&self, id: u64) -> Option<SocketAddr> {
        self.state()
            .sessions
            .get(&id)
            .and_then(|s| s.attach.as_ref().map(|a| a.addr))
    }

    pub fn backlog(&self, id: u64) -> Option<usize> {
        self.state()
            .sessions
            .get(&id)
            .map(|s| s.attach.as_ref().map_or(0, |a| a.output.pending()) + s.pty_in.len())
    }

    /// Moves the server to a new loopback port; sessions stay.
    pub fn rebind(&self) -> io::Result<SocketAddr> {
        let (socket, tcp) = bind_both("127.0.0.1:0".parse().expect("addr"))?;
        let addr = socket.local_addr()?;
        let mut state = self.state();
        state.socket = socket;
        state.tcp = tcp;
        Ok(addr)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        let mut state = self.state();
        for session in state.sessions.values() {
            if let Some(pty) = &session.pty {
                pty.hangup();
            }
        }
        let pids: Vec<i32> = state.sessions.values().map(|s| s.pid).collect();
        state.sessions.clear();
        let deadline = Instant::now() + Duration::from_millis(500);
        let mut waiting = pids;
        while !waiting.is_empty() && Instant::now() < deadline {
            waiting.retain(|pid| try_wait(*pid).is_none());
            thread::sleep(Duration::from_millis(10));
        }
        for pid in waiting {
            kill_group(pid);
            let _ = try_wait(pid);
        }
    }
}

/// Sends to a client over UDP, or over the TCP connection `addr` stands for.
fn send_to(socket: &UdpSocket, tcp: &TcpSide, bytes: &[u8], addr: SocketAddr) {
    if is_tcp_addr(addr) {
        tcp.send(bytes, addr);
    } else {
        let _ = socket.send_to(bytes, addr);
    }
}

/// The UDP socket and a TCP listener on the same port. A port the system
/// picks is picked again until TCP gets it too; on a given port a server
/// without TCP (taken by another program) still serves UDP.
fn bind_both(addr: SocketAddr) -> io::Result<(UdpSocket, TcpSide)> {
    let mut tries = 0;
    loop {
        let socket = bind_udp(addr)?;
        let local = socket.local_addr()?;
        match TcpSide::bind(local) {
            Ok(tcp) => return Ok((socket, tcp)),
            Err(err) if addr.port() == 0 && err.kind() == ErrorKind::AddrInUse && tries < 32 => tries += 1,
            Err(err) if addr.port() == 0 => return Err(err),
            Err(err) => {
                eprintln!("rsh: no TCP on {local}: {err}");
                return Ok((socket, TcpSide::none()));
            }
        }
    }
}

fn bind_tcp(addr: SocketAddr) -> TcpSide {
    TcpSide::bind(addr).unwrap_or_else(|err| {
        eprintln!("rsh: no TCP on {addr}: {err}");
        TcpSide::none()
    })
}

fn bind_udp(addr: SocketAddr) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(addr)?;
    socket.set_nonblocking(true)?;
    set_buffers(&socket);
    Ok(socket)
}

pub(crate) fn set_buffers(socket: &UdpSocket) {
    let size: libc::c_int = 1 << 20;
    for option in [libc::SO_SNDBUF, libc::SO_RCVBUF] {
        unsafe {
            libc::setsockopt(
                socket.as_raw_fd(),
                libc::SOL_SOCKET,
                option,
                (&size as *const libc::c_int).cast(),
                std::mem::size_of::<libc::c_int>() as libc::socklen_t,
            );
        }
    }
}

fn keys_of(store: &AuthStore) -> BTreeMap<u64, Key> {
    store
        .entries()
        .into_iter()
        .map(|(_, token)| Key::from_token(&token))
        .map(|key| (key.id(), key))
        .collect()
}

/// Listens for the client's `-R` forwards, on loopback only (as ssh's
/// GatewayPorts=no): the client must not open the server's network.
fn listen_remote(specs: &[String]) -> (Forwarder, Vec<String>) {
    let mut forward = Forwarder::new(false);
    let mut notes = Vec::new();
    for text in specs {
        match Spec::decode(text) {
            Some(spec) if !spec.bind.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback()) => {
                notes.push(format!("remote forward {}:{}: only loopback addresses may listen on the server", spec.bind, spec.port));
            }
            Some(spec) => {
                if let Err(err) = forward.listen(&spec) {
                    notes.push(format!("remote forward {}:{} -> {}: {err}", spec.bind, spec.port, spec.target));
                }
            }
            None => notes.push(format!("bad remote forward {text}")),
        }
    }
    (forward, notes)
}

/// Changes when the file is rewritten, replaced, removed or chmod-ed.
fn modified(path: &PathBuf) -> Option<(SystemTime, SystemTime, u64, u32)> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path).ok()?;
    let changed = UNIX_EPOCH + Duration::new(meta.ctime().max(0) as u64, meta.ctime_nsec().clamp(0, 999_999_999) as u32);
    Some((meta.modified().ok()?, changed, meta.ino(), meta.mode()))
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn serve(shared: Arc<Shared>) {
    let mut buf = vec![0u8; 65536];
    while !shared.stop.load(Ordering::Relaxed) {
        let (mut fds, timeout) = {
            let state = shared.state.lock().unwrap_or_else(|p| p.into_inner());
            let mut fds = vec![libc::pollfd {
                fd: state.socket.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            }];
            state.tcp.pollfds(&mut fds);
            for session in state.sessions.values() {
                if let Some(attach) = &session.attach {
                    attach.forward.pollfds(&mut fds);
                }
                if let Some(pty) = &session.pty {
                    let mut events = libc::POLLIN;
                    if !session.pty_in.is_empty() {
                        events |= libc::POLLOUT;
                    }
                    fds.push(libc::pollfd {
                        fd: pty.fd(),
                        events,
                        revents: 0,
                    });
                }
            }
            (fds, state.next_timeout(Instant::now()))
        };
        unsafe {
            libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout.as_millis().min(200) as i32);
        }
        let mut state = shared.state.lock().unwrap_or_else(|p| p.into_inner());
        let now = Instant::now();
        state.receive(&mut buf, now);
        for fd in &fds[1..] {
            if fd.revents == 0 {
                continue;
            }
            if let Some(id) = state.session_by_fd(fd.fd) {
                if fd.revents & libc::POLLOUT != 0 {
                    state.flush_input(id);
                }
                if fd.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
                    state.read_pty(id, &mut buf, now);
                }
            }
        }
        state.forwards(&fds, Instant::now());
        state.timers(Instant::now());
        if crate::upgrade::take_request() {
            state.upgrade();
        }
    }
}

/// Version of the state `export` writes; a binary that reads another one
/// is refused before exec (`rsh --handoff-format`).
pub const HANDOFF_FORMAT: u32 = 3;
const HANDOFF_MAGIC: &[u8] = b"RSHUP3";

fn age_ms(at: Instant, now: Instant) -> u64 {
    now.saturating_duration_since(at).as_millis() as u64
}

fn since(now: Instant, ms: u64) -> Instant {
    now.checked_sub(Duration::from_millis(ms)).unwrap_or(now)
}

/// What a previous process handed over in a live upgrade.
struct Handoff {
    socket: UdpSocket,
    sessions: BTreeMap<u64, Session>,
    zombies: Vec<(i32, Instant)>,
    replies: VecDeque<([u8; 24], SocketAddr, Instant, Vec<u8>)>,
}

/// Reads what `State::export` wrote: the socket, every session with its
/// screen rebuilt from the rendered terminal, unreaped children and the
/// HELLO/KILL replies that must still not run twice.
fn import(bytes: &[u8]) -> Option<Handoff> {
    use std::os::fd::FromRawFd;
    let mut r = proto::Reader::new(bytes);
    if r.bytes()? != HANDOFF_MAGIC {
        return None;
    }
    let now = Instant::now();
    let prefix: [u8; 16] = r.bytes_fixed(16)?.try_into().ok()?;
    if !crate::crypto::resume_nonces(prefix, r.u64()?) {
        return None;
    }
    let socket_fd = r.i32()?;
    let socket = unsafe { UdpSocket::from_raw_fd(socket_fd) };
    unsafe {
        libc::fcntl(socket_fd, libc::F_SETFD, libc::FD_CLOEXEC);
    }
    let _ = socket.set_nonblocking(true);
    let mut sessions = BTreeMap::new();
    for _ in 0..r.u16()? {
        let id = r.u64()?;
        let host = r.str()?;
        let label = r.str()?;
        let command = r.str()?;
        let created = UNIX_EPOCH + Duration::from_millis(r.u64()?);
        let pid = r.i32()?;
        let fd = r.i32()?;
        let closed = r.u8()? != 0;
        let exit_code = if r.u8()? != 0 { Some(r.i32()?) } else { None };
        let ended = r.u8()? != 0;
        let (cols, rows) = (r.u16()?, r.u16()?);
        let mut term = Terminal::with_size(cols, rows);
        term.feed(r.bytes()?);
        let tail = r.bytes()?.to_vec();
        term.feed(&tail);
        let pty_in: VecDeque<u8> = r.bytes()?.iter().copied().collect();
        let last_active = since(now, r.u64()?);
        let last_client = since(now, r.u64()?);
        let mut session = Session {
            id,
            host,
            label,
            command,
            created,
            pty: (fd >= 0).then(|| Pty::adopt(fd, pid)),
            pid,
            closed_at: closed.then_some(now),
            exit_code,
            ended_at: ended.then_some(now),
            term,
            tail,
            pty_in,
            attach: None,
            last_active,
            last_client,
        };
        if r.u8()? == 1 {
            let attach_id = r.u64()?;
            let c2s: [u8; 32] = r.bytes_fixed(32)?.try_into().ok()?;
            let s2c: [u8; 32] = r.bytes_fixed(32)?.try_into().ok()?;
            let token = r.u64()?;
            let addr: SocketAddr = r.str()?.parse().ok()?;
            let last_heard = since(now, r.u64()?);
            let window = ReplayWindow::load(&mut r)?;
            let mut input = Receiver::new(0);
            input.next = r.u64()?;
            let mut output = Sender::new(r.u32()?);
            output.reset(&session.screen(0));
            output.set_lossless(is_tcp_addr(addr));
            let remote_forwards = r.strs()?;
            let (mut forward, notes) = listen_remote(&remote_forwards);
            forward.set_next_id(r.u32()?);
            for note in notes {
                eprintln!("rsh: after upgrade: {note}");
            }
            session.attach = Some(Attachment {
                id: attach_id,
                key: Key::from_parts(c2s, s2c),
                token,
                addr,
                last_heard,
                heard_at_addr: last_heard,
                candidate: None,
                window,
                input,
                output,
                reply_due: true,
                ack_at: None,
                repaint_due: false,
                syncing_until: None,
                screen_at: None,
                mid_sequence: None,
                shadow: None,
                deflate: false,
                screens: false,
                zstream: false,
                compress: false,
                z: None,
                zraw: Vec::new(),
                update_len: 0,
                updates: 0,
                switched: None,
                output_at: None,
                output_gap: Duration::ZERO,
                frames: 0,
                step_bytes: 0,
                made: (now, 0, 0),
                ratio: 4.0,
                doubling: true,
                end_offset: None,
                forward,
                forwards_closed: false,
                remote_forwards,
            });
        }
        sessions.insert(id, session);
    }
    let zombies = (0..r.u16()?).map(|_| r.i32().map(|pid| (pid, now))).collect::<Option<Vec<_>>>()?;
    let mut replies = VecDeque::new();
    for _ in 0..r.u16()? {
        let nonce: [u8; 24] = r.bytes_fixed(24)?.try_into().ok()?;
        let addr: SocketAddr = r.str()?.parse().ok()?;
        let at = since(now, r.u64()?);
        replies.push_back((nonce, addr, at, r.bytes()?.to_vec()));
    }
    Some(Handoff {
        socket,
        sessions,
        zombies,
        replies,
    })
}

impl State {
    fn next_timeout(&self, now: Instant) -> Duration {
        let mut wait = Duration::from_millis(if self.tmux.asking.is_some() { 10 } else { 200 });
        for session in self.sessions.values() {
            let reachable = session
                .attach
                .as_ref()
                .is_some_and(|a| now.duration_since(a.last_heard) <= ASSOCIATION_TIMEOUT);
            if let Some(deadline) = session.attach.as_ref().and_then(|a| a.output.deadline()).filter(|_| reachable) {
                wait = wait.min(deadline.saturating_duration_since(now));
            }
            if let Some(at) = session.attach.as_ref().and_then(|a| a.ack_at).filter(|_| reachable) {
                wait = wait.min(at.saturating_duration_since(now));
            }
            if let Some(at) = session.attach.as_ref().and_then(Attachment::next_screen_at).filter(|_| reachable) {
                wait = wait.min(at.saturating_duration_since(now));
            }
            if session.closed_at.is_some() && session.exit_code.is_none() {
                wait = wait.min(Duration::from_millis(20));
            }
            if let Some(attach) = session.attach.as_ref().filter(|_| reachable) {
                if let Some(deadline) = attach.forward.deadline() {
                    wait = wait.min(deadline.saturating_duration_since(now));
                }
                if attach.forward.needs_timer() {
                    wait = wait.min(Duration::from_millis(100));
                }
            }
        }
        wait.max(Duration::from_millis(1))
    }

    /// Writes everything a new binary needs to carry on, and lists the fds
    /// it must inherit. Sessions whose shell already ended are left to end.
    fn export(&self) -> (Vec<u8>, Vec<i32>) {
        let now = Instant::now();
        let mut w = proto::Writer::new();
        let mut fds = vec![self.socket.as_raw_fd()];
        let (prefix, counter) = crate::crypto::nonce_state();
        w.bytes(HANDOFF_MAGIC).rest(&prefix).u64(counter).i32(self.socket.as_raw_fd());
        w.u16(self.sessions.len() as u16);
        for s in self.sessions.values() {
            let fd = s.pty.as_ref().map_or(-1, Pty::fd);
            if fd >= 0 {
                fds.push(fd);
            }
            let (cols, rows) = s.term.size();
            let created = s.created.duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
            let pty_in: Vec<u8> = s.pty_in.iter().copied().collect();
            w.u64(s.id)
                .str(&s.host)
                .str(&s.label)
                .str(&s.command)
                .u64(created)
                .i32(s.pid)
                .i32(fd)
                .u8(u8::from(s.closed_at.is_some()));
            match s.exit_code {
                Some(code) => w.u8(1).i32(code),
                None => w.u8(0),
            };
            w.u8(u8::from(s.ended_at.is_some()))
                .u16(cols)
                .u16(rows)
                .bytes(&s.term.render(usize::MAX))
                .bytes(&s.tail)
                .bytes(&pty_in)
                .u64(age_ms(s.last_active, now))
                .u64(age_ms(s.last_client, now));
            match &s.attach {
                None => {
                    w.u8(0);
                }
                Some(a) => {
                    let (c2s, s2c) = a.key.parts();
                    w.u8(1)
                        .u64(a.id)
                        .rest(&c2s)
                        .rest(&s2c)
                        .u64(a.token)
                        .str(&a.addr.to_string())
                        .u64(age_ms(a.last_heard, now));
                    a.window.save(&mut w);
                    w.u64(a.input.next).u32(a.output.gen).strs(&a.remote_forwards).u32(a.forward.next_id());
                }
            }
        }
        w.u16(self.zombies.len() as u16);
        for (pid, _) in &self.zombies {
            w.i32(*pid);
        }
        w.u16(self.replies.len() as u16);
        for (nonce, addr, at, reply) in &self.replies {
            w.rest(nonce).str(&addr.to_string()).u64(age_ms(*at, now)).bytes(reply);
        }
        (w.finish(), fds)
    }

    /// Becomes the binary at the configured path, keeping all sessions; on
    /// any failure this process simply goes on serving.
    fn upgrade(&mut self) {
        let Some((exe, dir)) = self.upgrade.clone() else {
            return;
        };
        eprintln!("rsh: upgrading to {} with {} sessions", exe.display(), self.sessions.len());
        let (state, fds) = self.export();
        let err = crate::upgrade::exec(&exe, HANDOFF_FORMAT, &state, &fds, &dir);
        eprintln!("rsh: upgrade failed, still serving: {err}");
    }

    fn session_by_fd(&self, fd: i32) -> Option<u64> {
        self.sessions
            .values()
            .find(|s| s.pty.as_ref().is_some_and(|pty| pty.fd() == fd))
            .map(|s| s.id)
    }

    fn receive(&mut self, buf: &mut [u8], now: Instant) {
        for _ in 0..256 {
            match self.socket.recv_from(buf) {
                Ok((n, addr)) => {
                    // Nothing a client sends is this large; drop it before
                    // spending any cryptography on it.
                    if !self.drop_incoming && !self.drop_udp && n <= proto::MAX_REQUEST {
                        let datagram = buf[..n].to_vec();
                        self.dispatch(&datagram, addr, now);
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::Interrupted => break,
                Err(_) => break,
            }
        }
        for (packet, addr) in self.tcp.receive(now) {
            if !self.drop_incoming && packet.len() <= proto::MAX_REQUEST {
                self.dispatch(&packet, addr, now);
            }
        }
    }

    fn send(&self, key: &Key, kind: u8, id: u64, payload: &[u8], addr: SocketAddr) -> Vec<u8> {
        let bytes = proto::seal(key, Dir::ToClient, kind, id, payload);
        send_to(&self.socket, &self.tcp, &bytes, addr);
        bytes
    }

    fn dispatch(&mut self, datagram: &[u8], addr: SocketAddr, now: Instant) {
        let Some((kind, id)) = proto::peek(datagram) else {
            return;
        };
        match kind {
            KIND_HELLO | KIND_LIST | KIND_PING => self.dispatch_control(kind, id, datagram, addr, now),
            KIND_CONFIRM => self.handle_confirm(id, datagram, addr, now),
            KIND_STREAM | KIND_CHANNEL => {
                let Some(&session) = self.attach_index.get(&id) else {
                    return;
                };
                let Some(key) = self.sessions.get(&session).and_then(|s| s.attach.as_ref()).map(|a| a.key.clone()) else {
                    return;
                };
                if let Some(packet) = proto::open(&key, Dir::ToServer, datagram) {
                    self.tcp.known(addr, now);
                    if kind == KIND_STREAM {
                        self.handle_stream(session, &packet, addr, now);
                    } else {
                        self.handle_channel(session, &packet, addr, now);
                    }
                }
            }
            _ => {}
        }
    }

    /// HELLO, LIST and PING: sealed with a token key named by the header.
    fn dispatch_control(&mut self, kind: u8, key_id: u64, datagram: &[u8], addr: SocketAddr, now: Instant) {
        let Some(key) = self.keys.get(&key_id).cloned() else {
            return;
        };
        let Some(packet) = proto::open(&key, Dir::ToServer, datagram) else {
            return;
        };
        self.tcp.known(addr, now);
        if let Some((_, from, _, reply)) = self.replies.iter().find(|(nonce, _, _, _)| *nonce == packet.nonce) {
            // A retry is answered again, but only to where the request came
            // from: a replay from a forged address must not be reflected.
            if *from == addr {
                send_to(&self.socket, &self.tcp, reply, addr);
            }
            return;
        }
        let mut reader = proto::Reader::new(&packet.payload);
        let Some(time_ms) = reader.u64() else {
            return;
        };
        let reply = if now_ms().abs_diff(time_ms) > MAX_SKEW_MS {
            self.send(&key, KIND_DENIED, key_id, &proto::encode_denied(&packet.nonce, "clock skew too large"), addr)
        } else {
            match kind {
                KIND_HELLO => self.handle_hello(&key, &packet, addr, now),
                KIND_LIST => {
                    let tmux = reader.u8().is_some_and(|flags| flags & proto::LIST_TMUX != 0);
                    match self.handle_list(&key, &packet, addr, tmux, now) {
                        Some(reply) => reply,
                        // Answered once tmux has; a retry meanwhile joins the wait.
                        None => return,
                    }
                }
                _ => self.handle_ping(&key, &packet, addr),
            }
        };
        self.remember_reply(packet.nonce, addr, now, reply);
    }

    fn remember_reply(&mut self, nonce: [u8; 24], addr: SocketAddr, now: Instant, reply: Vec<u8>) {
        self.replies.push_back((nonce, addr, now, reply));
        while self.replies.len() > 1024
            || self
                .replies
                .front()
                .is_some_and(|(_, _, at, _)| now.duration_since(*at) > Duration::from_millis(2 * MAX_SKEW_MS))
        {
            self.replies.pop_front();
        }
    }

    /// Starts a handshake: the server's X25519 half and a fresh attachment
    /// id. Nothing else happens until CONFIRM proves the client's half.
    fn handle_hello(&mut self, key: &Key, packet: &Packet, addr: SocketAddr, now: Instant) -> Vec<u8> {
        let denied = |state: &Self, reason: &str| {
            state.send(key, KIND_DENIED, key.id(), &proto::encode_denied(&packet.nonce, reason), addr)
        };
        let Some(hello) = Hello::decode(&packet.payload) else {
            return denied(self, "bad hello");
        };
        let (secret, server_pub) = crate::crypto::ephemeral();
        let Some(shared) = crate::crypto::shared(&secret, &hello.client_pub) else {
            return denied(self, "bad key exchange");
        };
        let attach_key = key.attach(&shared, &hello.client_pub, &server_pub);
        let mut attach = random_u64() | 1;
        while self.attach_index.contains_key(&attach) || self.pending.contains_key(&attach) {
            attach = random_u64() | 1;
        }
        self.pending.retain(|_, p| p.expires > now);
        while self.pending.len() >= MAX_PENDING {
            let oldest = self.pending.iter().min_by_key(|(_, p)| p.expires).map(|(id, _)| *id);
            if let Some(oldest) = oldest {
                self.pending.remove(&oldest);
            }
        }
        self.pending.insert(
            attach,
            Pending {
                key: attach_key,
                token: key.id(),
                hello,
                expires: now + PENDING_LIFETIME,
                done: None,
            },
        );
        let ready = Ready {
            echo: packet.nonce,
            attach,
            server_pub,
            server: crate::version::describe(),
        };
        self.send(key, KIND_READY, key.id(), &ready.encode(), addr)
    }

    /// The client proved it holds the key agreed in HELLO/READY: only now
    /// is a session started, attached or ended.
    fn handle_confirm(&mut self, attach: u64, datagram: &[u8], addr: SocketAddr, now: Instant) {
        let Some(pending) = self.pending.get(&attach) else {
            return;
        };
        let Some(packet) = proto::open(&pending.key, Dir::ToServer, datagram) else {
            return;
        };
        if let Some((to, reply)) = &pending.done {
            // Only to where it went: a replay must not be reflected.
            if *to == addr {
                send_to(&self.socket, &self.tcp, reply, addr);
            }
            return;
        }
        let (key, token) = (pending.key.clone(), pending.token);
        let hello = pending.hello.clone();
        let outcome = self.carry_out(attach, &key, token, &hello, &packet, addr, now);
        let reply = self.send(&key, KIND_ATTACHED, attach, &outcome.encode(), addr);
        if let Some(pending) = self.pending.get_mut(&attach) {
            pending.done = Some((addr, reply));
        }
        if outcome.session != 0 {
            self.pump(outcome.session, now);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn carry_out(&mut self, attach_id: u64, key: &Key, token: u64, hello: &Hello, packet: &Packet, addr: SocketAddr, now: Instant) -> Attached {
        let refuse = |reason: String| Attached {
            session: 0,
            label: String::new(),
            created: false,
            notes: Vec::new(),
            killed: Vec::new(),
            denied: reason,
        };
        let live = |s: &Session| s.host == hello.host && s.ended_at.is_none();
        if let Action::Kill(target) = &hello.action {
            let mut killed = Vec::new();
            for session in self.sessions.values() {
                let matches = session.id.to_string() == *target || session.label == *target;
                if matches && session.host == hello.host {
                    if let Some(pty) = &session.pty {
                        pty.hangup();
                        self.zombies.push((session.pid, now));
                    }
                    killed.push(session.id);
                }
            }
            let mut done = refuse(String::new());
            done.killed = killed;
            return done;
        }
        let cols = hello.cols.clamp(1, MAX_COLS);
        let rows = hello.rows.clamp(1, MAX_ROWS);
        let command = hello.command.join(" ");
        // A session joined to a tmux session is a tmux session's way in,
        // not a shell: only the same tmux name reaches it again.
        let shell = |s: &Session| live(s) && !crate::tmux::is_wrapper(&s.command);
        let existing = if crate::tmux::is_wrapper(&command) {
            // One rsh session per tmux session; joining again takes it over.
            self.sessions.values().find(|s| live(s) && s.pty.is_some() && s.command == command).map(|s| s.id)
        } else if !hello.command.is_empty() {
            None
        } else {
            match &hello.action {
                Action::Default => self.sessions.values().filter(|s| shell(s)).min_by_key(|s| s.created).map(|s| s.id),
                Action::Named(name) => self.sessions.values().find(|s| shell(s) && &s.label == name).map(|s| s.id),
                Action::Attach(id) => match self.sessions.get(id) {
                    Some(s) if live(s) => Some(*id),
                    _ => return refuse("unknown session".into()),
                },
                Action::Kill(_) => None,
            }
        };
        let (id, created) = match existing {
            Some(id) => (id, false),
            None => {
                let label = match &hello.action {
                    Action::Named(name) => sanitize_name(name),
                    _ if !hello.command.is_empty() => sanitize_name(&hello.command.join(" ")),
                    _ => None,
                }
                .unwrap_or_else(|| proto::free_name(|name| self.sessions.values().any(|s| live(s) && s.label == name)));
                match self.create(hello, &label, cols, rows, now) {
                    Ok(id) => (id, true),
                    Err(err) => return refuse(format!("cannot start shell: {err}")),
                }
            }
        };
        let session = self.sessions.get_mut(&id).expect("session");
        if let Some(old) = session.attach.take() {
            self.attach_index.remove(&old.id);
            let bytes = proto::seal(&old.key, Dir::ToClient, KIND_KICKED, old.id, &proto::Writer::new().str("attached from another client").finish());
            send_to(&self.socket, &self.tcp, &bytes, old.addr);
        }
        session.resize(cols, rows);
        let mut output = Sender::new(0);
        output.set_lossless(is_tcp_addr(addr));
        let first = session.screen(ATTACH_HISTORY);
        let (forward, notes) = listen_remote(&hello.forwards);
        session.attach = Some(Attachment {
            id: attach_id,
            key: key.clone(),
            token,
            addr,
            last_heard: now,
            heard_at_addr: now,
            candidate: None,
            window: ReplayWindow::pinned(&packet.nonce),
            input: Receiver::new(0),
            output,
            reply_due: false,
            ack_at: None,
            repaint_due: false,
            syncing_until: None,
            screen_at: None,
            mid_sequence: None,
            shadow: None,
            deflate: hello.features & proto::FEATURE_DEFLATE != 0,
            screens: hello.features & proto::FEATURE_SCREENS != 0,
            zstream: hello.features & proto::FEATURE_ZSTREAM != 0,
            compress: hello.features & proto::FEATURE_COMPRESS != 0,
            z: None,
            zraw: Vec::new(),
            update_len: 0,
            updates: 0,
            switched: None,
            output_at: None,
            output_gap: Duration::ZERO,
            frames: 0,
            step_bytes: 0,
            made: (now, 0, 0),
            ratio: 4.0,
            doubling: true,
            end_offset: None,
            forward,
            forwards_closed: false,
            remote_forwards: hello.forwards.clone(),
        });
        session.attach.as_mut().expect("attachment").reset_output(&first);
        session.last_client = now;
        self.attach_index.insert(attach_id, id);
        Attached {
            session: id,
            label: session.label.clone(),
            created,
            notes,
            killed: Vec::new(),
            denied: String::new(),
        }
    }

    fn create(&mut self, hello: &Hello, label: &str, cols: u16, rows: u16, now: Instant) -> io::Result<u64> {
        // Short enough to type after `rsh kill`; packets are authenticated
        // by their keys, so the id need not be secret.
        let mut id = 100_000 + random_u64() % 900_000;
        while self.sessions.contains_key(&id) {
            id = 100_000 + random_u64() % 900_000;
        }
        let pty = Pty::spawn(&SpawnOptions {
            session_id: id,
            cols,
            rows,
            env: &hello.env,
            command: &hello.command,
        })?;
        let pid = pty.pid();
        self.sessions.insert(
            id,
            Session {
                id,
                host: hello.host.clone(),
                label: label.to_string(),
                command: hello.command.join(" "),
                created: SystemTime::now(),
                pty: Some(pty),
                pid,
                closed_at: None,
                exit_code: None,
                ended_at: None,
                term: Terminal::with_size(cols, rows),
                tail: Vec::new(),
                pty_in: VecDeque::new(),
                attach: None,
                last_active: now,
                last_client: now,
            },
        );
        Ok(id)
    }

    /// Answers a LIST now, or None when it asked for tmux sessions and they
    /// are being looked up (`answer_lists` sends it then).
    fn handle_list(&mut self, key: &Key, packet: &Packet, addr: SocketAddr, tmux: bool, now: Instant) -> Option<Vec<u8>> {
        if tmux && !self.tmux.known.as_ref().is_some_and(|(at, _)| now.duration_since(*at) < TMUX_FRESH) {
            if !self.tmux.waiting.iter().any(|(_, nonce, _)| *nonce == packet.nonce) {
                self.tmux.waiting.push((key.clone(), packet.nonce, addr));
            }
            if self.tmux.asking.is_none() {
                let (send, receive) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let _ = send.send(crate::tmux::list());
                });
                self.tmux.asking = Some((now, receive));
            }
            return None;
        }
        Some(self.list_reply(key, &packet.nonce, addr, tmux))
    }

    /// Sends the LISTs that waited for tmux once it answered or took too long.
    fn answer_lists(&mut self, now: Instant) {
        let Some((asked, receive)) = &self.tmux.asking else {
            return;
        };
        match receive.try_recv() {
            Ok(found) => self.tmux.known = Some((now, found)),
            Err(std::sync::mpsc::TryRecvError::Empty) if now.duration_since(*asked) < TMUX_WAIT => return,
            // Too slow (a first lookup through heavy rc files): these LISTs
            // get what is known, and the next asks again.
            Err(_) => {}
        }
        self.tmux.asking = None;
        for (key, nonce, addr) in std::mem::take(&mut self.tmux.waiting) {
            let reply = self.list_reply(&key, &nonce, addr, true);
            self.remember_reply(nonce, addr, now, reply);
        }
    }

    fn list_reply(&self, key: &Key, nonce: &[u8; 24], addr: SocketAddr, tmux: bool) -> Vec<u8> {
        let now = Instant::now();
        let mut sessions: Vec<&Session> = self.sessions.values().collect();
        sessions.sort_by_key(|s| (s.created, s.id));
        let infos: Vec<SessionInfo> = sessions
            .into_iter()
            .take(60)
            .map(|s| SessionInfo {
                id: s.id,
                host: s.host.clone(),
                label: s.label.clone(),
                alive: s.ended_at.is_none(),
                attached: s
                    .attach
                    .as_ref()
                    .is_some_and(|a| now.duration_since(a.last_heard) < Duration::from_secs(5)),
                cols: s.term.size().0,
                rows: s.term.size().1,
                created_unix: s.created.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
                idle_secs: now.duration_since(s.last_active).as_secs(),
                command: s.command.clone(),
                title: s.term.title().chars().take(80).collect(),
            })
            .collect();
        let unix = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let found = self.tmux.known.as_ref().filter(|_| tmux).map(|(_, found)| found.as_slice()).unwrap_or_default();
        let tmux: Vec<TmuxInfo> = found
            .iter()
            .take(60)
            .map(|t| TmuxInfo {
                name: t.name.chars().take(64).collect(),
                clients: t.clients,
                cols: t.cols,
                rows: t.rows,
                idle_secs: unix.saturating_sub(t.activity),
                // The rsh session joined to it: its label is the tmux name.
                session: self
                    .sessions
                    .values()
                    .find(|s| s.ended_at.is_none() && s.pty.is_some() && crate::tmux::is_wrapper(&s.command) && s.label == t.name)
                    .map_or(0, |s| s.id),
            })
            .collect();
        self.send(key, KIND_LIST_REPLY, key.id(), &encode_list(nonce, &infos, &crate::version::describe(), &tmux), addr)
    }

    /// Tells a client whose server went quiet whether its attachment still
    /// exists; after a restart the server cannot reach it any other way.
    fn handle_ping(&mut self, key: &Key, packet: &Packet, addr: SocketAddr) -> Vec<u8> {
        let mut reader = proto::Reader::new(&packet.payload);
        let _ = reader.u64();
        let alive = reader.u64().is_some_and(|attach| self.attach_index.contains_key(&attach));
        let reply = proto::Writer::new().rest(&packet.nonce).u8(u8::from(alive)).finish();
        self.send(key, KIND_PONG, key.id(), &reply, addr)
    }

    /// Bookkeeping common to every packet of an attachment: replay window,
    /// keepalive, and moving to a new client address.
    fn heard(attach: &mut Attachment, packet: &Packet, addr: SocketAddr, tcp: &TcpSide, now: Instant) -> Option<bool> {
        let newest = packet.counter > attach.window.top();
        if !attach.window.check(&packet.nonce) {
            return None;
        }
        attach.last_heard = now;
        if addr == attach.addr {
            attach.heard_at_addr = now;
            attach.candidate = None;
        } else if newest {
            // A new address is taken once two newest packets came from it,
            // or once the old one has gone quiet or its TCP connection
            // closed: one packet replayed from a forged address cannot
            // divert the stream on its own.
            let count = match attach.candidate {
                Some((candidate, count)) if candidate == addr => count + 1,
                _ => 1,
            };
            let closed = is_tcp_addr(attach.addr) && !tcp.has(attach.addr);
            if count >= 2 || closed || now.duration_since(attach.heard_at_addr) >= Duration::from_secs(2) {
                // What went into a TCP connection that is gone goes again;
                // over TCP nothing else would resend it.
                if is_tcp_addr(attach.addr) || is_tcp_addr(addr) {
                    attach.output.rewind();
                }
                attach.output.set_lossless(is_tcp_addr(addr));
                attach.output.forget_capacity();
                attach.addr = addr;
                attach.heard_at_addr = now;
                attach.candidate = None;
            } else {
                attach.candidate = Some((addr, count));
            }
        }
        if attach.forwards_closed {
            let (forward, _) = listen_remote(&attach.remote_forwards);
            let next = attach.forward.next_id();
            attach.forward = forward;
            attach.forward.set_next_id(next);
            attach.forwards_closed = false;
        }
        Some(newest)
    }

    fn handle_stream(&mut self, id: u64, packet: &Packet, addr: SocketAddr, now: Instant) {
        let Some(stream) = ClientStream::decode(&packet.payload) else {
            return;
        };
        let Some(session) = self.sessions.get_mut(&id) else {
            return;
        };
        let attach = session.attach.as_mut().expect("attachment");
        let Some(newest) = Self::heard(attach, packet, addr, &self.tcp, now) else {
            return;
        };
        session.last_client = now;
        if stream.flags & FLAG_DETACH != 0 {
            if let Some(old) = session.attach.take() {
                self.attach_index.remove(&old.id);
            }
            if session.ended_at.is_some() {
                self.sessions.remove(&id);
            }
            return;
        }
        // A reordered packet carries an older window size; only the newest counts.
        if newest && stream.cols > 0 && stream.rows > 0 {
            session.resize(stream.cols.min(MAX_COLS), stream.rows.min(MAX_ROWS));
        }
        let attach = session.attach.as_mut().expect("attachment");
        let advanced = attach.output.on_ack(stream.out_gen, stream.out_ack, now, !stream.data.is_empty() || stream.flags & proto::FLAG_REPEAT != 0);
        if !stream.data.is_empty() && stream.in_offset > attach.input.next {
            // Past a gap: acked at once, so duplicate acks reach the client.
            attach.reply_due = true;
        } else if !stream.data.is_empty() {
            attach.ack_at.get_or_insert(now + INPUT_ACK_DELAY);
        } else if stream.flags & !proto::FLAG_REPEAT != 0 || (!advanced && attach.output.deadline().is_none()) {
            // A heartbeat with nothing in flight: the answer is how the
            // client knows the server is there. Acks need no answer.
            attach.reply_due = true;
        }
        let room = INPUT_BACKLOG.saturating_sub(session.pty_in.len());
        let data = attach.input.accept(0, stream.in_offset, stream.data, room);
        let live = session.pty.is_some();
        let interrupt = live && data.contains(&0x03) && attach.output.pending() > INTERRUPT_FLUSH;
        let asked_gen = |gen: u32| gen == attach.output.gen || attach.switched == Some(attach.output.gen) && gen == attach.output.gen.wrapping_sub(1);
        let redraw = stream.flags & FLAG_REDRAW != 0 && asked_gen(stream.out_gen);
        if !data.is_empty() && live {
            session.pty_in.extend(data);
            session.last_active = now;
        }
        if interrupt || redraw {
            let screen = session.screen(0);
            let attach = session.attach.as_mut().expect("attachment");
            attach.reset_output(&screen);
            attach.repaint_due = false;
            // The client now has this repaint, not the shadow's screen.
            attach.shadow = None;
            attach.syncing_until = None;
        }
        self.flush_input(id);
        self.pump(id, now);
    }

    fn handle_channel(&mut self, id: u64, packet: &Packet, addr: SocketAddr, now: Instant) {
        let Some(msg) = forward::Message::decode(&packet.payload) else {
            return;
        };
        let Some(attach) = self.sessions.get_mut(&id).and_then(|s| s.attach.as_mut()) else {
            return;
        };
        if Self::heard(attach, packet, addr, &self.tcp, now).is_none() {
            return;
        }
        attach.forward.receive(&msg, now, &|_| true);
        self.send_channels(id, now);
    }

    /// Moves forwarded connections along for every attached client.
    fn forwards(&mut self, fds: &[libc::pollfd], now: Instant) {
        let ids: Vec<u64> = self
            .sessions
            .values()
            .filter(|s| s.attach.as_ref().is_some_and(|a| !a.forward.is_empty()))
            .map(|s| s.id)
            .collect();
        for id in ids {
            if let Some(attach) = self.sessions.get_mut(&id).and_then(|s| s.attach.as_mut()) {
                attach.forward.mark_ready(fds);
                attach.forward.service(now);
            }
            self.send_channels(id, now);
        }
    }

    fn send_channels(&mut self, id: u64, now: Instant) {
        let (socket, tcp) = (&self.socket, &self.tcp);
        let Some(session) = self.sessions.get_mut(&id) else {
            return;
        };
        let Some(attach) = session.attach.as_mut() else {
            return;
        };
        if now.duration_since(attach.last_heard) > ASSOCIATION_TIMEOUT {
            return;
        }
        for payload in attach.forward.outgoing(now) {
            let bytes = proto::seal(&attach.key, Dir::ToClient, KIND_CHANNEL, attach.id, &payload);
            send_to(socket, tcp, &bytes, attach.addr);
        }
    }

    fn flush_input(&mut self, id: u64) {
        let Some(session) = self.sessions.get_mut(&id) else {
            return;
        };
        let Some(pty) = &session.pty else {
            session.pty_in.clear();
            return;
        };
        while !session.pty_in.is_empty() {
            let (front, _) = session.pty_in.as_slices();
            match pty.write(front) {
                Ok(0) => break,
                Ok(n) => {
                    session.pty_in.drain(..n);
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    }

    fn read_pty(&mut self, id: u64, buf: &mut [u8], now: Instant) {
        let Some(session) = self.sessions.get_mut(&id) else {
            return;
        };
        let mut total = 0;
        let mut closed = false;
        while total < READ_BUDGET {
            let Some(pty) = &session.pty else {
                break;
            };
            match pty.read(buf) {
                Ok(0) => {
                    closed = true;
                    break;
                }
                Ok(n) => {
                    total += n;
                    session.output(&buf[..n]);
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(_) => {
                    closed = true;
                    break;
                }
            }
        }
        if total > 0 {
            session.last_active = now;
        }
        // A burst too big for the path (a `cat` over a WAN) goes as screens
        // from the start: none of it has been sent yet, so no queue of it
        // forms ahead of the screen that ends it.
        if total >= BURST && session.attach.as_ref().is_some_and(|a| {
            a.screens && a.syncing_until.is_none() && a.output.first_acked() && a.output.srtt().is_some_and(|srtt| srtt >= DEFLATE_RTT)
        }) {
            let attach = session.attach.as_mut().expect("attachment");
            attach.syncing_until = Some(now + SYNC_SPAN);
            session.refresh(now);
        }
        if closed {
            session.pty = None;
            session.pty_in.clear();
            session.closed_at = Some(now);
        }
        self.pump(id, now);
    }

    /// Sends what the window allows, plus an ack if the client is owed one.
    fn pump(&mut self, id: u64, now: Instant) {
        let (socket, tcp) = (&self.socket, &self.tcp);
        let Some(session) = self.sessions.get_mut(&id) else {
            return;
        };
        // At the end the last update goes at once: the client stops at the
        // stream's end and would miss one still waiting for its turn.
        let ending = session.exit_code.is_some();
        let due = |a: &Attachment| a.repaint_due && if a.shadow.is_some() { ending || a.next_screen_due(now) } else { a.output.first_acked() };
        if session.attach.as_ref().is_some_and(due) {
            session.refresh(now);
        }
        let exit_code = session.exit_code;
        let Some(attach) = session.attach.as_mut() else {
            return;
        };
        if now.duration_since(attach.last_heard) > ASSOCIATION_TIMEOUT {
            return;
        }
        // While the stream still has unsent bytes, output gathers: one flush
        // for all of it compresses better and costs fewer datagrams.
        if attach.output.all_sent() || exit_code.is_some() {
            attach.stage(exit_code.is_some());
        }
        if exit_code.is_some() {
            // No output follows the exit, so the end only moves when a
            // repaint starts a new generation; it must follow that.
            attach.end_offset = Some(attach.output.end());
        }
        let mut sent = false;
        let mut batch = Vec::new();
        loop {
            // A generation that is one deflate stream needs no more of it.
            let squeeze = attach.deflate && attach.z.is_none() && (attach.syncing_until.is_some() || attach.output.srtt().is_some_and(|srtt| srtt >= DEFLATE_RTT));
            let chunk = attach.output.next_chunk_max(now, if squeeze { proto::SQUEEZED } else { proto::CHUNK });
            let ack_due = attach.reply_due || attach.ack_at.is_some_and(|at| now >= at);
            if chunk.is_none() && attach.output.all_sent() && attach.stage(false) > 0 {
                // What gathered while the stream was busy goes now.
                continue;
            }
            if chunk.is_none() && (sent || !ack_due) {
                break;
            }
            let (offset, mut data) = chunk.unwrap_or((attach.output.end(), Vec::new()));
            let mut flags = if exit_code.is_some() { FLAG_EXITED } else { 0 };
            if attach.z.is_some() {
                flags |= proto::FLAG_ZSTREAM;
            }
            if attach.syncing_until.is_some() {
                flags |= proto::FLAG_UPDATES;
            }
            if squeeze && data.len() > 64 {
                // As much as compresses into one datagram: first what the
                // recent ratio says fits, halving until it does; at worst a
                // plain chunk.
                let taken = data.len();
                let guess = (proto::CHUNK as f64 * attach.ratio * 0.9) as usize;
                let mut keep = taken.min(guess.max(proto::CHUNK));
                loop {
                    let packed = proto::deflate(&data[..keep]);
                    if packed.len() <= proto::CHUNK && packed.len() < keep {
                        attach.ratio = (keep as f64 / packed.len() as f64).clamp(1.0, 16.0);
                        attach.output.take_back(offset, taken, keep);
                        data = packed;
                        flags |= proto::FLAG_DEFLATE;
                        break;
                    }
                    if keep <= proto::CHUNK {
                        attach.output.take_back(offset, taken, keep);
                        data.truncate(keep);
                        break;
                    }
                    keep = (keep / 2).max(proto::CHUNK);
                }
            }
            let stream = ServerStream {
                flags,
                in_ack: attach.input.next,
                out_gen: attach.output.gen,
                out_offset: offset,
                exit_code: exit_code.unwrap_or(0),
                end_offset: attach.end_offset.unwrap_or_else(|| attach.output.end()),
                data: &data,
            };
            let sealed = proto::seal(&attach.key, Dir::ToClient, KIND_STREAM, attach.id, &stream.encode());
            // On a lossy path with room to spare, data goes twice (each copy
            // sealed apart, as the replay window wants unique nonces).
            // On at the first resend: a loss seen is likely the path's.
            // TCP loses nothing.
            let loss = attach.output.loss();
            let resent = attach.output.take_resend();
            // A slow link has no room for copies.
            attach.doubling = !is_tcp_addr(attach.addr)
                && attach.output.slow_rate().is_none()
                && (resent || loss > DOUBLE_ON || (attach.doubling && loss > DOUBLE_OFF));
            if !stream.data.is_empty() && attach.doubling && attach.output.in_flight() < attach.output.window() / 2 {
                batch.push(proto::seal(&attach.key, Dir::ToClient, KIND_STREAM, attach.id, &stream.encode()));
            }
            batch.push(sealed);
            sent = true;
            attach.reply_due = false;
            attach.ack_at = None;
        }
        if is_tcp_addr(attach.addr) {
            for packet in &batch {
                tcp.send(packet, attach.addr);
            }
        } else {
            crate::net::send_all(socket, attach.addr, &batch);
        }
    }

    fn timers(&mut self, now: Instant) {
        let ids: Vec<u64> = self.sessions.keys().copied().collect();
        for id in ids {
            let Some(session) = self.sessions.get_mut(&id) else {
                continue;
            };
            if session.closed_at.is_some() && session.exit_code.is_none() {
                session.exit_code = try_wait(session.pid);
                let waited = session.closed_at.map_or(Duration::ZERO, |at| now.duration_since(at));
                if session.exit_code.is_none() && waited > Duration::from_secs(2) {
                    kill_group(session.pid);
                    self.zombies.push((session.pid, now));
                    session.exit_code = Some(-1);
                }
                if session.exit_code.is_some() {
                    session.ended_at = Some(now);
                    if let Some(attach) = session.attach.as_mut() {
                        attach.reply_due = true;
                    }
                }
            }
            if let Some(ended) = session.ended_at {
                if session.attach.is_none() || now.duration_since(ended) > ENDED_LINGER {
                    if let Some(attach) = self.sessions.remove(&id).and_then(|s| s.attach) {
                        self.attach_index.remove(&attach.id);
                    }
                    continue;
                }
            }
            if let Some(attach) = session.attach.as_mut() {
                // Nobody to forward to while the client is away; reopened
                // when it is heard again.
                if !attach.forwards_closed && now.duration_since(attach.last_heard) > ASSOCIATION_TIMEOUT {
                    attach.forward.close_all();
                    attach.forwards_closed = true;
                }
            }
            let mut timed_out = false;
            if let (Some(limit), Some(session)) = (self.disconnect_timeout, self.sessions.get(&id)) {
                timed_out = session.pty.is_some() && now.duration_since(session.last_client) > limit;
            }
            if timed_out {
                if let Some(session) = self.sessions.get_mut(&id) {
                    if let Some(pty) = &session.pty {
                        pty.hangup();
                        session.last_client = now;
                        self.zombies.push((session.pid, now));
                    }
                }
            }
            if let Some(attach) = self.sessions.get(&id).and_then(|s| s.attach.as_ref()) {
                let ack_due = attach.ack_at.is_some_and(|at| now >= at);
                if attach.output.deadline().is_some_and(|deadline| now >= deadline) || attach.reply_due || ack_due || attach.next_screen_due(now) {
                    self.pump(id, now);
                }
            }
        }
        self.answer_lists(now);
        self.pending.retain(|_, pending| pending.expires > now);
        self.zombies.retain(|(pid, since)| {
            if try_wait(*pid).is_some() {
                return false;
            }
            if now.duration_since(*since) > Duration::from_secs(2) {
                kill_group(*pid);
            }
            now.duration_since(*since) < Duration::from_secs(30)
        });
        if now.duration_since(self.auth_checked) > Duration::from_secs(2) {
            self.auth_checked = now;
            self.reload_auth();
        }
    }

    /// Takes the authorized file's current tokens. A file that is gone or
    /// that others could write grants nothing (fail closed), and a token no
    /// longer there ends its pending handshakes and attachments.
    fn reload_auth(&mut self) {
        let Some(path) = self.auth_path.clone() else {
            return;
        };
        let stamp = modified(&path);
        if stamp == self.auth_stamp {
            return;
        }
        self.auth_stamp = stamp;
        self.keys = match AuthStore::load(&path) {
            Ok(store) => keys_of(&store),
            Err(err) => {
                eprintln!("rsh: {}: {err}; no token is accepted until it is fixed", path.display());
                BTreeMap::new()
            }
        };
        self.pending.retain(|_, p| self.keys.contains_key(&p.token));
        for session in self.sessions.values_mut() {
            if session.attach.as_ref().is_some_and(|a| !self.keys.contains_key(&a.token)) {
                let old = session.attach.take().expect("attachment");
                self.attach_index.remove(&old.id);
                let bytes = proto::seal(&old.key, Dir::ToClient, KIND_KICKED, old.id, &proto::Writer::new().str("this client's token was revoked").finish());
                send_to(&self.socket, &self.tcp, &bytes, old.addr);
            }
        }
    }
}

impl Attachment {
    /// In the screens-only mode, when the next update may go: once the last
    /// one is out (though not yet acked); at once while the path shows no
    /// queue, else half a round trip after the last (as mosh paces every
    /// frame). A queue raises the round trip and with it the interval.
    fn next_screen_at(&self) -> Option<Instant> {
        let at = self.screen_due_at()?;
        // Not in the middle of an escape sequence (`TAIL_WAIT`).
        let held = self.mid_sequence.filter(|_| self.shadow.is_some()).map(|since| since + TAIL_WAIT);
        Some(held.map_or(at, |held| at.max(held)))
    }

    /// When the next update would go, the program's output aside.
    fn screen_due_at(&self) -> Option<Instant> {
        if !self.repaint_due || self.shadow.is_none() || !self.output.all_sent() {
            return None;
        }
        let last = self.screen_at?;
        if let Some(bdp) = self.output.slow_bdp() {
            // A slow link: the next update waits until it fits what the
            // path delivers in its least round trip plus two updates, so
            // updates do not queue on the link and age there (with one,
            // an update now and then waited a round trip).
            let in_flight = self.output.in_flight_wire();
            let update = self.update_len as u64 + proto::WIRE_OVERHEAD;
            let budget = bdp + 2 * update;
            if in_flight > 0 && in_flight + update > budget {
                return None;
            }
            // Then once the last update is off the link at its rate. Output
            // that came while it was busy waits a little for newer output:
            // an update of the newest screen as it comes shows it younger
            // than one of an older screen at once.
            let rate = self.output.slow_rate().expect("slow");
            let gap = Duration::from_secs_f64((self.update_len as u64 + proto::WIRE_OVERHEAD) as f64 / rate) + SLOW_ROOM;
            // When the program writes in steps (frames), output that came
            // while the link was busy waits for the next step if that is
            // due soon after; output that stopped goes at once.
            let idle = last + gap.max(self.screen_gap());
            return Some(match self.output_at {
                Some(at) if at >= idle => at,
                Some(at) => {
                    let next = at + self.output_gap;
                    if next > idle && next <= idle + FRESH_WAIT { next } else { idle }
                }
                None => idle,
            });
        }
        if self.output.queueing() < LAG_LIMIT / 2 {
            // Not faster than a frame every 20 ms (or the program's step):
            // a burst of output (a `cat`) would otherwise queue an update
            // per read.
            return Some(last + self.screen_gap());
        }
        let half = self.output.srtt().map_or(Duration::from_millis(50), |srtt| srtt / 2);
        Some(last + half.clamp(self.screen_gap(), Duration::from_millis(250)))
    }

    /// The least time between updates: `SCREEN_GAP`, or the program's own
    /// step when it draws faster (to `MIN_SCREEN_GAP`).
    fn screen_gap(&self) -> Duration {
        if self.animating() {
            self.output_gap.clamp(MIN_SCREEN_GAP, SCREEN_GAP)
        } else {
            SCREEN_GAP
        }
    }

    /// The program draws frames (see `ANIMATION_FRAMES`).
    fn animating(&self) -> bool {
        self.frames >= ANIMATION_FRAMES
    }

    fn next_screen_due(&self, now: Instant) -> bool {
        self.next_screen_at().is_some_and(|at| now >= at)
    }

    /// Whether output should go as one deflate stream: the client can take
    /// it, and asked for it, or the link is far, slow, or behind.
    fn wants_z(&self) -> bool {
        self.zstream
            && (self.compress
                || self.syncing_until.is_some()
                || self.output.slow_rate().is_some()
                || self.output.srtt().is_some_and(|srtt| srtt >= DEFLATE_RTT))
    }

    /// Output for the stream: straight in, or waiting for the compressor
    /// (`stage`).
    fn push_output(&mut self, bytes: &[u8]) -> bool {
        // Behind what already waits, whatever is wanted now: never ahead of it.
        if self.z.is_some() || !self.zraw.is_empty() || self.wants_z() {
            self.zraw.extend_from_slice(bytes);
            false
        } else {
            self.output.push(bytes);
            true
        }
    }

    /// Bytes `bytes` are expected to take in the stream.
    fn stream_len(&self, bytes: usize) -> usize {
        if self.z.is_some() || self.wants_z() {
            (bytes as f64 / self.ratio) as usize
        } else {
            bytes
        }
    }

    /// Puts what waits into the stream: compressed in one flush, or, to
    /// start compressing, in a new generation once the plain one before is
    /// all acked (the client goes on from it, with no repaint). Returns the
    /// bytes it added.
    fn stage(&mut self, ending: bool) -> usize {
        if self.zraw.is_empty() {
            return 0;
        }
        if self.z.is_none() {
            let drained = self.output.acked() == self.output.end();
            if !self.wants_z() || ending {
                // At the end no switch is waited for: the client stops at it.
                let raw = std::mem::take(&mut self.zraw);
                self.output.push(&raw);
                return raw.len();
            }
            if !drained {
                return 0;
            }
            self.z = Some(proto::Deflater::default());
            self.output.reset(&[]);
            // A new generation that is no repaint: a repaint asked for the
            // one before still stands (the client asked with its number).
            self.switched = Some(self.output.gen);
        }
        let raw = std::mem::take(&mut self.zraw);
        let packed = self.z.as_mut().expect("compressor").push(&raw);
        self.ratio = (raw.len() as f64 / packed.len().max(1) as f64).clamp(1.0, 64.0);
        self.output.push(&packed);
        packed.len()
    }

    /// A new generation whose stream starts with `bytes` (a repaint),
    /// compressed when that is wanted.
    fn reset_output(&mut self, bytes: &[u8]) {
        self.zraw.clear();
        if self.wants_z() {
            let mut z = proto::Deflater::default();
            self.output.reset(&z.push(bytes));
            self.z = Some(z);
        } else {
            self.z = None;
            self.output.reset(bytes);
        }
    }
}

impl Session {
    fn resize(&mut self, cols: u16, rows: u16) {
        if self.term.size() == (cols, rows) {
            return;
        }
        self.term.resize(cols, rows);
        if let Some(pty) = &self.pty {
            pty.resize(cols, rows);
        }
    }

    /// The screen as bytes for a client: a repaint, then any unfinished
    /// escape sequence, so the terminal ends up mid-sequence like the program.
    fn screen(&self, history: usize) -> Vec<u8> {
        let mut bytes = self.term.render(history);
        bytes.extend_from_slice(&self.tail);
        bytes
    }

    /// Brings the client up to the screen in the screens-only mode: the
    /// difference from what it has, or a repaint when that will not do
    /// (or the difference does not reproduce the screen).
    fn refresh(&mut self, now: Instant) {
        let at_ground = self.tail.is_empty();
        let Some(attach) = self.attach.as_mut() else {
            return;
        };
        if let Some(shadow) = attach.shadow.as_mut().filter(|_| at_ground) {
            if let Some(bytes) = self.term.diff_from(shadow) {
                shadow.feed(&bytes);
                if self.term.same_screen(shadow) {
                    let direct = attach.push_output(&bytes);
                    let staged = attach.stage(false);
                    attach.update_len = if direct { bytes.len() } else if staged > 0 { staged } else { attach.stream_len(bytes.len()) };
                    attach.updates = attach.updates.wrapping_add(1);
                    attach.repaint_due = false;
                    attach.screen_at = Some(now);
                    return;
                }
            }
        }
        let screen = self.screen(0);
        let (cols, rows) = self.term.size();
        let attach = self.attach.as_mut().expect("attachment");
        attach.reset_output(&screen);
        attach.update_len = attach.output.pending();
        attach.updates = attach.updates.wrapping_add(1);
        if attach.syncing_until.is_some() {
            // Updates are paced by the path's queue (`next_screen_at`); a
            // round trip's wait for the first ack would only delay them.
            attach.output.forget_carried();
        }
        attach.repaint_due = false;
        attach.screen_at = Some(now);
        attach.shadow = attach.syncing_until.map(|_| {
            let mut shadow = Terminal::with_history(cols, rows, 0);
            shadow.feed(&screen);
            shadow
        });
    }

    fn output(&mut self, bytes: &[u8]) {
        let was_ground = self.tail.is_empty();
        if let Some(attach) = self.attach.as_mut().filter(|a| a.screens && was_ground) {
            // On a slow link, output that would take long to drain starts
            // the screens-only mode at once, from the screen the client
            // will have: all before was sent, so no repaint is needed. Before
            // the link's rate is known, so does a program drawing in steps
            // (an animation) faster than the ack clock lets through: within
            // a frame or two of its start, not after a queue of it formed.
            let now = Instant::now();
            let rate = attach.output.slow_rate().or_else(|| attach.output.drain_rate(now).filter(|_| attach.animating()));
            if let Some(rate) = rate.filter(|_| {
                attach.syncing_until.is_none() && !attach.repaint_due && attach.output.all_sent() && attach.zraw.is_empty()
            }) {
                let incoming = attach.stream_len(bytes.len()) as f64;
                // Half the limit: the first frames of an animation would
                // otherwise queue before the mode starts.
                if incoming > 128.0 && (attach.output.in_flight() as f64 + incoming) / rate > LAG_LIMIT.as_secs_f64() / 2.0 {
                    let (cols, rows) = self.term.size();
                    let mut shadow = Terminal::with_history(cols, rows, 0);
                    shadow.feed(&self.term.render(0));
                    attach.shadow = Some(shadow);
                    attach.syncing_until = Some(now + SYNC_SPAN);
                    attach.screen_at = now.checked_sub(SYNC_SPAN);
                    attach.repaint_due = true;
                }
            }
        }
        let cut = self.term.feed_to_ground(bytes);
        if cut > 0 {
            self.tail.clear();
        }
        self.tail.extend_from_slice(&bytes[cut..]);
        if self.tail.len() > 64 * 1024 {
            self.tail.clear();
        }
        let Some(attach) = self.attach.as_mut() else {
            return;
        };
        let now = Instant::now();
        attach.mid_sequence = if self.tail.is_empty() { None } else { attach.mid_sequence.or(Some(now)) };
        if now.duration_since(attach.made.0) >= 2 * SYNC_SPAN {
            attach.made = (now, 0, 0);
        } else if now.duration_since(attach.made.0) >= SYNC_SPAN {
            attach.made = (now, 0, attach.made.1);
        }
        attach.made.1 += bytes.len();
        attach.step_bytes += bytes.len();
        // Reads less than a few ms apart are one write (a frame read in
        // pieces); a stream that does not pause has no steps.
        let gap = attach.output_at.map_or(SYNC_SPAN, |at| now.duration_since(at));
        if gap >= Duration::from_millis(5) && gap < SYNC_SPAN {
            attach.output_gap = if attach.output_gap.is_zero() { gap } else { (attach.output_gap * 3 + gap) / 4 };
            // A frame: the write before was a screenful's worth of bytes
            // (not a key's echo) and this one came soon after.
            let frame = attach.step_bytes >= FRAME_BYTES && gap <= FRAME_GAP;
            attach.frames = if frame { attach.frames.saturating_add(1) } else { 0 };
            attach.step_bytes = 0;
            attach.output_at = Some(now);
        } else if gap >= SYNC_SPAN {
            attach.output_gap = Duration::ZERO;
            attach.frames = 0;
            attach.step_bytes = 0;
            attach.output_at = Some(now);
        } else {
            // Within a write: not a step, but the output is newer.
            attach.output_at = Some(now);
        }
        if attach.syncing_until.is_some() {
            // Still behind while updates queue on the path or wait to go,
            // or while the program writes more than half of what the link
            // carries (the updates keep the queue short then, but bytes
            // would not): a slow link's rate, or for a program drawing
            // frames, what the ack clock lets through. That is what the
            // updates themselves got through, so most animations keep the
            // mode until they stop: going back to bytes only to fall behind
            // again costs a repaint, which restarts the ack clock, a stall
            // each time over a far path.
            let rate = attach.output.slow_rate().or_else(|| attach.output.drain_rate(now).filter(|_| attach.animating()));
            let heavy = rate.is_some_and(|rate| {
                attach.stream_len(attach.made.1.max(attach.made.2)) as f64 > rate * SYNC_SPAN.as_secs_f64() / 2.0
            });
            if heavy || attach.output.queueing() > LAG_LIMIT / 2 || !attach.output.all_sent() {
                attach.syncing_until = Some(now + SYNC_SPAN);
            }
            if attach.syncing_until.is_some_and(|until| now < until) || attach.repaint_due || !was_ground {
                // What this output makes on the screen goes as an update
                // (`pump`), not as these bytes.
                attach.repaint_due = true;
                return;
            }
            // Kept up that long, and the client has this screen: bytes again.
            attach.syncing_until = None;
            attach.shadow = None;
        }
        if attach.repaint_due {
            return;
        }
        let output = &attach.output;
        // Into a compressed stream new bytes take about a ratio's share.
        let incoming = attach.stream_len(bytes.len());
        let lag = output.pending() + attach.stream_len(attach.zraw.len()) + incoming;
        // Time to drain at one window per round trip; a repaint only helps
        // once the client has the last one.
        let window = output.window() as f64;
        // Past two windows: on a slow link the window is small, so is this.
        // Or the path itself holds it up: the window's worth already sent
        // waits in a queue (a slow link) longer than the limit.
        // On a slow link: longer than the limit to drain at its rate.
        // Or a program drawing in steps that the ack clock would take
        // longer than the limit to let through, before the rate is known.
        let stepped = attach.animating();
        let slow = output.first_acked()
            && ((lag > 2 * output.window() as usize && output.srtt().is_some_and(|srtt| srtt.mul_f64(lag as f64 / window) > LAG_LIMIT))
                || (output.pending() > 0 && output.queueing() > LAG_LIMIT)
                || (attach.screens && output.slow_rate().is_some_and(|rate| lag > 512 && lag as f64 / rate > LAG_LIMIT.as_secs_f64()))
                || (attach.screens && stepped && output.drain_rate(now).is_some_and(|rate| lag > 512 && lag as f64 / rate > LAG_LIMIT.as_secs_f64())));
        // Memory: what is held, staged bytes as they are.
        let held = output.pending() + attach.zraw.len() + bytes.len();
        if !(held > OUTPUT_BACKLOG || slow) {
            attach.push_output(bytes);
        } else if output.first_acked() {
            if attach.screens {
                attach.syncing_until = Some(now + SYNC_SPAN);
            }
            self.refresh(now);
        } else {
            if attach.screens {
                attach.syncing_until = Some(now + SYNC_SPAN);
            }
            // Over the memory bound before the client has the last repaint:
            // restarting now would never let one arrive, so output is
            // dropped and pump repaints once it is acked.
            attach.repaint_due = true;
        }
    }
}

fn sanitize_name(name: &str) -> Option<String> {
    let clean: String = name
        .trim()
        .chars()
        .filter(|ch| !ch.is_control())
        .take(64)
        .collect();
    (!clean.is_empty()).then_some(clean)
}
