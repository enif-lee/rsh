use crate::crypto::{random_u64, Dir, Key};
use crate::forward::{self, Forwarder, Spec};
use crate::net::AuthStore;
use crate::proto::{
    self, encode_list, Action, Attached, ClientStream, Hello, Packet, Ready, Receiver, ReplayWindow, Sender, TmuxInfo,
    ServerStream, SessionInfo, FLAG_DETACH, FLAG_EXITED, FLAG_REDRAW, KIND_ATTACHED, KIND_CHANNEL, KIND_CONFIRM,
    KIND_DENIED, KIND_HELLO, KIND_KICKED, KIND_LIST, KIND_LIST_REPLY, KIND_PING, KIND_PONG, KIND_READY, KIND_STREAM,
};
use crate::path::PathModel;
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
/// How long a program is quiet before its session gives back spare memory.
const TRIM_AFTER: Duration = Duration::from_secs(2);
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
/// Output this many times smaller deflated is compressed at the smallest
/// level on any link (`z_level`): a dashboard is 10 to 15, a full screen of
/// random colours 2 to 3.
const WELL_COMPRESSED: f64 = 4.0;
/// Pushes this large or larger say how well output compresses.
const RATIO_MIN: usize = 256;
/// Once at level 9 for output that compresses well, it stays there until
/// output compresses less than this (`pick_level`): output near the line
/// does not swap compressors back and forth.
const STILL_COMPRESSED: f64 = 3.0;
/// A level holds at least this long, but for the link limiting the screen,
/// which takes level 9 at once (`pick_level`).
const LEVEL_DWELL: Duration = Duration::from_secs(1);
/// Output for the compressor waits while the program is still writing (its
/// next read this soon), up to `GATHER_MAX` or `GATHER_BYTES`: one flush
/// and datagram for what came in many small reads (`yes`, a build log), not
/// a datagram of headers around a few bytes each. Output that stops (a
/// key's echo) goes after `GATHER_GAP`.
const GATHER_GAP: Duration = Duration::from_millis(2);
/// Frames on a path without a measured bottleneck use a shorter quiet
/// gap. Slow paths retain coalescing to avoid extra packets/flushes.
const FRAME_GATHER_GAP: Duration = Duration::from_millis(1);
const GATHER_MAX: Duration = Duration::from_millis(10);
const GATHER_BYTES: usize = 16 * 1024;
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

impl TmuxList {
    fn ask(&mut self, now: Instant) {
        let (send, receive) = std::sync::mpsc::channel();
        thread::spawn(move || { let _ = send.send(crate::tmux::list()); });
        self.asking = Some((now, receive));
    }

    fn invalidate(&mut self, now: Instant) {
        self.known = None;
        // A lookup started before rename can still carry the old name.
        // Keep its waiting requests, but replace that lookup as well.
        self.asking = None;
        if !self.waiting.is_empty() { self.ask(now); }
    }
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
    /// When the program last wrote, and whether the terminal's spare
    /// history rows were given back since (`Terminal::trim`).
    output_seen: Instant,
    trimmed: bool,
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
    /// The path as the client's arrival reports show it, when it sends
    /// them (`FEATURE_PATH`).
    path: Option<PathModel>,
    /// The client inflates a generation sent as one deflate stream, and
    /// whether it asked for that on any link (HELLO features).
    zstream: bool,
    compress: bool,
    /// The generation under way is one deflate stream: its compressor.
    z: Option<proto::Deflater>,
    /// Output not yet in the stream: for the compressor (one flush per
    /// pump), or until the plain generation before is all acked.
    zraw: Vec<u8>,
    /// Since when `zraw` has gathered (see `GATHER_GAP`).
    gathering: Option<Instant>,
    /// The generation `stage` opened to start compressing, which repaints
    /// nothing.
    switched: Option<u32>,
    /// Bytes of the last screen update as sent, and how many went.
    update_len: usize,
    /// When the updates sent so far are off a slow link at the path
    /// model's rate (see `screen_due_at`).
    link_free: Option<Instant>,
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
    /// When the compressor took its level (`LEVEL_DWELL`).
    z_since: Option<Instant>,
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
    // The UDP socket's and the TCP side's entries come first in `fds`.
    let mut net;
    while !shared.stop.load(Ordering::Relaxed) {
        let (mut fds, timeout) = {
            let state = shared.state.lock().unwrap_or_else(|p| p.into_inner());
            let mut fds = vec![libc::pollfd {
                fd: state.socket.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            }];
            state.tcp.pollfds(&mut fds);
            net = fds.len();
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
        // Only when there is something: a call that finds nothing is still
        // a recv and an accept, at every wake.
        if fds[..net].iter().any(|fd| fd.revents != 0) {
            state.receive(&mut buf, now);
        }
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

/// Deflate's level for the next push into a compressed stream at `current`
/// (None: a new one): the smallest output where the link limits the screen
/// (a slow path, or screen updates; it limits what there is to compress
/// too) or where output compresses well (level 9 is fast there: a
/// dashboard is 180 MB/s); the fastest elsewhere (a full screen of colours:
/// 177 MB/s at 2, 39 at 9, for 0.43 and 0.36 of its size). Each change
/// starts a compressor, so a level is held `LEVEL_DWELL` (`held`: since it
/// was taken) and output compressing between `STILL_COMPRESSED` and
/// `WELL_COMPRESSED` keeps the one it has.
fn pick_level(current: Option<u8>, ratio: f64, limited: bool, held: Option<Duration>) -> u8 {
    if limited {
        return proto::SMALL_LEVEL;
    }
    let well = if current == Some(proto::SMALL_LEVEL) { ratio >= STILL_COMPRESSED } else { ratio >= WELL_COMPRESSED };
    let want = if well { proto::SMALL_LEVEL } else { proto::FAST_LEVEL };
    match current {
        Some(level) if level != want && held.is_some_and(|held| held < LEVEL_DWELL) => level,
        _ => want,
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
            output_seen: now,
            trimmed: false,
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
                path: None,
                zstream: false,
                compress: false,
                z: None,
                zraw: Vec::new(),
                gathering: None,
                update_len: 0,
                link_free: None,
                updates: 0,
                switched: None,
                output_at: None,
                output_gap: Duration::ZERO,
                frames: 0,
                step_bytes: 0,
                made: (now, 0, 0),
                ratio: 4.0,
                z_since: None,
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
            if let Some(at) = session.attach.as_ref().filter(|a| a.output.all_sent()).and_then(Attachment::gathered_at).filter(|_| reachable) {
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
        if let Action::Rename { target, name, tmux } = &hello.action {
            let cleaned = if *tmux { Some(crate::tmux::clean_name(name)).filter(|s| !s.is_empty()) } else { sanitize_name(name) };
            if cleaned.as_deref() != Some(name.as_str()) {
                return refuse("use a nonempty name of at most 64 characters without controls or tmux separators".into());
            }
            if *tmux {
                if let Err(err) = crate::tmux::rename(target, name) { return refuse(err); }
                for session in self.sessions.values_mut().filter(|s| s.ended_at.is_none() && s.command == crate::tmux::command(target)) {
                    session.label = name.clone();
                    session.command = crate::tmux::command(name);
                }
                self.tmux.invalidate(now);
            } else {
                let Some(id) = target.parse::<u64>().ok().filter(|id| self.sessions.get(id).is_some_and(|s| live(s) && !crate::tmux::is_wrapper(&s.command))) else {
                    return refuse("unknown rsh session".into());
                };
                if self.sessions.values().any(|s| live(s) && s.id != id && s.label == *name && !crate::tmux::is_wrapper(&s.command)) {
                    return refuse("session name already exists".into());
                }
                self.sessions.get_mut(&id).expect("session").label = name.clone();
            }
            let mut done = refuse(String::new());
            done.label = name.clone();
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
                Action::Kill(_) | Action::Rename { .. } => None,
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
            path: (hello.features & proto::FEATURE_PATH != 0).then(|| PathModel::new(now)),
            zstream: hello.features & proto::FEATURE_ZSTREAM != 0,
            compress: hello.features & proto::FEATURE_COMPRESS != 0,
            z: None,
            zraw: Vec::new(),
            gathering: None,
            update_len: 0,
            link_free: None,
            updates: 0,
            switched: None,
            output_at: None,
            output_gap: Duration::ZERO,
            frames: 0,
            step_bytes: 0,
            made: (now, 0, 0),
            ratio: 4.0,
            z_since: None,
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
                output_seen: now,
                trimmed: false,
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
                self.tmux.ask(now);
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
        let found = self.tmux.known.as_ref().filter(|_| tmux).map(|(_, found)| found.as_slice()).unwrap_or_default();
        let joined: Vec<u64> = found.iter().take(60).map(|t| {
            self.sessions.values()
                .find(|s| s.ended_at.is_none() && s.pty.is_some() && crate::tmux::is_wrapper(&s.command) && s.label == t.name)
                .map_or(0, |s| s.id)
        }).collect();
        let mut sessions: Vec<&Session> = self.sessions.values().collect();
        // Each displayed tmux row needs its wrapper's live rsh state, even
        // when older plain sessions fill the response's 60-session limit.
        sessions.sort_by_key(|s| (!joined.contains(&s.id), s.created, s.id));
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
                    // Three idle keepalives (`client::IDLE_HEARTBEAT`): one
                    // lost does not show an idle client as gone.
                    .is_some_and(|a| now.duration_since(a.last_heard) < 3 * crate::client::IDLE_HEARTBEAT),
                cols: s.term.size().0,
                rows: s.term.size().1,
                created_unix: s.created.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
                idle_secs: now.duration_since(s.last_active).as_secs(),
                command: s.command.clone(),
                title: s.term.title().chars().take(80).collect(),
            })
            .collect();
        let unix = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let tmux: Vec<TmuxInfo> = found
            .iter()
            .take(60)
            .zip(joined)
            .map(|(t, session)| TmuxInfo {
                name: t.name.chars().take(64).collect(),
                clients: t.clients,
                cols: t.cols,
                rows: t.rows,
                idle_secs: unix.saturating_sub(t.activity),
                session,
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
                attach.link_free = None;
                if let Some(path) = attach.path.as_mut() {
                    path.forget(now);
                }
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
        if let (Some(path), Some(report)) = (attach.path.as_mut(), proto::Feedback::decode(stream.feedback).filter(|_| !stream.feedback.is_empty())) {
            path.feedback(&report, now);
            if !is_tcp_addr(attach.addr) {
                // A rate over `SLOW_RATE` holds nothing up for the sender (it
                // takes none over it from its own measure either), and its
                // window is then the model's: a long burst over a fast path
                // spreads too (1.3 MB/s read over RTT 300 ms with nothing in
                // the way kept the sender's own shrunken window).
                attach.output.set_path(path.rate(now).filter(|rate| *rate < proto::SLOW_RATE), path.window_rtt(now));
            }
        }
        attach.open_window(now);
        let advanced = attach.output.on_ack(stream.out_gen, stream.out_ack, now, !stream.data.is_empty() || stream.flags & proto::FLAG_REPEAT != 0);
        if !stream.data.is_empty() && stream.in_offset > attach.input.next {
            // Past a gap: acked at once, so duplicate acks reach the client.
            attach.reply_due = true;
        } else if !stream.data.is_empty() {
            attach.ack_at.get_or_insert(now + INPUT_ACK_DELAY);
        } else if stream.flags & !(proto::FLAG_REPEAT | proto::FLAG_FEEDBACK) != 0 || (!advanced && attach.output.deadline().is_none()) {
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
        let gathered = |a: &Attachment| a.gathered_at().is_some_and(|at| now >= at);
        if exit_code.is_some() || (attach.output.all_sent() && gathered(attach)) {
            attach.stage(exit_code.is_some());
        }
        if exit_code.is_some() {
            // No output follows the exit, so the end only moves when a
            // repaint starts a new generation; it must follow that.
            attach.end_offset = Some(attach.output.end());
        }
        attach.open_window(now);
        let mut sent = false;
        let mut batch = Vec::new();
        loop {
            // A generation that is one deflate stream needs no more of it.
            let squeeze = attach.deflate && attach.z.is_none() && (attach.syncing_until.is_some() || attach.output.srtt().is_some_and(|srtt| srtt >= DEFLATE_RTT));
            let chunk = attach.output.next_chunk_max(now, if squeeze { proto::SQUEEZED } else { proto::CHUNK });
            let ack_due = attach.reply_due || attach.ack_at.is_some_and(|at| now >= at);
            if chunk.is_none() && attach.output.all_sent() && gathered(attach) && attach.stage(false) > 0 {
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
            let udp = !is_tcp_addr(attach.addr);
            // Each datagram a number of its own (copies too), for the path
            // model: its size on the wire is known only once sealed, and
            // the number's four bytes are part of it.
            let number = |path: &mut Option<PathModel>, len: usize, copy: bool| {
                path.as_mut().filter(|_| udp).map(|path| path.number(Instant::now(), len + 4 + proto::SEAL_OVERHEAD + 28, copy))
            };
            let mut stream = ServerStream {
                flags,
                in_ack: attach.input.next,
                out_gen: attach.output.gen,
                out_offset: offset,
                exit_code: exit_code.unwrap_or(0),
                end_offset: attach.end_offset.unwrap_or_else(|| attach.output.end()),
                pn: None,
                data: &data,
            };
            stream.pn = number(&mut attach.path, stream.encode().len(), false);
            let sealed = proto::seal(&attach.key, Dir::ToClient, KIND_STREAM, attach.id, &stream.encode());
            // On a lossy path with room to spare, data goes twice (each copy
            // sealed apart, as the replay window wants unique nonces).
            // On at the first resend: a loss seen is likely the path's.
            // TCP loses nothing.
            let loss = attach.output.loss();
            let resent = attach.output.take_resend();
            // A slow link has no room for copies (with the path model: when
            // a queue stands or they would not fit in its rate).
            let roomy = match attach.path.as_ref() {
                Some(path) => path.room_for_copies(now),
                None => attach.output.slow_rate().is_none(),
            };
            // With the path model, by the datagrams it saw lost (copies
            // hide none of those: an original lost counts whether its copy
            // came or not); without it, by resends, on at the first. A
            // resend alone (a timeout early in a session, before the round
            // trip is known) kept copies on for half a minute over a clean
            // LAN, every datagram twice.
            let modelled = attach.modelled();
            let lost = match attach.path.as_ref().filter(|_| modelled) {
                Some(path) => path.loss().0,
                None => loss,
            };
            let first = resent && !modelled;
            attach.doubling = !is_tcp_addr(attach.addr) && roomy && (first || lost > DOUBLE_ON || (attach.doubling && lost > DOUBLE_OFF));
            // Nor has a path whose acks show a queue growing (a link that
            // turned slow before its rate is known: copies would take half
            // of it); a passing one, so doubling stays on for after it.
            let room = attach.output.in_flight() < attach.output.window() / 2 && !attach.output.waited();
            // The copy after the original: numbers go out in order, which
            // the path model reads trains by.
            batch.push(sealed);
            // An independently deflated datagram cannot be cut to a prefix.
            let small = stream.data.len() < proto::CHUNK && (attach.syncing_until.is_none() || attach.update_len <= proto::CHUNK);
            let probe = udp && small && flags & proto::FLAG_DEFLATE == 0 && !stream.data.is_empty() && attach.path.as_mut().is_some_and(|path| path.probe_due(now));
            if !stream.data.is_empty() && (attach.doubling && room || probe) {
                // A capacity probe needs only a short second datagram. Its
                // prefix is ordinary duplicate stream data; it changes no
                // protocol or receive state and costs at most 64 data bytes.
                let data = if probe && !(attach.doubling && room) { &stream.data[..stream.data.len().min(64)] } else { stream.data };
                let mut copy = ServerStream { data, pn: None, ..stream.clone() };
                copy.pn = number(&mut attach.path, copy.encode().len(), true);
                batch.push(proto::seal(&attach.key, Dir::ToClient, KIND_STREAM, attach.id, &copy.encode()));
            }
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
            if let Some(path) = attach.path.as_mut() {
                path.departed(Instant::now());
            }
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
            // A session whose program has been quiet a while gives back the
            // rows alacritty keeps spare for its history (a thousand at a
            // time: 3.8 MB at 160 columns), and its buffers what a burst of
            // output grew them to (they stay allocated otherwise).
            // (Not while the alternate screen is in use: tried again later.)
            if !session.trimmed && now.duration_since(session.output_seen) >= TRIM_AFTER {
                session.trimmed = session.term.trim();
            }
            if session.pty_in.is_empty() {
                crate::proto::shrink(&mut session.pty_in);
            }
            if session.tail.capacity() > 64 * 1024 && session.tail.len() < session.tail.capacity() / 4 {
                session.tail.shrink_to(session.tail.len() * 2);
            }
            if let Some(attach) = session.attach.as_mut().filter(|a| a.output.all_sent() && a.output.in_flight() == 0) {
                attach.output.trim();
                if let Some(path) = attach.path.as_mut() {
                    path.trim();
                }
                if attach.zraw.capacity() > 64 * 1024 && attach.zraw.is_empty() {
                    attach.zraw.shrink_to(0);
                }
                // Quiet output only: a compressor started again each frame
                // would cost more than it saves.
                if attach.zraw.is_empty() && now.duration_since(session.output_seen) >= TRIM_AFTER {
                    if let Some(z) = attach.z.as_mut() {
                        z.park();
                    }
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
                let gathered = attach.output.all_sent() && attach.gathered_at().is_some_and(|at| now >= at);
                if attach.output.deadline().is_some_and(|deadline| now >= deadline) || attach.reply_due || ack_due || attach.next_screen_due(now) || gathered {
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
            let in_flight = self.output.in_flight_wire();
            let update = self.update_len as u64 + proto::WIRE_OVERHEAD;
            let rate = self.output.slow_rate().expect("slow");
            let gap = if self.modelled() {
                // With the path model's rate the link's time is reckoned,
                // not measured: the next update goes once what went before
                // is off the link (`link_free`), with the screen as it is
                // then. Not a frame early to keep the program's pace: an
                // update of 38 ms over 33 ms frames then skipped whole
                // frames, gaps of 66 to 80 ms where the link's pace gives
                // 33 to 55. In flight past the round trip and four updates
                // (losses, resends) holds them as a safeguard.
                if in_flight > 0 && in_flight + update > bdp + 4 * update {
                    return None;
                }
                let free = self.link_free.unwrap_or(last);
                free.saturating_duration_since(last)
            } else {
                // A slow link: the next update waits until it fits what the
                // path delivers in its least round trip plus two updates, so
                // updates do not queue on the link and age there (with one,
                // an update now and then waited a round trip).
                if in_flight > 0 && in_flight + update > bdp + 2 * update {
                    return None;
                }
                // Then once the last update is off the link at its rate.
                // Output that came while it was busy waits a little for
                // newer output: an update of the newest screen as it comes
                // shows it younger than one of an older screen at once.
                Duration::from_secs_f64(update as f64 / rate) + SLOW_ROOM
            };
            // When the program writes in steps (frames), output that came
            // while the link was busy waits for the next step if that is
            // due soon after; output that stopped goes at once.
            let idle = last + gap.max(self.screen_gap());
            // With the path model's reckoning, not past a quarter of the
            // link's time: the wait adds to the gap between updates (a
            // 38 ms update and 20 ms of it made gaps of 58 ms, the slowest
            // frames shown; 1 Mbit/s random screens 1% low 8.4 to 11.4,
            // 5% low 16.6 to 19.7, for 8 ms of age).
            let wait = if self.modelled() || is_tcp_addr(self.addr) { FRESH_WAIT.min(gap / 4) } else { FRESH_WAIT };
            return Some(match self.output_at {
                Some(at) if at >= idle => at,
                Some(at) => {
                    let next = at + self.output_gap;
                    if next > idle && next <= idle + wait { next } else { idle }
                }
                None => idle,
            });
        }
        if self.queue_delay() < LAG_LIMIT / 2 {
            // Not faster than a frame every 20 ms (or the program's step):
            // a burst of output (a `cat`) would otherwise queue an update
            // per read.
            let tcp_animation = is_tcp_addr(self.addr) && self.animating();
            let gap = if tcp_animation {
                self.output_gap.mul_f64(0.9).clamp(MIN_SCREEN_GAP, SCREEN_GAP)
            } else {
                self.screen_gap()
            };
            let due = last + gap;
            return Some(if tcp_animation {
                self.output_at.map_or(due, |at| {
                    due.max((at + FRAME_GATHER_GAP).min(last + SCREEN_GAP))
                })
            } else {
                due
            });
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

    /// The sender's least window with no bottleneck in the path model:
    /// twice what the path took lately, or, over a path seen to take a
    /// burst at once (`PathModel::fast`), what the program writes (stream
    /// bytes a second; a burst after a pause, as an animation starting,
    /// opens it at once rather than a doubling a round trip from what the
    /// pause delivered), over the round trip. Kept current as output
    /// comes, not only as reports do (a round trip later).
    fn open_window(&mut self, now: Instant) {
        if !self.modelled() {
            return;
        }
        let Some(path) = self.path.as_ref() else {
            return;
        };
        let Some(rtt) = path.window_rtt(now) else {
            return;
        };
        // With neither a bottleneck nor a queue the smoothed round trip is
        // the path's, and the least may be a shorter path's (100 ms that
        // turned 300 kept the window a third short for seconds); with a
        // queue, sized by it the window would keep the queue standing.
        let rtt = if path.rate(now).is_none() && path.queue_ms() < 20.0 { path.srtt().map_or(rtt, |srtt| srtt.max(rtt)) } else { rtt };
        // The program's pace only on a path seen to take a burst at once:
        // on a slow one not yet measured, the first of a burst would queue.
        let made = if path.fast(now) { self.stream_len(self.made.1.max(self.made.2)) as f64 / SYNC_SPAN.as_secs_f64() } else { 0.0 };
        let rate = path.delivery_rate().max(made);
        self.output.set_path_window(rate, rtt);
    }

    /// An update of `update_len` went now: when the slow link is through
    /// with it at the path model's rate.
    fn reckon_link(&mut self, now: Instant) {
        if !self.modelled() {
            return;
        }
        if let (Some(rate), Some(bdp)) = (self.output.slow_rate(), self.output.slow_bdp()) {
            // Not before what is in flight past a round trip's worth (the
            // queue the acks show) is through, less a frame for acks that
            // come in bursts: the reckoning does not drift from the link.
            let queued = now + Duration::from_secs_f64(self.output.in_flight_wire().saturating_sub(bdp) as f64 / rate);
            let queued = queued.checked_sub(self.screen_gap()).unwrap_or(now);
            let start = self.link_free.map_or(now, |free| free.max(now)).max(queued);
            // Each datagram's overhead, not the update's once.
            let datagrams = (self.update_len as u64).div_ceil(proto::CHUNK as u64).max(1);
            let wire = self.update_len as u64 + datagrams * proto::WIRE_OVERHEAD;
            self.link_free = Some(start + Duration::from_secs_f64(wire as f64 / rate));
        }
    }

    /// Whether the path model speaks for the path (its reports came).
    fn modelled(&self) -> bool {
        self.path.as_ref().is_some_and(PathModel::reported) && !is_tcp_addr(self.addr)
    }

    /// How fast output drains, when that holds it up: a slow link's rate,
    /// or, with no path model, for a program drawing frames the ack
    /// clock's guess (which the model, seeing the bottleneck itself, does
    /// not need: a far path with no bottleneck takes every frame).
    fn drain_guess(&self, now: Instant) -> Option<f64> {
        if self.modelled() {
            return self.output.slow_rate();
        }
        self.output.slow_rate().or_else(|| self.output.drain_rate(now).filter(|_| self.animating()))
    }

    /// Bytes waiting for the link, not for an ack of bytes already on
    /// their way. Up to a bandwidth-delay product can be in flight with
    /// no queue. Its wire overhead takes part of that allowance too.
    fn drain_backlog(&self, pending: usize, now: Instant) -> usize {
        if !self.modelled() && !is_tcp_addr(self.addr) {
            return pending;
        }
        let bdp = if is_tcp_addr(self.addr) { self.output.drain_bdp(now) } else { self.output.slow_bdp() };
        let Some(bdp) = bdp else {
            return pending;
        };
        let flight = self.output.in_flight();
        let overhead = self.output.in_flight_wire() - flight;
        let carried = bdp.saturating_sub(overhead).min(flight);
        pending.saturating_sub(carried as usize)
    }

    /// Whether the path's model shows a bottleneck well enough to start
    /// screen updates: two trains showed its rate, or a queue stands in
    /// front of it. One spread train alone (a busy host or hop over a fast
    /// path) can otherwise start them unnecessarily on a far path.
    /// Without the model, true.
    fn bottleneck_shown(&self, now: Instant) -> bool {
        let Some(path) = self.path.as_ref().filter(|_| self.modelled()) else {
            return true;
        };
        path.rate_samples(now) >= 2 || path.queue_ms() >= 5.0
    }

    /// The queue in front of the path's bottleneck: the model's (from
    /// arrival times), else what the round trips show over their least.
    fn queue_delay(&self) -> Duration {
        match self.path.as_ref().filter(|_| self.modelled()) {
            Some(path) => Duration::from_secs_f64(path.queue_ms().max(0.0) / 1000.0),
            None => self.output.queueing(),
        }
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
            if self.zraw.is_empty() {
                self.gathering = Some(Instant::now());
            }
            self.zraw.extend_from_slice(bytes);
            false
        } else {
            self.output.push(bytes);
            true
        }
    }

    /// When what `zraw` gathered may go into the stream (see
    /// `GATHER_GAP`); None when nothing waits. Frames use a shorter quiet
    /// gap to join their PTY pieces with less delay and scheduling jitter.
    fn gathered_at(&self) -> Option<Instant> {
        let since = self.gathering.filter(|_| !self.zraw.is_empty())?;
        // Waiting for the plain generation's acks before compressing
        // (`stage`): nothing goes until they come, so no time is due.
        if self.z.is_none() && self.wants_z() && self.output.acked() != self.output.end() {
            return None;
        }
        if self.zraw.len() >= GATHER_BYTES {
            return Some(since);
        }
        let gap = if self.animating() && self.output.slow_rate().is_none() { FRAME_GATHER_GAP } else { GATHER_GAP };
        let quiet = self.output_at.map_or(since, |at| at + gap);
        Some(quiet.min(since + GATHER_MAX))
    }

    /// Deflate's level for what goes into the stream now (`pick_level`);
    /// `fresh`: for a new stream, which starts a compressor anyway, so no
    /// level is held for it (the last one's still decides the hysteresis).
    fn z_level(&self, now: Instant, fresh: bool) -> u8 {
        let limited = self.syncing_until.is_some() || self.output.slow_rate().is_some();
        let held = self.z_since.map(|at| now.saturating_duration_since(at)).filter(|_| !fresh);
        pick_level(self.z.as_ref().map(proto::Deflater::level), self.ratio, limited, held)
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
            // A new stream: its level by the policy, its own since now.
            let now = Instant::now();
            self.z = Some(proto::Deflater::new(self.z_level(now, true)));
            self.z_since = Some(now);
            self.output.reset(&[]);
            // A new generation that is no repaint: a repaint asked for the
            // one before still stands (the client asked with its number).
            self.switched = Some(self.output.gen);
        }
        let raw = std::mem::take(&mut self.zraw);
        let now = Instant::now();
        let level = self.z_level(now, false);
        let z = self.z.as_mut().expect("compressor");
        if z.level() != level {
            z.set_level(level);
            self.z_since = Some(now);
        }
        let packed = z.push(&raw);
        // Not from a few bytes (a key's echo): a flush's own bytes would
        // make anything look incompressible, and hold `z_level` low.
        if raw.len() >= RATIO_MIN {
            self.ratio = (raw.len() as f64 / packed.len().max(1) as f64).clamp(1.0, 64.0);
        }
        self.output.push(&packed);
        packed.len()
    }

    /// A new generation whose stream starts with `bytes` (a repaint),
    /// compressed when that is wanted.
    fn reset_output(&mut self, bytes: &[u8]) {
        self.zraw.clear();
        if self.wants_z() {
            let now = Instant::now();
            let level = self.z_level(now, true);
            if self.z.as_ref().map(proto::Deflater::level) != Some(level) {
                self.z_since = Some(now);
            }
            let mut z = proto::Deflater::new(level);
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
                    attach.reckon_link(now);
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
        attach.reckon_link(now);
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
        self.output_seen = Instant::now();
        self.trimmed = false;
        let was_ground = self.tail.is_empty();
        if let Some(attach) = self.attach.as_mut().filter(|a| a.screens && was_ground) {
            // On a slow link, output that would take long to drain starts
            // the screens-only mode at once, from the screen the client
            // will have: all before was sent, so no repaint is needed. Before
            // the link's rate is known, so does a program drawing in steps
            // (an animation) faster than the ack clock lets through: within
            // a frame or two of its start, not after a queue of it formed.
            let now = Instant::now();
            let rate = attach.drain_guess(now);
            if let Some(rate) = rate.filter(|_| {
                attach.syncing_until.is_none() && !attach.repaint_due && attach.output.all_sent() && attach.zraw.is_empty() && attach.bottleneck_shown(now)
            }) {
                let incoming = attach.stream_len(bytes.len());
                // Half the limit: the first frames of an animation would
                // otherwise queue before the mode starts.
                let backlog = attach.drain_backlog(attach.output.in_flight() as usize + incoming, now);
                if incoming > 128 && backlog as f64 / rate > LAG_LIMIT.as_secs_f64() / 2.0 {
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
            let rate = attach.drain_guess(now);
            let heavy = rate.is_some_and(|rate| {
                attach.stream_len(attach.made.1.max(attach.made.2)) as f64 > rate * SYNC_SPAN.as_secs_f64() / 2.0
            });
            if heavy || attach.queue_delay() > LAG_LIMIT / 2 || !attach.output.all_sent() {
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
        let backlog = attach.drain_backlog(lag, now);
        // Time to drain at one window per round trip; a repaint only helps
        // once the client has the last one.
        let window = output.window() as f64;
        // Past two windows: on a slow link the window is small, so is this.
        // Or the path itself holds it up: the window's worth already sent
        // waits in a queue (a slow link) longer than the limit.
        // On a slow link: longer than the limit to drain at its rate.
        // Or a program drawing in steps that the ack clock would take
        // longer than the limit to let through, before the rate is known.
        let stepped = attach.animating() && !attach.modelled();
        // A measured bandwidth-delay product can be in transit over TCP
        // too. Unsent bytes and flight beyond that allowance remain queued.
        let stepped_lag = if is_tcp_addr(attach.addr) { backlog } else { lag };
        let slow = output.first_acked()
            && ((lag > 2 * output.window() as usize && output.srtt().is_some_and(|srtt| srtt.mul_f64(lag as f64 / window) > LAG_LIMIT))
                || (output.pending() > 0 && attach.queue_delay() > LAG_LIMIT)
                || (attach.screens && attach.bottleneck_shown(now) && output.slow_rate().is_some_and(|rate| backlog > 512 && backlog as f64 / rate > LAG_LIMIT.as_secs_f64()))
                || (attach.screens && stepped && output.drain_rate(now).is_some_and(|rate| stepped_lag > 512 && stepped_lag as f64 / rate > LAG_LIMIT.as_secs_f64())));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_listing_keeps_live_wrapper_state_past_the_plain_session_limit() {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("test", Action::Default, (80, 24), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let addr = "127.0.0.1:12345".parse().unwrap();
        let attached = state.carry_out(1, &key, 0, &hello, &packet, addr, now);
        let wrapper = state.sessions.get_mut(&attached.session).unwrap();
        wrapper.label = "late".into();
        wrapper.command = crate::tmux::command("late");
        let pid = wrapper.pid;
        for id in 1..=60 {
            state.sessions.insert(id, Session {
                id, host: "test".into(), label: format!("older-{id}"), command: String::new(),
                created: UNIX_EPOCH + Duration::from_secs(id), pty: None, pid,
                closed_at: None, exit_code: None, ended_at: None, term: Terminal::with_size(2, 2),
                tail: Vec::new(), pty_in: VecDeque::new(), attach: None, last_active: now,
                last_client: now, output_seen: now, trimmed: false,
            });
        }
        state.tmux.known = Some((now, vec![crate::tmux::Found { name: "late".into(), clients: 1, activity: 0, cols: 80, rows: 24 }]));
        let with_tmux = state.list_reply(&key, &[0; 24], addr, true);
        let plain = state.list_reply(&key, &[0; 24], addr, false);
        state.sessions.retain(|id, _| *id == attached.session);
        drop(state);
        let decode = |bytes: &[u8]| proto::decode_list(&proto::open(&key, Dir::ToClient, bytes).unwrap().payload).unwrap().1;
        let listed = decode(&with_tmux);
        assert_eq!(listed.sessions.len(), 60);
        assert_eq!(listed.tmux[0].session, attached.session);
        assert!(listed.sessions.iter().any(|s| s.id == attached.session && s.attached));
        let plain = decode(&plain);
        assert!(plain.tmux.is_empty());
        assert_eq!(plain.sessions.len(), 60);
        assert!(!plain.sessions.iter().any(|s| s.id == attached.session), "plain LIST keeps its existing creation order");
    }

    #[test]
    fn rename_invalidates_inflight_tmux_lookup_without_losing_waiters() {
        let key = Key::from_token("test");
        let mut tmux = TmuxList::default();
        let (send, receive) = std::sync::mpsc::channel();
        tmux.asking = Some((Instant::now(), receive));
        tmux.waiting.push((key, [0; 24], "127.0.0.1:1".parse().unwrap()));
        tmux.invalidate(Instant::now());
        assert!(send.send(Vec::new()).is_err(), "the pre-rename result cannot enter the cache");
        assert_eq!(tmux.waiting.len(), 1);
        assert!(tmux.asking.is_some(), "pending requests get a fresh lookup");
        assert!(tmux.known.is_none());
    }

    #[test]
    fn stepped_output_gathers_briefly_without_skipping_generation_waits() {
        let server = Server::spawn();
        let mut state = server.state();
        let start = Instant::now();
        let key = Key::from_token("test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("test", Action::Default, (80, 24), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let attached = state.carry_out(1, &key, 0, &hello, &packet, "127.0.0.1:12345".parse().unwrap(), start);
        let attach = state.sessions.get_mut(&attached.session).unwrap().attach.as_mut().unwrap();
        attach.output = Sender::new(0);
        attach.zstream = true;
        attach.compress = true;
        attach.z = Some(proto::Deflater::new(proto::FAST_LEVEL));
        attach.zraw = b"echo".to_vec();
        attach.gathering = Some(start);
        attach.output_at = Some(start);
        attach.frames = 0;
        assert_eq!(attach.gathered_at(), Some(start + GATHER_GAP), "a small echo still coalesces");

        // Pieces of the same frame still join, using the latest piece's
        // quiet gap; flushing at once can send a partial frame.
        attach.frames = ANIMATION_FRAMES;
        attach.output_at = Some(start + Duration::from_micros(400));
        assert_eq!(attach.gathered_at(), Some(start + Duration::from_micros(400) + FRAME_GATHER_GAP));

        attach.output.set_path(Some(32_000.0), Some(100.0));
        assert_eq!(attach.gathered_at(), Some(start + Duration::from_micros(400) + GATHER_GAP), "a measured bottleneck retains coalescing");
        attach.output.set_path(None, Some(100.0));

        // Even a frame cannot start the compressed generation before the
        // preceding plain generation reaches the peer.
        attach.z = None;
        attach.output.push(b"plain");
        assert_eq!(attach.gathered_at(), None);
        assert!(attach.output.next_chunk(start).is_some());
        attach.output.on_ack(0, 5, start + Duration::from_millis(1), false);
        assert_eq!(attach.gathered_at(), Some(start + Duration::from_micros(400) + FRAME_GATHER_GAP));

        attach.z = Some(proto::Deflater::new(proto::FAST_LEVEL));
        attach.frames = 0;
        attach.zraw.resize(GATHER_BYTES, b'x');
        assert_eq!(attach.gathered_at(), Some(start), "large unstepped output still flushes at the byte limit");
    }

    #[test]
    fn bytes_in_transit_do_not_start_screen_updates_but_a_backlog_does() {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("test", Action::Default, (80, 24), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let attached = state.carry_out(1, &key, 0, &hello, &packet, "127.0.0.1:12345".parse().unwrap(), now);
        let session = state.sessions.get_mut(&attached.session).unwrap();

        let start = now - Duration::from_secs(1);
        for (rate, unsent, corroborated, screens) in [
            (55_000.0, 0, true, false),
            (55_000.0, 0, false, false),
            (55_000.0, 14_000, true, true),
            (8_000.0, 0, true, true),
            (8_000.0, 0, false, false),
        ] {
            let attach = session.attach.as_mut().unwrap();
            // A few spread datagrams can show a low rate on a fast path.
            // One sample leaves the proactive gate closed, exercising the
            // later lag check; two open it, exercising both entry paths.
            let mut path = PathModel::new(start);
            let count = if corroborated { 3 } else { 2 };
            let arrivals = (0..count).map(|i| (path.number(start, 600, false), 150_000 + i * 10_000)).collect();
            let sent_us = 150_000 + (count - 1) * 10_000;
            path.feedback(&proto::Feedback { sent_us, arrivals }, start + Duration::from_millis(320));
            assert_eq!(path.rate_samples(now), (count - 1) as usize);
            assert!(path.queue_ms() < 5.0);
            attach.path = Some(path);
            attach.output = Sender::new(0);
            attach.output.push(&vec![b'x'; 6_000]);
            while attach.output.next_chunk(now).is_some() {}
            assert_eq!(attach.output.in_flight(), 6_000);
            attach.output.push(&vec![b'x'; unsent]);
            attach.output.set_path(Some(rate), Some(300.0));
            attach.screens = true;
            attach.zstream = false;
            attach.z = None;
            attach.zraw.clear();
            attach.syncing_until = None;
            attach.repaint_due = false;
            attach.shadow = None;
            // At 55 KB/s the 6 KB in flight fits well inside the 300 ms
            // path. It is not 109 ms of queue; a new 600-byte write fits.
            // Unsent output, or flight beyond a real slow link's BDP,
            // still starts screen updates.
            session.output(&vec![b'x'; 600]);
            assert_eq!(session.attach.as_ref().unwrap().syncing_until.is_some(), screens, "rate {rate}, unsent {unsent}, corroborated {corroborated}");
        }
    }

    #[test]
    fn tcp_transit_credit_requires_a_measured_bdp_and_keeps_excess_queued() {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("test", Action::Default, (80, 24), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let tcp = "[fd72:7368:7463:7000::1]:1".parse().unwrap();
        assert!(is_tcp_addr(tcp));
        let attached = state.carry_out(1, &key, 0, &hello, &packet, tcp, now);
        let attach = state.sessions.get_mut(&attached.session).unwrap().attach.as_mut().unwrap();
        attach.path = None;
        attach.output = Sender::new(0);
        attach.output.set_lossless(true);
        attach.output.push(&vec![b'x'; 6_000]);
        while attach.output.next_chunk(now).is_some() {}
        assert_eq!(attach.output.in_flight(), 6_000);
        assert_eq!(attach.drain_backlog(6_600, now), 6_600, "unknown capacity gets no transit credit");
        // A modelled rate with no sampled floor must not bypass the gate.
        attach.output.set_path(Some(55_000.0), Some(300.0));
        assert_eq!(attach.drain_backlog(6_600, now), 6_600, "model alone is not a sampled floor");
        attach.output = Sender::new(0);
        attach.output.set_lossless(true);
        let sent_at = now - Duration::from_millis(300);
        attach.output.push(b"x");
        let (offset, data) = attach.output.next_chunk(sent_at).unwrap();
        let end = offset + data.len() as u64;
        assert!(attach.output.on_ack(0, end, now, false));
        attach.output.push(&vec![b'x'; 6_000]);
        while attach.output.next_chunk(now).is_some() {}
        // Synthetic rate now isolates queue accounting after a real RTT sample.
        attach.output.set_path(Some(55_000.0), Some(300.0));
        assert_eq!(attach.drain_backlog(6_600, now), 600, "in-flight data fits the measured BDP");
        attach.output.push(&vec![b'x'; 14_000]);
        assert_eq!(attach.drain_backlog(20_600, now), 14_600, "unsent bytes remain backlog");
        attach.output.set_path(Some(8_000.0), Some(300.0));
        let slow = attach.drain_backlog(20_600, now);
        assert!(slow > 18_200, "wire overhead and flight beyond BDP remain backlog: {slow}");
        attach.addr = "127.0.0.1:12345".parse().unwrap();
        assert_eq!(attach.drain_backlog(20_600, now), 20_600, "unmodelled UDP is unchanged");
    }

    #[test]
    fn tcp_transit_does_not_trigger_proactive_or_stepped_screen_modes() {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("test", Action::Default, (80, 24), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let tcp = "[fd72:7368:7463:7000::1]:1".parse().unwrap();
        let attached = state.carry_out(1, &key, 0, &hello, &packet, tcp, now);
        let session = state.sessions.get_mut(&attached.session).unwrap();
        for (rate, unsent, expected) in [(55_000.0, 0, false), (55_000.0, 14_000, true), (8_000.0, 0, true)] {
            let attach = session.attach.as_mut().unwrap();
            attach.path = None;
            attach.output = Sender::new(0);
            attach.output.set_lossless(true);
            let sent_at = now - Duration::from_millis(300);
            attach.output.push(b"x");
            let (offset, data) = attach.output.next_chunk(sent_at).unwrap();
            let end = offset + data.len() as u64;
            assert!(attach.output.on_ack(0, end, now, false));
            attach.output.push(&vec![b'x'; 6_000]);
            while attach.output.next_chunk(now).is_some() {}
            attach.output.push(&vec![b'x'; unsent]);
            // Synthetic rate isolates both screen-entry decisions.
            attach.output.set_path(Some(rate), Some(300.0));
            attach.screens = true;
            attach.zstream = false;
            attach.z = None;
            attach.zraw.clear();
            attach.syncing_until = None;
            attach.repaint_due = false;
            attach.shadow = None;
            attach.frames = ANIMATION_FRAMES;
            attach.step_bytes = FRAME_BYTES;
            attach.output_gap = Duration::from_millis(16);
            attach.output_at = Some(Instant::now() - Duration::from_millis(16));
            session.output(&vec![b'x'; 600]);
            assert_eq!(session.attach.as_ref().unwrap().syncing_until.is_some(), expected,
                       "rate {rate}, unsent {unsent}");
        }
    }

    #[test]
    fn tcp_transit_near_sample_keeps_backlog_equal_to_main() {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("test", Action::Default, (80, 24), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let tcp = "[fd72:7368:7463:7000::1]:1".parse().unwrap();
        let attached = state.carry_out(1, &key, 0, &hello, &packet, tcp, now);
        let attach = state.sessions.get_mut(&attached.session).unwrap().attach.as_mut().unwrap();
        for rtt_ms in [1, 19] {
            attach.path = None;
            attach.output = Sender::new(0);
            attach.output.set_lossless(true);
            let sent_at = now - Duration::from_millis(rtt_ms);
            attach.output.push(b"x");
            let (offset, data) = attach.output.next_chunk(sent_at).unwrap();
            let end = offset + data.len() as u64;
            assert!(attach.output.on_ack(0, end, now, false));
            attach.output.push(&vec![b'x'; 6_000]);
            while attach.output.next_chunk(now).is_some() {}
            attach.output.set_path(Some(55_000.0), Some(300.0));
            for lag in [6_600, 20_600] {
                assert_eq!(attach.drain_backlog(lag, now), lag, "near RTT {rtt_ms}, lag {lag}");
            }
            // Existing UDP modelled/unmodelled selection bypasses drain_bdp.
            attach.addr = "127.0.0.1:12345".parse().unwrap();
            assert_eq!(attach.drain_backlog(6_600, now), 6_600, "unmodelled UDP remains unchanged");
            attach.addr = tcp;
        }
    }

    #[test]
    fn the_level_follows_the_link_at_once_and_the_output_with_a_lag() {
        let (small, fast) = (proto::SMALL_LEVEL, proto::FAST_LEVEL);
        let long = Some(LEVEL_DWELL * 2);
        let short = Some(LEVEL_DWELL / 4);
        // A new stream: by how well output compressed.
        assert_eq!(pick_level(None, 10.0, false, None), small);
        assert_eq!(pick_level(None, 2.3, false, None), fast);
        // The link limiting the screen: level 9 at once, held or not.
        assert_eq!(pick_level(Some(fast), 2.3, true, short), small);
        // Between the lines the level stays: up at 4, down below 3.
        assert_eq!(pick_level(Some(fast), 3.5, false, long), fast);
        assert_eq!(pick_level(Some(small), 3.5, false, long), small);
        assert_eq!(pick_level(Some(small), 2.9, false, long), fast);
        assert_eq!(pick_level(Some(fast), 4.0, false, long), small);
        // Not within the dwell of the last change.
        assert_eq!(pick_level(Some(small), 2.0, false, short), small);
        assert_eq!(pick_level(Some(fast), 10.0, false, short), fast);
    }
}

#[cfg(test)]
mod tcp_cadence_margin_tests {
    use super::*;

    fn attachment() -> (Attachment, Instant) {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("test", Action::Default, (80, 24), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let tcp = "[fd72:7368:7463:7000::1]:1".parse().unwrap();
        let attached = state.carry_out(1, &key, 0, &hello, &packet, tcp, now);
        let mut attach = state.sessions.get_mut(&attached.session).unwrap().attach.take().unwrap();
        attach.path = None;
        attach.output = Sender::new(0);
        attach.output.set_lossless(true);
        attach.repaint_due = true;
        attach.shadow = Some(Terminal::with_history(80, 24, 0));
        attach.screen_at = Some(now);
        attach.output_at = None;
        attach.frames = ANIMATION_FRAMES;
        attach.mid_sequence = None;
        (attach, now)
    }

    #[test]
    fn tcp_cadence_margin_scales_raw_step_before_existing_bounds() {
        let (mut attach, now) = attachment();
        assert!(attach.output.slow_bdp().is_none());
        assert!(attach.queue_delay() < LAG_LIMIT / 2);
        for (step_ms, due_ms, global_ms) in [(1,8,8),(8,8,8),(10,9,10),(20,18,20),(25,20,20)] {
            attach.output_gap = Duration::from_millis(step_ms);
            assert_eq!(attach.screen_due_at(), Some(now + Duration::from_millis(due_ms)), "step {step_ms}");
            assert_eq!(attach.screen_gap(), Duration::from_millis(global_ms), "global cadence stays unchanged");
        }
    }

    #[test]
    fn tcp_cadence_margin_preserves_nonanimating_and_both_udp_paths() {
        let (mut attach, now) = attachment();
        attach.output_gap = Duration::from_millis(10);
        attach.frames = ANIMATION_FRAMES - 1;
        assert_eq!(attach.screen_due_at(), Some(now + Duration::from_millis(20)));
        attach.frames = ANIMATION_FRAMES;
        attach.addr = "127.0.0.1:12345".parse().unwrap();
        assert!(!attach.modelled());
        assert_eq!(attach.screen_due_at(), Some(now + Duration::from_millis(10)));
        let sent = now - Duration::from_millis(300);
        let mut path = PathModel::new(sent);
        let pn = path.number(sent, 600, false);
        path.feedback(&proto::Feedback { sent_us: 300_000, arrivals: vec![(pn, 300_000)] }, now);
        attach.path = Some(path);
        assert!(attach.modelled());
        assert!(attach.output.slow_bdp().is_none());
        assert!(attach.queue_delay() < LAG_LIMIT / 2);
        assert_eq!(attach.screen_due_at(), Some(now + Duration::from_millis(10)));
    }

    #[test]
    fn tcp_cadence_margin_keeps_slow_queue_and_atomicity_gates() {
        let (mut attach, now) = attachment();
        attach.output_gap = Duration::from_millis(20);
        attach.update_len = 1000;
        attach.output.set_path(Some(32_000.0), Some(300.0));
        let serialized = Duration::from_secs_f64((1000 + proto::WIRE_OVERHEAD) as f64 / 32_000.0) + SLOW_ROOM;
        assert!(serialized > SCREEN_GAP);
        assert_eq!(attach.screen_due_at(), Some(now + serialized), "known slow branch unchanged");

        // Public ACK samples keep the least RTT near but show a long queue.
        attach.output = Sender::new(1);
        attach.output.set_lossless(true);
        let sent = now - Duration::from_millis(500);
        attach.output.push(b"a");
        let (offset, data) = attach.output.next_chunk(sent).unwrap();
        let end = offset + data.len() as u64;
        assert!(attach.output.on_ack(1, end, sent + Duration::from_millis(1), false));
        let sent = now - Duration::from_millis(400);
        attach.output.push(b"b");
        let (offset, data) = attach.output.next_chunk(sent).unwrap();
        let end = offset + data.len() as u64;
        assert!(attach.output.on_ack(1, end, now, false));
        assert!(attach.output.slow_bdp().is_none());
        assert!(attach.queue_delay() >= LAG_LIMIT / 2);
        let half = attach.output.srtt().unwrap() / 2;
        assert!(half > SCREEN_GAP && half < Duration::from_millis(250));
        assert_eq!(attach.screen_due_at(), Some(now + half), "high queue branch unchanged");

        attach.output = Sender::new(0);
        attach.mid_sequence = Some(now);
        assert_eq!(attach.next_screen_at(), Some(now + TAIL_WAIT), "unfinished escape tail still holds refresh");
        attach.repaint_due = false;
        assert_eq!(attach.screen_due_at(), None);
        attach.repaint_due = true;
        attach.shadow = None;
        assert_eq!(attach.screen_due_at(), None);
        attach.shadow = Some(Terminal::with_history(80, 24, 0));
        attach.output.push(b"unsent");
        assert_eq!(attach.screen_due_at(), None);
    }
}

#[cfg(test)]
mod tcp_fresh_quarter55_tests {
    use super::*;

    // Synthetic known rate isolates the deadline; no estimator-learning claim.
    fn with_attachment(check: impl FnOnce(&mut Attachment, Instant)) {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("fresh-quarter55-test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("fresh-quarter55-test", Action::Default, (100, 30), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let tcp = "[fd72:7368:7463:7000::1]:12345".parse().unwrap();
        let attached = state.carry_out(1, &key, 0, &hello, &packet, tcp, now);
        let attach = state.sessions.get_mut(&attached.session).unwrap().attach.as_mut().unwrap();
        let last = now - Duration::from_secs(2);
        attach.output = Sender::new(0);
        attach.output.set_lossless(true);
        attach.output.set_path(Some(21_000.0), Some(100.0));
        attach.path = None;
        attach.shadow = Some(Terminal::with_history(100, 30, 0));
        attach.repaint_due = true;
        attach.screen_at = Some(last);
        attach.update_len = 332;
        attach.output_gap = Duration::from_nanos(16_666_666);
        attach.output_at = Some(last + attach.output_gap);
        attach.frames = ANIMATION_FRAMES;
        attach.mid_sequence = None;
        attach.link_free = None;
        check(attach, last);
    }

    #[test]
    fn known_slow_tcp_does_not_wait_past_the_quarter_gap() {
        with_attachment(|attach, last| {
            assert!(is_tcp_addr(attach.addr));
            assert!(!attach.modelled());
            let step = attach.screen_gap();
            let gap = Duration::from_secs_f64((332 + proto::WIRE_OVERHEAD) as f64 / 21_000.0) + SLOW_ROOM;
            assert!(gap > step && step * 2 - gap > gap / 4);
            assert_eq!(attach.screen_due_at(), Some(last + gap));
            assert_eq!(attach.screen_gap(), step, "global cadence unchanged");
        });
    }

    #[test]
    fn both_udp_fresh_wait_paths_keep_their_old_deadlines() {
        with_attachment(|attach, last| {
            let gap = Duration::from_secs_f64((332 + proto::WIRE_OVERHEAD) as f64 / 21_000.0) + SLOW_ROOM;
            attach.addr = "127.0.0.1:12345".parse().unwrap();
            attach.output.set_lossless(false);
            assert!(!attach.modelled());
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap * 2));
            let mut path = PathModel::new(last);
            let arrivals = (0..2).map(|i| (path.number(last, 600, false), 100_000 + i * 10_000)).collect();
            path.feedback(&proto::Feedback { sent_us: 110_000, arrivals }, last + Duration::from_millis(220));
            attach.path = Some(path);
            attach.link_free = Some(last + gap);
            assert!(attach.modelled());
            assert_eq!(attach.screen_due_at(), Some(last + gap));
        });
    }

    #[test]
    fn eighty_ms_cap_unknown_rate_and_tail_gates_stay_unchanged() {
        with_attachment(|attach, last| {
            attach.output.set_path(Some(50_000.0), Some(100.0));
            attach.update_len = 3_636;
            attach.output_at = Some(last + attach.output_gap * 4);
            let gap = Duration::from_secs_f64((3_636 + proto::WIRE_OVERHEAD) as f64 / 50_000.0) + SLOW_ROOM;
            assert_eq!(gap, Duration::from_millis(80));
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap * 5));
            attach.update_len = 4_096;
            attach.output_at = Some(last + attach.output_gap * 5);
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap * 6));
            let began = last + Duration::from_millis(100);
            attach.mid_sequence = Some(began);
            assert_eq!(attach.next_screen_at(), Some(began + TAIL_WAIT));
            attach.mid_sequence = None;
            attach.output = Sender::new(0);
            attach.output.set_lossless(true);
            attach.output_at = Some(last);
            assert!(attach.output.slow_rate().is_none());
            assert_eq!(attach.screen_due_at(), Some(last + attach.output_gap.mul_f64(0.9).clamp(MIN_SCREEN_GAP, SCREEN_GAP)), "unknown-rate TCP follows the margin branch");
            attach.output.push(b"not sent");
            assert_eq!(attach.screen_due_at(), None);
            attach.output = Sender::new(0);
            attach.shadow = None;
            assert_eq!(attach.screen_due_at(), None);
        });
    }
}
#[cfg(test)]
mod tcp_margin_quiet65_tests {
    use super::*;

    // Deadline-only fixtures: no complete-producer-frame or runtime-FPS proof.
    fn with_attachment(check: impl FnOnce(&mut Attachment, Instant, Instant)) {
        let server = Server::spawn();
        let mut state = server.state();
        let now = Instant::now();
        let key = Key::from_token("margin-quiet65-test");
        let command = vec!["sleep".into(), "30".into()];
        let hello = crate::net::new_hello("margin-quiet65-test", Action::Default, (100, 30), &command);
        let packet = Packet { kind: KIND_CONFIRM, session: 1, counter: 0, nonce: [0; 24], payload: Vec::new() };
        let tcp = "[fd72:7368:7463:7000::1]:12345".parse().unwrap();
        let attached = state.carry_out(1, &key, 0, &hello, &packet, tcp, now);
        let attach = state.sessions.get_mut(&attached.session).unwrap().attach.as_mut().unwrap();
        let last = now - Duration::from_secs(2);
        attach.output = Sender::new(0);
        attach.output.set_lossless(true);
        attach.path = None;
        attach.shadow = Some(Terminal::with_history(100, 30, 0));
        attach.repaint_due = true;
        attach.screen_at = Some(last);
        attach.output_at = None;
        attach.output_gap = Duration::from_millis(20);
        attach.frames = ANIMATION_FRAMES;
        attach.mid_sequence = None;
        check(attach, last, now);
    }

    #[test]
    fn recent_output_wait_is_capped_by_the_existing_twenty_ms_deadline() {
        with_attachment(|attach, last, _| {
            assert!(attach.output.slow_bdp().is_none());
            assert!(attach.queue_delay() < LAG_LIMIT / 2);
            for (output_us, expected_us) in [(17_000,18_000),(18_500,19_500),(19_500,20_000),(20_000,20_000),(2_000_000,20_000)] {
                attach.output_at = Some(last + Duration::from_micros(output_us));
                assert_eq!(attach.screen_due_at(), Some(last + Duration::from_micros(expected_us)));
                assert_eq!(attach.screen_gap(), SCREEN_GAP, "global cadence unchanged");
            }
            // Repeated sub-ms pieces cannot push this logical raw deadline
            // beyond last+20ms; this is not a bound on actual poll service.
            for output_us in (18_000..2_000_000).step_by(400) {
                attach.output_at = Some(last + Duration::from_micros(output_us));
                let due = attach.screen_due_at().unwrap();
                assert!(due >= last + Duration::from_millis(18));
                assert!(due <= last + SCREEN_GAP);
            }
            attach.output_at = None;
            assert_eq!(attach.screen_due_at(), Some(last + Duration::from_millis(18)));
            attach.output_gap = Duration::from_millis(25);
            attach.output_at = Some(last + Duration::from_millis(100));
            assert_eq!(attach.screen_due_at(), Some(last + SCREEN_GAP), "already-clamped margin cannot grow");
        });
    }

    #[test]
    fn other_cadence_udp_slow_and_high_queue_paths_stay_unchanged() {
        with_attachment(|attach, last, now| {
            attach.output_at = Some(last + Duration::from_millis(200));
            attach.frames = ANIMATION_FRAMES - 1;
            assert_eq!(attach.screen_due_at(), Some(last + SCREEN_GAP));
            attach.frames = ANIMATION_FRAMES;
            attach.output_gap = Duration::from_millis(10);
            attach.addr = "127.0.0.1:12345".parse().unwrap();
            attach.output.set_lossless(false);
            assert!(!attach.modelled());
            assert_eq!(attach.screen_due_at(), Some(last + Duration::from_millis(10)));
            let sent = now - Duration::from_millis(300);
            let mut path = PathModel::new(sent);
            let pn = path.number(sent, 600, false);
            path.feedback(&proto::Feedback { sent_us: 300_000, arrivals: vec![(pn, 300_000)] }, now);
            attach.path = Some(path);
            assert!(attach.modelled());
            assert_eq!(attach.screen_due_at(), Some(last + Duration::from_millis(10)));

            attach.addr = "[fd72:7368:7463:7000::1]:12345".parse().unwrap();
            attach.path = None;
            attach.output.set_lossless(true);
            attach.output.set_path(Some(32_000.0), Some(300.0));
            attach.update_len = 1_000;
            assert!(attach.output.slow_bdp().is_some());
            assert_eq!(attach.screen_due_at(), Some(last + Duration::from_millis(200)), "known-slow result is not capped to20ms");

            attach.output = Sender::new(1);
            attach.output.set_lossless(true);
            let sent = now - Duration::from_millis(500);
            attach.output.push(b"a");
            let (offset, data) = attach.output.next_chunk(sent).unwrap();
            let end = offset + data.len() as u64;
            assert!(attach.output.on_ack(1, end, sent + Duration::from_millis(1), false));
            let sent = now - Duration::from_millis(400);
            attach.output.push(b"b");
            let (offset, data) = attach.output.next_chunk(sent).unwrap();
            let end = offset + data.len() as u64;
            assert!(attach.output.on_ack(1, end, now, false));
            assert!(attach.output.slow_bdp().is_none());
            assert!(attach.queue_delay() >= LAG_LIMIT / 2);
            let half = attach.output.srtt().unwrap() / 2;
            assert_eq!(attach.screen_due_at(), Some(last + half.clamp(attach.screen_gap(), Duration::from_millis(250))));
        });
    }

    #[test]
    fn tail_and_unsent_guards_remain_and_long_idle_is_not_atomic() {
        with_attachment(|attach, last, now| {
            attach.output_at = Some(last + Duration::from_micros(18_500));
            attach.mid_sequence = Some(last + Duration::from_millis(25));
            assert_eq!(attach.next_screen_at(), Some(last + Duration::from_millis(25) + TAIL_WAIT));
            attach.mid_sequence = None;
            attach.output_at = Some(now);
            assert!(attach.screen_due_at().unwrap() < now, "after a long idle, immediate rendering and partial-frame risk remain");
            attach.repaint_due = false;
            assert_eq!(attach.screen_due_at(), None);
            attach.repaint_due = true;
            attach.shadow = None;
            assert_eq!(attach.screen_due_at(), None);
            attach.shadow = Some(Terminal::with_history(100, 30, 0));
            attach.output.push(b"not sent");
            assert_eq!(attach.screen_due_at(), None);
        });
    }
}
