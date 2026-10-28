use crate::crypto::{Dir, Key};
use crate::forward::{self, Forwarder};
use crate::net::HostMux;
use crate::proto::{
    self, ClientStream, Receiver, ReplayWindow, Sender, ServerStream, FLAG_DETACH, FLAG_EXITED, FLAG_REDRAW, KIND_CHANNEL, KIND_KICKED,
    KIND_PING, KIND_PONG, KIND_STREAM,
};
use crate::predict::{Mode, Predictor};
use crate::pty::RawTerm;
use crate::terminal::Terminal;
use crate::wire::Wire;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Keepalive interval while nothing else is sent.
pub const HEARTBEAT: Duration = Duration::from_secs(1);
/// With no key typed and no output for `IDLE_AFTER`, the keepalive goes
/// this often (as mosh's 3 s, a little longer): an idle session costs a
/// datagram each way a second otherwise (20 idle sessions, 20 a second).
/// The silences that show the notice, ping and move port are as many
/// times longer then; a key typed wakes the session and they count from
/// it (`Wakeful`).
pub const IDLE_HEARTBEAT: Duration = Duration::from_secs(5);
const IDLE_AFTER: Duration = Duration::from_secs(10);
/// Silence after which the screen dims and the disconnected notice shows.
pub const SHOW_SILENCE: Duration = Duration::from_secs(3);
/// Silence after which the client moves to a new local port, as mosh does.
pub const PORT_HOP: Duration = Duration::from_secs(10);
/// Silence after which the client asks whether its attachment still exists.
pub const PING_SILENCE: Duration = Duration::from_secs(5);
/// Typed input kept while the server is unreachable.
const INPUT_BACKLOG: usize = 4 << 20;
/// Without a tty: detach this long after stdin closed and output went quiet.
const SCRIPT_IDLE: Duration = Duration::from_millis(1500);
const SCRIPT_LIMIT: Duration = Duration::from_secs(15);
/// An unfinished escape sequence longer than this is passed on anyway.
const HOLD_LIMIT: usize = 64 * 1024;
/// Output arriving in order is acked once this much is owed or this long
/// after the first unacked byte, whichever comes first.
const ACK_BYTES: usize = 16 * 1024;
const ACK_DELAY: Duration = Duration::from_millis(2);
/// Arrivals a packet reports at most (less than a datagram); as many
/// waiting, and one goes at once.
const ARRIVALS_MAX: usize = 48;
/// Duplicate acks sent per gap: the server retransmits after three, and
/// a few spare ones keep a lost ack from leaving it to the timeout.
const DUP_ACKS: u8 = 8;
/// Resends of output already here are acked at most this often, so a lost
/// ack is repaired without each copy reading as a duplicate ack.
const STALE_ACK_GAP: Duration = Duration::from_millis(50);

pub const DEFAULT_ESCAPE: u8 = b'~';

/// Lines for the `~l` window; it runs on its own thread.
pub type Lister = Arc<dyn Fn() -> Vec<ListRow> + Send + Sync>;

/// A row of the `~l` window; a row with a target can be picked to switch
/// to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListRow {
    pub text: String,
    pub target: Option<Target>,
}

/// What a client attaches: an rsh session, or a tmux session through the
/// rsh session joined to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// Host and session id.
    Rsh(String, u64),
    /// Host and tmux session name.
    Tmux(String, String),
}

pub struct Attached {
    pub host: String,
    pub peer: SocketAddr,
    /// The attachment's own keys, from the handshake.
    pub key: Key,
    /// The token key, for asking the server whether the attachment still
    /// exists when it has gone quiet.
    pub token_key: Key,
    pub session: u64,
    pub attach: u64,
    pub label: String,
    /// The handshake's TCP connection, for `run` to go on with.
    pub wire: std::cell::RefCell<Option<Wire>>,
}

impl Attached {
    pub fn new(host: &str, peer: SocketAddr, token_key: &Key, done: crate::net::Handshake) -> Self {
        Self {
            host: host.to_string(),
            peer,
            key: done.key,
            token_key: token_key.clone(),
            session: done.attached.session,
            attach: done.attach,
            label: done.attached.label,
            wire: std::cell::RefCell::new(done.wire),
        }
    }
}

pub enum Input {
    /// Raw keystrokes from this fd (a tty in raw mode).
    Tty(i32),
    /// Bytes streamed from a pipe or file; after its end the session
    /// detaches once output goes quiet.
    Pipe(i32),
    /// A fixed script, sent at once; the session detaches when it is done.
    Script(Vec<u8>),
    /// No input: the client stays until the session ends or a signal.
    Nothing,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Detached,
    /// `~.` in a tmux session: detached, and its rsh session is to end.
    Left,
    /// Detached to attach this instead, picked in `~l`.
    Switch(Target),
    Exited(i32),
    Kicked(String),
}

pub struct Options<'a> {
    pub size: (u16, u16),
    pub escape: Option<u8>,
    pub raw: Option<&'a RawTerm>,
    /// Local listeners (`-L`) and the channels of this session.
    pub forwarder: Option<Forwarder>,
    /// Targets the server may ask this client to connect to (`-R`).
    pub allowed_targets: Vec<String>,
    pub sessions: Option<Lister>,
    /// Counts `rsh test render` frames as they arrive for the terminal.
    pub frames: Option<Arc<std::sync::Mutex<crate::bench::FrameCount>>>,
    /// The session joins this tmux session: `~.` leaves it (`Outcome::Left`).
    pub tmux: Option<String>,
    /// OSC 52 clipboard reads reach the terminal (`clipboard_read` in
    /// `~/.rsh/config.yaml`).
    pub clipboard_read: bool,
}

static WINCH: AtomicBool = AtomicBool::new(false);
static CONT: AtomicBool = AtomicBool::new(false);
static QUIT: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(signal: libc::c_int) {
    match signal {
        libc::SIGWINCH => WINCH.store(true, Ordering::Relaxed),
        libc::SIGCONT => CONT.store(true, Ordering::Relaxed),
        _ => QUIT.store(true, Ordering::Relaxed),
    }
}

/// Signal handlers without SA_RESTART, so poll() wakes on them.
pub fn install_signal_handlers() {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = on_signal as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = 0;
        for signal in [libc::SIGWINCH, libc::SIGCONT, libc::SIGTERM, libc::SIGHUP, libc::SIGINT, libc::SIGQUIT] {
            libc::sigaction(signal, &action, std::ptr::null_mut());
        }
    }
}

struct Escape {
    key: Option<u8>,
    /// The escape key just typed, as it came in, held until the next key
    /// says whether it starts a command.
    held: Option<Vec<u8>>,
    /// Characters typed on the current line, as far as keys tell: Enter,
    /// Ctrl-C, Ctrl-U and Ctrl-W start over, Backspace takes one back. A
    /// printable escape key counts when this is 0, so `~~` and a Backspace
    /// leave `~` usable again.
    typed: usize,
    /// Inside a bracketed paste: pasted text is never a command.
    pasting: bool,
    /// The start of a sequence cut off at the end of a read, kept for the
    /// next one; sent as it is if nothing follows soon (`expire`).
    partial: Vec<u8>,
    partial_since: Option<Instant>,
}

/// How long a cut-off sequence waits for the rest.
const PARTIAL_WAIT: Duration = Duration::from_millis(50);

enum Command {
    Detach,
    Help,
    Redraw,
    List,
}

/// One piece of input, as far as the escape key cares.
#[derive(Debug, PartialEq, Eq)]
enum Unit {
    /// Something the terminal sends on its own (mouse, focus, answers to
    /// queries, key releases, a lone modifier key): it is passed on and
    /// changes nothing here.
    Report,
    /// A key, with the character it types when it types one plainly (no
    /// modifier but Shift, or Ctrl for a control character).
    Key(Option<char>),
    /// A sequence that has not ended within the bytes at hand.
    Partial,
}

impl Escape {
    fn new(key: Option<u8>) -> Self {
        Escape {
            key,
            held: None,
            typed: 0,
            pasting: false,
            partial: Vec::new(),
            partial_since: None,
        }
    }

    /// Splits keystrokes into bytes for the session and local commands.
    /// Like ssh: a printable escape character counts only at the start of
    /// a line, and it followed by anything else is sent as typed. Keys may
    /// come encoded (kitty keyboard protocol, modifyOtherKeys) when the
    /// program asked the terminal for that, and reports from the terminal
    /// (focus, mouse) may come between them.
    fn filter(&mut self, bytes: &[u8], out: &mut Vec<u8>) -> Vec<Command> {
        let mut commands = Vec::new();
        let Some(key) = self.key else {
            out.extend_from_slice(bytes);
            return commands;
        };
        let key = char::from(key);
        let mut input = std::mem::take(&mut self.partial);
        // The wait counts from the first cut, not from each new read.
        let since = self.partial_since.take();
        input.extend_from_slice(bytes);
        let mut rest = &input[..];
        while !rest.is_empty() {
            let (len, unit) = next_unit(rest);
            let raw = &rest[..len];
            rest = &rest[len..];
            if unit == Unit::Partial {
                self.partial = raw.to_vec();
                self.partial_since = Some(since.unwrap_or_else(Instant::now));
                break;
            }
            if raw == b"\x1b[200~" {
                // A held escape key belongs before the paste.
                if let Some(held) = self.held.take() {
                    out.extend_from_slice(&held);
                }
                self.pasting = true;
                out.extend_from_slice(raw);
                continue;
            }
            if raw == b"\x1b[201~" {
                // The line state is the pasted text's last character's.
                self.pasting = false;
                out.extend_from_slice(raw);
                continue;
            }
            let ch = match unit {
                Unit::Key(ch) if self.pasting => {
                    out.extend_from_slice(raw);
                    self.count(ch);
                    continue;
                }
                Unit::Key(ch) => ch,
                _ => {
                    out.extend_from_slice(raw);
                    continue;
                }
            };
            if let Some(held) = self.held.take() {
                let command = match ch {
                    Some('.') => Some(Command::Detach),
                    Some('h' | '?') => Some(Command::Help),
                    Some('r') => Some(Command::Redraw),
                    Some('l') => Some(Command::List),
                    _ => None,
                };
                if let Some(command) = command {
                    commands.push(command);
                    continue;
                }
                // Not a command: the escape key itself was typed, and then
                // this key unless it was the escape key again (`~~`).
                out.extend_from_slice(&held);
                self.count(Some(key));
                if ch != Some(key) {
                    out.extend_from_slice(raw);
                    self.count(ch);
                }
            } else if ch == Some(key) && (key < ' ' || self.typed == 0) {
                self.held = Some(raw.to_vec());
            } else {
                out.extend_from_slice(raw);
                self.count(ch);
            }
        }
        commands
    }

    fn count(&mut self, ch: Option<char>) {
        match ch {
            Some('\r' | '\n' | '\x03' | '\x15' | '\x17') => self.typed = 0,
            Some('\x7f' | '\x08') => self.typed = self.typed.saturating_sub(1),
            // One per character: UTF-8 continuation bytes come one by one.
            Some(c) if !c.is_control() && !('\u{80}'..='\u{bf}').contains(&c) => self.typed += 1,
            _ => {}
        }
    }

    /// A cut-off sequence nothing completed in time was typed as it is
    /// (Alt+[ for one): it goes to the session.
    fn expire(&mut self, now: Instant, out: &mut Vec<u8>) {
        if self.partial_since.is_some_and(|since| now.duration_since(since) >= PARTIAL_WAIT) {
            // A held escape key came first.
            if let Some(held) = self.held.take() {
                out.extend_from_slice(&held);
                self.count(self.key.map(char::from));
            }
            self.partial_since = None;
            let partial = std::mem::take(&mut self.partial);
            for &byte in &partial {
                if byte != 0x1b {
                    self.count(Some(char::from(byte)));
                }
            }
            out.extend_from_slice(&partial);
        }
    }

    /// The keys for an open window, joined with a cut-off sequence from
    /// the last read; a sequence cut again is kept for the next.
    /// Keys for an open window, and the reports among them (mouse, focus,
    /// a clipboard answer), which still go to the session. A unit cut
    /// between reads waits for the rest.
    fn window_keys(&mut self, bytes: &[u8]) -> (Vec<WindowKey>, Vec<u8>) {
        let mut input = std::mem::take(&mut self.partial);
        let since = self.partial_since.take();
        input.extend_from_slice(bytes);
        let mut keys = Vec::new();
        let mut reports = Vec::new();
        let mut rest = &input[..];
        while !rest.is_empty() {
            let (len, unit) = next_unit(rest);
            if unit == Unit::Partial {
                self.partial = rest.to_vec();
                self.partial_since = Some(since.unwrap_or_else(Instant::now));
                break;
            }
            if unit == Unit::Report {
                reports.extend_from_slice(&rest[..len]);
            } else {
                keys.push(window_key(&rest[..len], &unit));
            }
            rest = &rest[len..];
        }
        (keys, reports)
    }
}

/// The first unit of `bytes` and its length.
fn next_unit(bytes: &[u8]) -> (usize, Unit) {
    if bytes[0] != 0x1b || bytes.len() < 2 {
        return (1, Unit::Key(Some(char::from(bytes[0]))));
    }
    match bytes[1] {
        // X10 mouse: CSI M and three raw bytes.
        b'[' if bytes.get(2) == Some(&b'M') => {
            if bytes.len() >= 6 {
                (6, Unit::Report)
            } else {
                (bytes.len(), Unit::Partial)
            }
        }
        b'[' => {
            let mut end = 2;
            while end < bytes.len() && (0x30..=0x3f).contains(&bytes[end]) {
                end += 1;
            }
            let params_end = end;
            while end < bytes.len() && (0x20..=0x2f).contains(&bytes[end]) {
                end += 1;
            }
            match bytes.get(end) {
                Some(&last) if (0x40..=0x7e).contains(&last) => (end + 1, csi_unit(&bytes[2..params_end], last)),
                None => (bytes.len(), Unit::Partial),
                _ => (1, Unit::Key(None)),
            }
        }
        b'O' if bytes.len() >= 3 => (3, Unit::Key(None)),
        b'O' => (2, Unit::Partial),
        // Answers to OSC and DCS queries, up to BEL or ST.
        b']' | b'P' => (2..bytes.len())
            .find_map(|i| match bytes[i] {
                0x07 => Some(i + 1),
                0x1b if bytes.get(i + 1) == Some(&b'\\') => Some(i + 2),
                _ => None,
            })
            .map_or((bytes.len(), Unit::Partial), |end| (end, Unit::Report)),
        _ => (1, Unit::Key(Some('\x1b'))),
    }
}

fn csi_unit(params: &[u8], last: u8) -> Unit {
    let text = String::from_utf8_lossy(params);
    let first = params.first().copied();
    let mut fields = text.split(';');
    let mut keys = fields.next().unwrap_or("").split(':');
    let mut modifiers = fields.next().unwrap_or("1").split(':');
    let mods = modifiers.next().and_then(|m| m.parse::<u32>().ok()).unwrap_or(1);
    // kitty reports releases (event 3) of every key when asked to.
    if modifiers.next() == Some("3") {
        return Unit::Report;
    }
    match last {
        b'M' | b'm' if first == Some(b'<') => Unit::Report,
        b'I' | b'O' if params.is_empty() => Unit::Report,
        b'R' | b'c' | b'n' | b't' | b'y' => Unit::Report,
        _ if matches!(first, Some(b'?' | b'>' | b'=')) => Unit::Report,
        // kitty: CSI code[:shifted[:base]] ; modifiers[:event] ; text u
        b'u' => {
            let code = keys.next().and_then(|c| c.parse::<u32>().ok());
            let shifted = keys.next().and_then(|c| c.parse::<u32>().ok());
            match code {
                // Shift, Ctrl, Alt and the like pressed alone, Caps Lock, Num Lock.
                Some(57358..=57360 | 57441..=57454) => Unit::Report,
                Some(code) => Unit::Key(plain_key(code, shifted, mods)),
                None => Unit::Key(None),
            }
        }
        // xterm modifyOtherKeys: CSI 27 ; modifiers ; code ~
        b'~' if text.starts_with("27;") => {
            let mut fields = text.split(';').skip(1);
            let mods = fields.next().and_then(|m| m.parse().ok()).unwrap_or(1);
            let code = fields.next().and_then(|c| c.parse().ok());
            Unit::Key(code.and_then(|code| plain_key(code, None, mods)))
        }
        _ => Unit::Key(None),
    }
}

/// The character a key types when it types one plainly: unmodified or
/// with Shift, or a control character with Ctrl (and Shift). `mods` is
/// the protocols' 1 + bits (Shift 1, Alt 2, Ctrl 4, …); Caps and Num Lock
/// are ignored.
fn plain_key(code: u32, shifted: Option<u32>, mods: u32) -> Option<char> {
    const SHIFT: u32 = 1;
    const CTRL: u32 = 4;
    let bits = mods.saturating_sub(1) & !(64 | 128);
    match (code, bits) {
        (13 | 57414, 0 | SHIFT) => Some('\r'),
        // kitty's functional keys (keypad, F13 and up, media): no text.
        (57344..=63743, _) => None,
        // Shift+key without the shifted key reported (kitty flag 8 alone):
        // the US layout's, which is where `~` is.
        (_, SHIFT) if shifted.is_none() => {
            let base = char::from_u32(code)?;
            let us = "`1234567890-=[]\\;',./";
            let up = "~!@#$%^&*()_+{}|:\"<>?";
            Some(us.find(base).and_then(|at| up[at..].chars().next()).unwrap_or(base.to_ascii_uppercase()))
        }
        (_, 0) => char::from_u32(code),
        (_, SHIFT) => char::from_u32(shifted.unwrap_or(code)),
        (_, CTRL) | (_, 5) => {
            let base = if bits & SHIFT != 0 { shifted.unwrap_or(code) } else { code };
            let control = match char::from_u32(base)? {
                c @ '@'..='\x7f' => (c as u8) & 0x1f,
                '2' | ' ' => 0,
                '3' => 0x1b,
                '4' => 0x1c,
                '5' => 0x1d,
                '6' => 0x1e,
                '7' | '-' | '/' => 0x1f,
                _ => return None,
            };
            Some(char::from(control))
        }
        _ => None,
    }
}

pub fn escape_name(key: u8) -> String {
    if key < 0x20 {
        format!("Ctrl-{}", (key + 0x40) as char)
    } else {
        (key as char).to_string()
    }
}

struct Link {
    window: ReplayWindow,
    wire: Wire,
    key: Key,
    attach: u64,
    last_sent: Instant,
    /// Since when and how many output bytes arrived that no packet acked yet.
    owed: Option<(Instant, usize)>,
    /// The output ack (generation, offset) the last packet carried.
    acked: (u32, u64),
    /// Numbered datagrams that arrived since the last packet, and when
    /// (microseconds since `clock`), for the server's path model.
    arrivals: Vec<(u32, u32)>,
    clock: Instant,
}

impl Link {
    fn send(&mut self, stream: &ClientStream, now: Instant) {
        // A datagram's worth at most: few beside a chunk of input, the rest
        // with the next packet.
        let most = if stream.data.len() > 600 { 8 } else { ARRIVALS_MAX };
        let report = if self.arrivals.is_empty() {
            Vec::new()
        } else {
            let sent_us = self.micros(Instant::now());
            let arrivals = self.arrivals.drain(..self.arrivals.len().min(most)).collect();
            proto::Feedback { sent_us, arrivals }.encode()
        };
        let stream = ClientStream { feedback: &report, ..stream.clone() };
        let bytes = proto::seal(&self.key, Dir::ToServer, KIND_STREAM, self.attach, &stream.encode());
        let _ = self.wire.send(&bytes);
        self.last_sent = now;
        self.owed = None;
        self.acked = (stream.out_gen, stream.out_ack);
    }

    fn micros(&self, at: Instant) -> u32 {
        at.saturating_duration_since(self.clock).as_micros() as u32
    }

    fn arrived(&mut self, pn: u32, at: Instant) {
        let at = self.micros(at);
        self.arrivals.push((pn, at));
    }
}

pub fn run(mux: &mut HostMux, attached: &Attached, input: Input, screen: &mut dyn Write, mut opts: Options) -> io::Result<Outcome> {
    let mut out = Paced::new(screen, matches!(input, Input::Tty(_)));
    let mut forwarder = opts.forwarder.take().unwrap_or_else(|| Forwarder::new(true));
    let allowed = std::mem::take(&mut opts.allowed_targets);
    let wire = match attached.wire.borrow_mut().take() {
        Some(wire) => wire,
        None => Wire::udp(mux.socket_for(&attached.host, attached.peer)?, attached.peer)?,
    };
    let start = Instant::now();
    let mut link = Link {
        window: ReplayWindow::new(),
        wire,
        key: attached.key.clone(),
        attach: attached.attach,
        last_sent: start - HEARTBEAT,
        owed: None,
        acked: (0, 0),
        arrivals: Vec::new(),
        clock: start,
    };
    let mut ping: Option<[u8; 24]> = None;
    let mut last_ping = start;
    let tty = matches!(input, Input::Tty(_));
    let mut size = opts.size;
    let mut mirror = Terminal::with_history(size.0, size.1, 0);
    // Input stays resent on its timeout over TCP too: the server takes only
    // what the pty has room for and drops the rest of a datagram.
    let mut tx = Sender::new(0);
    let mut rx = Receiver::new(0);
    let mut escape = Escape::new(if tty { opts.escape } else { None });
    let host = attached.host.rsplit('@').next().unwrap_or(&attached.host);
    let mut decor = Decor {
        badge: tty.then(|| format!(" {}:{host} ", crate::cli::printable(&attached.label))),
        armed: false,
        selected: None,
        window: None,
        alert: None,
        drawn: Vec::new(),
    };
    let mut listing: Option<std::sync::mpsc::Receiver<Vec<ListRow>>> = None;
    // What this client is on, as `~l` rows name it.
    let here = match &opts.tmux {
        Some(name) => Target::Tmux(attached.host.clone(), name.clone()),
        None => Target::Rsh(attached.host.clone(), attached.session),
    };
    let mut list_rows: Vec<ListRow> = Vec::new();
    let mut last_heard = start;
    let mut last_hop = start;
    let mut stale_acked = start - STALE_ACK_GAP;
    // Duplicate acks sent for the gap at (generation, offset): three start
    // a fast retransmit; a few more cover lost ones.
    let mut dup_acked = (0u32, 0u64, 0u8);
    let mut last_output = start;
    let mut wake = Wakeful::new(start);
    let mut alert_drawn = start;
    let mut sync_since: Option<Instant> = None;
    let mut flags = 0u8;
    let mut redraw_wanted: Option<u32> = None;
    let mut force_send = false;
    let mut exit: Option<(i32, u32, u64)> = None;
    let mut stdin_eof: Option<Instant> = None;
    let mut behind = true;
    let mut updates = false;
    let mut predictor = Predictor::new(if tty { Mode::from_env() } else { Mode::Never });
    let mut repaint = false;
    let mut held: Vec<u8> = Vec::new();
    // The generation under way when it is one deflate stream, and its inflater.
    let mut zgen: Option<(u32, proto::Inflater)> = None;
    let mut local_ground = true;
    let mut buf = vec![0u8; 65536];
    if let Input::Script(script) = &input {
        tx.push(script);
        stdin_eof = Some(start);
    }
    let stdin_fd = match input {
        Input::Tty(fd) | Input::Pipe(fd) => Some(fd),
        Input::Script(_) | Input::Nothing => None,
    };
    loop {
        let now = Instant::now();
        let mut wait = wake.heartbeat(now).saturating_sub(now.duration_since(link.last_sent));
        if let Some(due) = out.due() {
            wait = wait.min(due.saturating_duration_since(now));
        }
        if let Some(deadline) = tx.deadline() {
            wait = wait.min(deadline.saturating_duration_since(now));
        }
        if let Some((since, _)) = link.owed {
            wait = wait.min((since + ACK_DELAY).saturating_duration_since(now));
        }
        if decor.alert.is_some() || wake.silence(last_heard, now) >= wake.scaled(SHOW_SILENCE, now) / 2 {
            wait = wait.min(Duration::from_millis(250));
        }
        if stdin_eof.is_some() {
            wait = wait.min(Duration::from_millis(50));
        }
        if listing.is_some() {
            wait = wait.min(Duration::from_millis(100));
        }
        if escape.partial_since.is_some() {
            wait = wait.min(PARTIAL_WAIT);
        }
        if let Some(deadline) = forwarder.deadline() {
            wait = wait.min(deadline.saturating_duration_since(now));
        }
        if forwarder.needs_timer() {
            wait = wait.min(Duration::from_millis(100));
        }
        if let Some(at) = link.wire.next_service(now) {
            wait = wait.min(at.saturating_duration_since(now));
        }
        let mut fds = vec![
            link.wire.pollfd(),
            libc::pollfd {
                fd: stdin_fd
                    .filter(|_| stdin_eof.is_none() && tx.pending() < INPUT_BACKLOG)
                    .unwrap_or(-1),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        forwarder.pollfds(&mut fds);
        unsafe {
            libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, wait.as_millis().max(1) as i32);
        }
        let now = Instant::now();
        if QUIT.swap(false, Ordering::Relaxed) {
            detach(&mut link, &rx, size, now);
            finish(&mut out, &mirror, &mut predictor, &mut decor)?;
            return Ok(Outcome::Detached);
        }
        if tty && WINCH.swap(false, Ordering::Relaxed) {
            let next = crate::pty::window_size(stdin_fd.unwrap_or(0));
            if next != size {
                size = next;
                // The terminal moved what rsh drew; a repaint clears it.
                if !decor.drawn.is_empty() {
                    decor.drawn.clear();
                    redraw_wanted = Some(rx.gen);
                }
                mirror.resize(size.0, size.1);
                force_send = true;
            }
        }
        if CONT.swap(false, Ordering::Relaxed) {
            if let Some(raw) = opts.raw {
                let _ = raw.reenter();
            }
            redraw_wanted = Some(rx.gen);
        }

        let mut wrote = false;
        if link.wire.service(now) {
            // The server takes the new connection at the first packet; what
            // went into the old one goes again.
            force_send = true;
            tx.rewind();
        }
        while let Some((n, arrived)) = link.wire.recv(&mut buf) {
            if proto::peek(&buf[..n]).is_some_and(|(kind, _)| kind == KIND_PONG) {
                // The server's answer to a PING, sealed with the token key.
                let Some(packet) = proto::open(&attached.token_key, Dir::ToClient, &buf[..n]) else {
                    continue;
                };
                let mut reader = proto::Reader::new(&packet.payload);
                let echo = reader.bytes_fixed(24);
                if ping.is_some_and(|nonce| echo == Some(&nonce[..])) && reader.u8() == Some(0) {
                    finish(&mut out, &mirror, &mut predictor, &mut decor)?;
                    return Ok(Outcome::Kicked(
                        "the server no longer has this session (it ended, another client took it, or the server restarted)"
                            .to_string(),
                    ));
                }
                continue;
            }
            let Some(packet) = proto::open(&link.key, Dir::ToClient, &buf[..n]) else {
                continue;
            };
            if packet.session != link.attach {
                continue;
            }
            if packet.kind == KIND_KICKED {
                finish(&mut out, &mirror, &mut predictor, &mut decor)?;
                return Ok(Outcome::Kicked(proto::Reader::new(&packet.payload).str().unwrap_or_default()));
            }
            if packet.kind == KIND_CHANNEL {
                if let Some(msg) = forward::Message::decode(&packet.payload) {
                    if link.window.check(&packet.nonce) {
                        last_heard = now;
                        forwarder.receive(&msg, now, &|target| allowed.iter().any(|t| t == target));
                    }
                }
                continue;
            }
            if packet.kind != KIND_STREAM {
                continue;
            }
            let Some(stream) = ServerStream::decode(&packet.payload) else {
                continue;
            };
            if !link.window.check(&packet.nonce) {
                continue;
            }
            last_heard = now;
            if let Some(pn) = stream.pn {
                link.arrived(pn, arrived);
                if link.arrivals.len() >= ARRIVALS_MAX {
                    force_send = true;
                }
            }
            // A compressed datagram stands for more stream bytes than it holds.
            let inflated;
            let data: &[u8] = if stream.flags & proto::FLAG_DEFLATE != 0 {
                match proto::inflate(stream.data) {
                    Some(bytes) => {
                        inflated = bytes;
                        &inflated
                    }
                    None => continue,
                }
            } else {
                stream.data
            };
            tx.on_ack(0, stream.in_ack, now, !data.is_empty());
            let gen = rx.gen;
            if !data.is_empty() {
                if let Some(count) = &opts.frames {
                    if let Ok(mut count) = count.lock() {
                        count.packet(stream.out_gen, stream.out_offset, data.len(), rx.gen, rx.next, now);
                    }
                }
                let (same, before, ahead) = (stream.out_gen == rx.gen, rx.next, rx.held());
                let new = rx.accept(stream.out_gen, stream.out_offset, data, usize::MAX);
                // A generation sent as one deflate stream is inflated in order.
                let new = if stream.flags & proto::FLAG_ZSTREAM != 0 && stream.out_gen == rx.gen && !new.is_empty() {
                    if zgen.as_ref().map(|(gen, _)| *gen) != Some(rx.gen) {
                        zgen = Some((rx.gen, proto::Inflater::default()));
                    }
                    match zgen.as_mut().and_then(|(_, z)| z.push(&new)) {
                        Some(bytes) => bytes,
                        None => {
                            // Broken: a repaint starts a new generation.
                            redraw_wanted = Some(rx.gen);
                            Vec::new()
                        }
                    }
                } else {
                    new
                };
                if same && stream.out_offset > before {
                    // Past a gap: acked at once, so the duplicate acks start
                    // a fast retransmit of the hole. An ack the server has
                    // not had yet (a delayed one) moves it on, not counted.
                    if (dup_acked.0, dup_acked.1) != (rx.gen, rx.next) {
                        dup_acked = (rx.gen, rx.next, 0);
                    }
                    // A second copy of what is held already says nothing new.
                    if dup_acked.2 < DUP_ACKS && rx.held() != ahead {
                        if link.acked == (rx.gen, rx.next) {
                            dup_acked.2 += 1;
                        }
                        link.send(&client_stream(&rx, size, 0, tx.end(), &[]), now);
                    }
                } else if rx.gen == gen && rx.next == before {
                    // A resend of what is here already: acked (seldom) only
                    // when the server may not know, as an ack can be lost.
                    if link.acked != (rx.gen, rx.next) && now.duration_since(stale_acked) >= STALE_ACK_GAP {
                        stale_acked = now;
                        force_send = true;
                    }
                } else if rx.gen == gen && rx.next - before > data.len() as u64 {
                    // A hole filled: the sender is waiting to hear it.
                    force_send = true;
                } else {
                    let (since, bytes) = link.owed.unwrap_or((now, 0));
                    link.owed = Some((since, bytes + data.len()));
                }
                if !new.is_empty() {
                    out.write_all(&decor.undo(&mirror))?;
                    out.write_all(&predictor.undo(&mirror))?;
                    // Only whole sequences reach the tty; a split tail waits
                    // for the next datagram, so the notice, badge and guesses
                    // are never written into the middle of one.
                    let cut = mirror.feed_to_ground(&new);
                    if cut > 0 {
                        let mut chunk = std::mem::take(&mut held);
                        chunk.extend_from_slice(&new[..cut]);
                        if let Some(count) = &opts.frames {
                            chunk = count.lock().map(|mut count| count.take(&chunk, now)).unwrap_or(chunk);
                        }
                        out.write_all(&strip_queries(&chunk, opts.clipboard_read))?;
                        held.clear();
                        held.extend_from_slice(&new[cut..]);
                        local_ground = true;
                    } else {
                        held.extend_from_slice(&new);
                    }
                    if held.len() > HOLD_LIMIT {
                        out.write_all(&strip_queries(&held, opts.clipboard_read))?;
                        held.clear();
                        local_ground = mirror.at_ground();
                    }
                    wrote = true;
                    repaint = true;
                    last_output = now;
                    wake.active(now);
                }
            }
            if rx.gen != gen {
                force_send = true;
            }
            if stream.out_gen == rx.gen {
                behind = rx.next < stream.end_offset;
                updates = stream.flags & proto::FLAG_UPDATES != 0;
            }
            if stream.flags & FLAG_EXITED != 0 {
                exit = Some((stream.exit_code, stream.out_gen, stream.end_offset));
            }
        }

        if fds[1].revents != 0 {
            if let Some(fd) = stdin_fd {
                let room = INPUT_BACKLOG.saturating_sub(tx.pending()).min(16384);
                let n = if room == 0 { -2 } else { unsafe { libc::read(fd, buf.as_mut_ptr().cast(), room) } };
                let gone = n == 0 || (n == -1 && fds[1].revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0);
                if gone && !tty {
                    stdin_eof = Some(now);
                } else if gone {
                    detach(&mut link, &rx, size, now);
                    finish(&mut out, &mirror, &mut predictor, &mut decor)?;
                    return Ok(Outcome::Detached);
                }
                if n > 0 && decor.window.is_some() {
                    // Keys go to the window, not the session; reports the
                    // terminal sends (mouse, focus, answers) still go on.
                    let (mut keys, reports) = escape.window_keys(&buf[..n as usize]);
                    if !reports.is_empty() && tx.pending() + reports.len() <= INPUT_BACKLOG {
                        tx.push(&reports);
                    }
                    if decor.selected.is_none() && !keys.is_empty() {
                        // A window one only reads: any key closes it.
                        keys = vec![WindowKey::Close];
                    }
                    for key in keys {
                    if decor.window.is_none() {
                        break;
                    }
                    let picked = decor.selected.and_then(|line| list_rows.get(line.wrapping_sub(2))).and_then(|row| row.target.clone());
                    match key {
                        WindowKey::Up | WindowKey::Down => {
                            let step = |line: usize| if key == WindowKey::Up { line.checked_sub(1) } else { Some(line + 1) };
                            let mut line = decor.selected.unwrap_or(2);
                            while let Some(next) = step(line).filter(|next| *next >= 2 && *next < list_rows.len() + 2) {
                                line = next;
                                if list_rows[line - 2].target.is_some() {
                                    out.write_all(&decor.undo(&mirror))?;
                                    decor.selected = Some(line);
                                    wrote = true;
                                    break;
                                }
                            }
                        }
                        WindowKey::Enter if picked.as_ref().is_some_and(|target| *target != here) => {
                            detach(&mut link, &rx, size, now);
                            finish(&mut out, &mirror, &mut predictor, &mut decor)?;
                            return Ok(Outcome::Switch(picked.expect("picked")));
                        }
                        WindowKey::Enter | WindowKey::Close => {
                            out.write_all(&decor.undo(&mirror))?;
                            decor.window = None;
                            decor.selected = None;
                            list_rows.clear();
                            listing = None;
                            wrote = true;
                        }
                        WindowKey::Other => {}
                    }
                    }
                } else if n > 0 {
                    let mut keys = Vec::with_capacity(n as usize);
                    let commands = escape.filter(&buf[..n as usize], &mut keys);
                    if decor.armed != escape.held.is_some() {
                        out.write_all(&decor.undo(&mirror))?;
                        decor.armed = escape.held.is_some();
                        wrote = true;
                    }
                    if tx.pending() + keys.len() <= INPUT_BACKLOG {
                        let start = tx.end();
                        tx.push(&keys);
                        predictor.typed(&keys, start, &mirror, now);
                        repaint = true;
                    }
                    if !keys.is_empty() {
                        wake.active(now);
                    }
                    for command in commands {
                    match command {
                        Command::Detach => {
                            detach(&mut link, &rx, size, now);
                            finish(&mut out, &mirror, &mut predictor, &mut decor)?;
                            return Ok(if opts.tmux.is_some() { Outcome::Left } else { Outcome::Detached });
                        }
                        Command::Redraw => redraw_wanted = Some(rx.gen),
                        Command::Help => {
                            out.write_all(&decor.undo(&mirror))?;
                            decor.window = Some(help_lines(escape.key.unwrap_or(DEFAULT_ESCAPE), &attached.host, &attached.label, opts.tmux.is_some()));
                            decor.selected = None;
                            list_rows.clear();
                            wrote = true;
                        }
                        Command::List => {
                            out.write_all(&decor.undo(&mirror))?;
                            let mut lines = vec!["rsh sessions".to_string(), String::new()];
                            if let Some(lister) = opts.sessions.clone() {
                                let (send, receive) = std::sync::mpsc::channel();
                                std::thread::spawn(move || {
                                    let _ = send.send(lister());
                                });
                                listing = Some(receive);
                                lines.push("asking every host...".to_string());
                            } else {
                                lines.push("not available here".to_string());
                            }
                            decor.window = Some(lines);
                            wrote = true;
                        }
                    }
                    }
                }
            }
        }
        if let Some(receive) = &listing {
            let answer = match receive.try_recv() {
                Ok(found) => Some(found),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(vec![ListRow {
                    text: "could not list sessions".to_string(),
                    target: None,
                }]),
            };
            if let Some(found) = answer {
                listing = None;
                out.write_all(&decor.undo(&mirror))?;
                let mut lines = vec!["rsh sessions".to_string(), String::new()];
                lines.extend(found.iter().map(|row| row.text.clone()));
                // The highlight starts on this session, else the first one.
                let start = found
                    .iter()
                    .position(|row| row.target.as_ref() == Some(&here))
                    .or_else(|| found.iter().position(|row| row.target.is_some()));
                decor.selected = start.map(|index| index + 2);
                list_rows = found;
                decor.window = Some(lines);
                wrote = true;
            }
        }
        // The start of a sequence nothing completed is typed text after all.
        let mut late = Vec::new();
        escape.expire(Instant::now(), &mut late);
        if !late.is_empty() && tx.pending() + late.len() <= INPUT_BACKLOG {
            let start = tx.end();
            tx.push(&late);
            predictor.typed(&late, start, &mirror, Instant::now());
            repaint = true;
        }

        let now = Instant::now();
        if link.owed.is_some_and(|(since, bytes)| bytes >= ACK_BYTES || now.duration_since(since) >= ACK_DELAY) {
            force_send = true;
        }
        // Any packet heard, stream or forwarding, ends the notice.
        if decor.alert.is_some() && local_ground && now.duration_since(last_heard) < wake.scaled(SHOW_SILENCE, now) {
            out.write_all(&decor.undo(&mirror))?;
            decor.alert = None;
            wrote = true;
            repaint = true;
        }
        // A repaint request rides on every packet until the new generation
        // it asks for arrives, so a lost datagram does not lose it.
        if redraw_wanted.is_some_and(|gen| gen != rx.gen) {
            redraw_wanted = None;
        }
        if redraw_wanted.is_some() {
            flags |= FLAG_REDRAW;
        }
        let mut sent = false;
        while let Some((offset, data)) = tx.next_chunk(now) {
            let stream = client_stream(&rx, size, flags, offset, &data);
            link.send(&stream, now);
            flags = 0;
            sent = true;
        }
        if !sent && (force_send || flags != 0 || now.duration_since(link.last_sent) >= wake.heartbeat(now)) {
            // Repeating an ack the server has is no loss signal: say so.
            let repeat = if link.acked == (rx.gen, rx.next) { proto::FLAG_REPEAT } else { 0 };
            let stream = client_stream(&rx, size, flags | repeat, tx.end(), &[]);
            link.send(&stream, now);
            flags = 0;
        }
        force_send = false;
        if !forwarder.is_empty() {
            forwarder.mark_ready(&fds);
            forwarder.service(now);
            for payload in forwarder.outgoing(now) {
                let bytes = proto::seal(&link.key, Dir::ToServer, KIND_CHANNEL, link.attach, &payload);
                let _ = link.wire.send(&bytes);
            }
        }

        if let Some((code, gen, end)) = exit {
            if rx.gen == gen && rx.next >= end {
                detach(&mut link, &rx, size, now);
                finish(&mut out, &mirror, &mut predictor, &mut decor)?;
                return Ok(Outcome::Exited(code));
            }
        }

        // While the notice is up the session is not idle: its silences
        // keep their active lengths (the notice is not taken down by the
        // longer idle ones).
        if decor.alert.is_some() {
            wake.active(now);
        }
        let silence = wake.silence(last_heard, now);
        // Nothing heard for a while: the screen dims and a notice in the
        // middle says so, updated every second, until the server answers.
        if tty && silence >= wake.scaled(SHOW_SILENCE, now) && local_ground
            && (decor.alert.is_none() || now.duration_since(alert_drawn) >= Duration::from_secs(1)) {
                let mut lines = vec![
                    "[ Session disconnected ]".to_string(),
                    String::new(),
                    format!("Last contact {} ago; reconnecting.", human(now.duration_since(last_heard))),
                ];
                lines.push("Keys typed now are sent once it is back.".to_string());
                if let Some(key) = opts.escape {
                    let at = if key < 0x20 { "" } else { " at the start of a line" };
                    lines.push(format!("{}.{at} detaches.", escape_name(key)));
                }
                if decor.alert.is_some() && !decor.drawn.is_empty() {
                    out.write_all(&decor.update_notice(&mirror, lines))?;
                } else {
                    // Guesses are not shown over the dimmed screen.
                    out.write_all(&predictor.undo(&mirror))?;
                    out.write_all(&decor.undo(&mirror))?;
                    decor.alert = Some(lines);
                }
                alert_drawn = now;
                wrote = true;
            }
        if silence >= wake.scaled(PING_SILENCE, now) && now.duration_since(last_ping) >= PING_SILENCE {
            // Only the token key is sure to work with a restarted server.
            let payload = proto::Writer::new().u64(crate::server::now_ms()).u64(link.attach).finish();
            let bytes = proto::seal(&attached.token_key, Dir::ToServer, KIND_PING, attached.token_key.id(), &payload);
            ping = bytes[13..proto::HEADER_LEN].try_into().ok();
            let _ = link.wire.send(&bytes);
            last_ping = now;
        }
        if silence >= wake.scaled(PORT_HOP, now) && now.duration_since(last_hop) >= PORT_HOP {
            if link.wire.is_tcp() {
                link.wire.reset(now);
            } else if let Ok(wire) = mux.rebind_for(&attached.host, attached.peer).and_then(|sock| Wire::udp(sock, attached.peer)) {
                link.wire = wire;
            }
            last_hop = now;
            tx.rewind();
        }
        predictor.reconcile(&mirror, tx.acked(), tx.srtt(), now);
        // What rsh draws over the screen is put back only when a token is
        // there: in between, the program's output would only cover it again.
        // (While the rest of a frame is awaited it joins that write.)
        let frame = out.ready(now);
        if frame && local_ground && decor.drawn.is_empty() {
            let bytes = decor.draw(&mirror);
            if !bytes.is_empty() {
                out.write_all(&bytes)?;
                // Drawing ends at the program's cursor; guesses go back on top.
                repaint = true;
                wrote = true;
            }
        }
        if frame && local_ground && decor.alert.is_none() && (repaint || predictor.drawn() || predictor.pending()) {
            out.write_all(&predictor.undo(&mirror))?;
            out.write_all(&predictor.draw(&mirror, tx.srtt(), now))?;
            repaint = false;
            wrote = true;
        }
        let _ = wrote;
        // A frame the program left open is kept open for a while, as a
        // terminal would; past that rsh ends it (a program killed mid-frame
        // must not hold every later update back).
        if mirror.synchronized() {
            let since = *sync_since.get_or_insert(now);
            out.hold = now.duration_since(since) < SYNC_HOLD;
        } else {
            sync_since = None;
            out.hold = false;
        }
        out.ground = local_ground;
        out.behind = behind;
        out.updates = updates;
        out.flush_paced(now)?;
        if let Some(eof) = stdin_eof {
            let quiet = now.duration_since(last_output) >= SCRIPT_IDLE && now.duration_since(eof) >= SCRIPT_IDLE;
            if (tx.pending() == 0 && !behind && quiet) || now.duration_since(eof) >= SCRIPT_LIMIT {
                detach(&mut link, &rx, size, now);
                return Ok(Outcome::Detached);
            }
        }
    }
}

fn client_stream<'a>(rx: &Receiver, size: (u16, u16), flags: u8, offset: u64, data: &'a [u8]) -> ClientStream<'a> {
    ClientStream {
        flags,
        cols: size.0,
        rows: size.1,
        out_gen: rx.gen,
        out_ack: rx.next,
        in_offset: offset,
        feedback: &[],
        data,
    }
}

fn detach(link: &mut Link, rx: &Receiver, size: (u16, u16), now: Instant) {
    let stream = client_stream(rx, size, FLAG_DETACH, 0, &[]);
    for _ in 0..3 {
        link.send(&stream, now);
    }
}

/// Whether the session is idle (no key typed, no output for `IDLE_AFTER`):
/// then the keepalive is `IDLE_HEARTBEAT`, and silences count as many times
/// longer. Waking (a key, output) counts silence from then: the last reply
/// heard while idle may be seconds old, which is no silence yet.
struct Wakeful {
    active: Instant,
    woke: Instant,
}

impl Wakeful {
    fn new(now: Instant) -> Self {
        Wakeful { active: now, woke: now }
    }

    fn idle(&self, now: Instant) -> bool {
        now.duration_since(self.active) >= IDLE_AFTER
    }

    fn active(&mut self, now: Instant) {
        if self.idle(now) {
            self.woke = now;
        }
        self.active = now;
    }

    fn heartbeat(&self, now: Instant) -> Duration {
        if self.idle(now) { IDLE_HEARTBEAT } else { HEARTBEAT }
    }

    /// `limit` as long as it is while active, or idle.
    fn scaled(&self, limit: Duration, now: Instant) -> Duration {
        if self.idle(now) { limit * (IDLE_HEARTBEAT.as_millis() / HEARTBEAT.as_millis()) as u32 } else { limit }
    }

    fn silence(&self, heard: Instant, now: Instant) -> Duration {
        now.duration_since(heard.max(self.woke))
    }
}

/// Gathered output written without waiting for the frame.
const PACED_MAX: usize = 64 * 1024;

/// Output to the terminal. Writes are paced by a token bucket with a burst
/// of 2, refilled a little faster than the output's frames come (from
/// about 120 a second down to 33), so frames go out as they come (jitter
/// does not make them wait); output that comes without steps (a flood)
/// keeps the rate it had, 33 a second after a pause. Output the server has sent
/// more of waits for the rest (at most `CATCH_UP`), so a frame split over
/// datagrams is one write. Each write is one synchronized
/// update (`?2026h` … `?2026l`): what rsh takes off the screen and puts
/// back (header, guesses) around the program's output shows as one
/// picture. The program's own synchronized updates are folded into these;
/// one it has not ended yet stays open past the write.
struct Paced<'a> {
    inner: &'a mut dyn Write,
    pending: Vec<u8>,
    tokens: f64,
    filled: Instant,
    /// The program is inside a synchronized update of its own.
    pub hold: bool,
    /// The bytes end where no sequence is half done; a write that does not
    /// (a long image or clipboard string cut by `HOLD_LIMIT`) goes out as
    /// it is, for marks inside it would end the string.
    pub ground: bool,
    /// A terminal, not a pipe or file: only then are writes wrapped.
    terminal: bool,
    /// The last write's buffer, kept for the next.
    update: Vec<u8>,
    /// The server has sent more than has arrived (`end_offset`): the rest of
    /// what is gathered, e.g. a frame split over datagrams, is on its way.
    pub behind: bool,
    /// When the gathered output started.
    since: Option<Instant>,
    /// How often a token comes (see `MIN_REFILL`).
    every: Duration,
    /// When output last came after a step (not within a frame).
    arrived: Option<Instant>,
    /// What is gathered clears the screen (a repaint).
    clears: bool,
    /// The server sends screen updates (`FLAG_UPDATES`): each goes out whole.
    pub updates: bool,
}

const BURST: f64 = 2.0;
/// Tokens come a little faster than 30 a second: with 30 fps output, a token
/// spent on anything else (the header, guesses) would otherwise never be
/// made up, and every frame would wait for the next token from then on.
/// Output that comes in steps faster than that (a program drawing at 60
/// fps) brings them down to 0.9 of its step, to `MIN_REFILL` at most.
const REFILL: Duration = Duration::from_millis(30);
const MIN_REFILL: Duration = Duration::from_millis(8);
/// Output this far apart is a new step (a frame), not more of one.
const STEP: Duration = Duration::from_millis(4);
/// How long gathered output waits for the rest the server has sent, so a
/// frame goes out whole and costs one write, not two.
const CATCH_UP: Duration = Duration::from_millis(10);
/// The longest a repaint waits for its rest (see `Paced::catch_up`).
const REPAINT_WAIT: Duration = Duration::from_millis(300);
/// The longest rsh keeps a program's synchronized update open.
const SYNC_HOLD: Duration = Duration::from_millis(150);

impl<'a> Paced<'a> {
    fn new(inner: &'a mut dyn Write, terminal: bool) -> Self {
        Paced { inner, pending: Vec::new(), tokens: BURST, filled: Instant::now(), hold: false, ground: true, terminal, update: Vec::new(), behind: false, since: None, every: REFILL, arrived: None, clears: false, updates: false }
    }

    fn refill(&mut self, now: Instant) {
        let rate = 1.0 / self.every.as_secs_f64();
        self.tokens = (self.tokens + now.duration_since(self.filled).as_secs_f64() * rate).min(BURST);
        self.filled = now;
    }

    /// Waiting a moment for the rest of what the server sent.
    fn catching_up(&self, now: Instant) -> bool {
        self.behind && (self.pending.len() < PACED_MAX || self.clears || self.updates) && self.since.is_some_and(|since| now.duration_since(since) < self.catch_up())
    }

    /// How long gathered output waits for the rest the server sent: a
    /// repaint (it clears the screen) or a screen update longer, as shown
    /// in part it flashes or tears (a frame waits for a resend instead).
    fn catch_up(&self) -> Duration {
        if self.clears || self.updates {
            REPAINT_WAIT
        } else {
            CATCH_UP
        }
    }

    /// Writes what is gathered when a write is allowed.
    fn flush_paced(&mut self, now: Instant) -> io::Result<()> {
        self.refill(now);
        // A large heap goes out anyway: one big write would keep the loop
        // from reading datagrams while the terminal takes it.
        if !self.pending.is_empty() && !self.catching_up(now) && (self.tokens >= 1.0 || self.pending.len() >= PACED_MAX) {
            self.tokens = (self.tokens - 1.0).max(0.0);
            self.write_out()?;
        }
        Ok(())
    }

    /// A write would go out now, or nothing is waiting.
    fn ready(&mut self, now: Instant) -> bool {
        self.refill(now);
        self.pending.is_empty() || self.tokens >= 1.0 || self.pending.len() >= PACED_MAX
    }

    /// When gathered output may go.
    fn due(&self) -> Option<Instant> {
        let token = self.filled + self.every.mul_f64((1.0 - self.tokens).max(0.0));
        let rest = self.since.filter(|_| self.behind).map(|since| since + self.catch_up());
        (!self.pending.is_empty()).then(|| rest.map_or(token, |rest| rest.max(token)))
    }

    /// The output's steps set how often a write may go; a frame split over
    /// datagrams (`behind`) is one step, a pause starts over.
    fn arrive(&mut self, now: Instant) {
        if self.behind {
            return;
        }
        if let Some(gap) = self.arrived.map(|at| now.duration_since(at)) {
            if gap >= Duration::from_secs(1) {
                self.every = REFILL;
            } else if gap >= STEP {
                self.every = ((self.every * 3 + gap.mul_f64(0.9)) / 4).clamp(MIN_REFILL, REFILL);
            }
        }
        self.arrived = Some(now);
    }

    fn write_out(&mut self) -> io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        if self.terminal && self.ground {
            let mut update = std::mem::take(&mut self.update);
            update.clear();
            update.extend_from_slice(SYNC_BEGIN);
            without_sync(&self.pending, &mut update);
            if !self.hold {
                update.extend_from_slice(SYNC_END);
            }
            self.inner.write_all(&update)?;
            self.update = update;
        } else {
            self.inner.write_all(&self.pending)?;
        }
        self.pending.clear();
        self.since = None;
        self.clears = false;
        self.inner.flush()
    }
}

const SYNC_BEGIN: &[u8] = b"\x1b[?2026h";
const SYNC_END: &[u8] = b"\x1b[?2026l";

/// Appends `bytes` without the program's own synchronized update marks,
/// copying the runs between ESC bytes whole.
fn without_sync(bytes: &[u8], out: &mut Vec<u8>) {
    let mut rest = bytes;
    while let Some(at) = rest.iter().position(|byte| *byte == 0x1b) {
        out.extend_from_slice(&rest[..at]);
        rest = &rest[at..];
        if rest.starts_with(SYNC_BEGIN) || rest.starts_with(SYNC_END) {
            rest = &rest[SYNC_BEGIN.len()..];
        } else {
            out.push(0x1b);
            rest = &rest[1..];
        }
    }
    out.extend_from_slice(rest);
}

impl Write for Paced<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.since.is_none() && !buf.is_empty() {
            self.since = Some(Instant::now());
        }
        if !buf.is_empty() {
            self.arrive(Instant::now());
        }
        if buf.windows(4).any(|w| w == b"\x1b[2J") {
            self.clears = true;
        }
        self.pending.extend_from_slice(buf);
        Ok(buf.len())
    }

    /// Writes everything now.
    fn flush(&mut self) -> io::Result<()> {
        self.write_out()?;
        self.inner.flush()
    }
}

impl Drop for Paced<'_> {
    fn drop(&mut self) {
        self.hold = false;
        let _ = self.flush();
    }
}

/// Hands the tty back: takes off what rsh drew and turns off every mode the
/// remote program left on, so the local shell gets a sane terminal.
fn finish(out: &mut Paced, mirror: &Terminal, predictor: &mut Predictor, decor: &mut Decor) -> io::Result<()> {
    // Nothing stays held when the terminal is handed back.
    out.hold = false;
    // CAN ends any sequence the tty might still be inside; it is ignored otherwise.
    out.write_all(b"\x18")?;
    out.write_all(&decor.undo(mirror))?;
    out.write_all(&predictor.undo(mirror))?;
    out.write_all(&mirror.reset_sequence())?;
    out.flush()
}

/// Drops requests that make the local terminal type its own data back to
/// the remote program: window/icon title reports (CSI 20 t, 21 t) and,
/// unless `clipboard_read`, OSC 52 clipboard reads (which hand the local
/// clipboard to the host). Everything else, including clipboard writes,
/// passes unchanged.
pub fn strip_queries(bytes: &[u8], clipboard_read: bool) -> std::borrow::Cow<'_, [u8]> {
    let mut out: Option<Vec<u8>> = None;
    let mut copied = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }
        let (end, drop) = match bytes[i + 1] {
            b']' => {
                let mut j = i + 2;
                let mut end = None;
                while j < bytes.len() {
                    if bytes[j] == 0x07 {
                        end = Some((j, j + 1));
                        break;
                    }
                    if bytes[j] == 0x1b && bytes.get(j + 1) == Some(&b'\\') {
                        end = Some((j, j + 2));
                        break;
                    }
                    j += 1;
                }
                let Some((body_end, end)) = end else {
                    break;
                };
                let body = &bytes[i + 2..body_end];
                (end, !clipboard_read && body.starts_with(b"52;") && body.ends_with(b";?"))
            }
            b'[' => {
                let mut j = i + 2;
                while j < bytes.len() && (0x30..=0x3f).contains(&bytes[j]) {
                    j += 1;
                }
                if j >= bytes.len() {
                    break;
                }
                let params = &bytes[i + 2..j];
                let report = bytes[j] == b't' && (params == b"20" || params == b"21");
                (j + 1, report)
            }
            _ => (i + 2, false),
        };
        if drop {
            let buf = out.get_or_insert_with(Vec::new);
            buf.extend_from_slice(&bytes[copied..i]);
            copied = end;
        }
        i = end;
    }
    match out {
        None => std::borrow::Cow::Borrowed(bytes),
        Some(mut buf) => {
            buf.extend_from_slice(&bytes[copied..]);
            std::borrow::Cow::Owned(buf)
        }
    }
}

/// What rsh draws over the program's screen: the host badge at the top
/// right and the `~h`/`~l` window. Like the predictor's guesses it is taken
/// off before output is written and put back after, so the screen under it
/// stays the program's.
struct Decor {
    badge: Option<String>,
    /// The escape key is held: the badge turns orange until the next key.
    armed: bool,
    /// The highlighted line of a window one picks from (index into
    /// `window`, whose line 0 is the title).
    selected: Option<usize>,
    /// First line is the title.
    window: Option<Vec<String>>,
    /// The connection notice, drawn over the dimmed screen; first line is
    /// the title.
    alert: Option<Vec<String>>,
    /// Covered spans of the tty: (row, from, to).
    drawn: Vec<(usize, usize, usize)>,
}

impl Decor {
    fn undo(&mut self, mirror: &Terminal) -> Vec<u8> {
        if self.drawn.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (row, from, to) in self.drawn.drain(..) {
            out.extend(mirror.render_span(row, from, to));
        }
        overdraw(mirror, out)
    }

    /// Draws what is due over a screen that has nothing of it. The badge
    /// stays off while the cursor on the top row reaches it and while the
    /// screen is dimmed.
    fn draw(&mut self, mirror: &Terminal) -> Vec<u8> {
        use ratatui::layout::Rect;
        use ratatui::style::{Color, Style};
        use ratatui::widgets::Paragraph;
        let (cols, rows) = mirror.size();
        let mut body = Vec::new();
        if self.alert.is_some() {
            for row in 0..usize::from(rows) {
                body.extend(mirror.render_span_dimmed(row, 0, usize::from(cols)));
                self.drawn.push((row, 0, usize::from(cols)));
            }
            body.extend(self.notice(mirror));
        }
        if let Some(badge) = &self.badge {
            // White on dark blue (256-color 18); dark orange (130) while the
            // escape key waits. Dark, so white reads well on any palette.
            let width = text_width(badge) as u16;
            let from = cols.saturating_sub(width);
            // Off only while the cursor on the top row reaches it, where it
            // would cover what is being typed.
            let (row, col) = mirror.cursor();
            let in_the_way = row == 0 && col + 2 >= usize::from(from);
            if self.alert.is_none() && !in_the_way && cols >= width + 20 {
                let mut from = from;
                let mut label = badge.clone();
                if mirror.cell(0, usize::from(from)).is_some_and(|cell| cell.is_spacer()) {
                    from -= 1;
                    label.insert(0, ' ');
                }
                let area = Rect::new(from, 0, cols - from, 1);
                let background = if self.armed { Color::Indexed(130) } else { Color::Indexed(18) };
                body.extend(widget_bytes(area, Paragraph::new(label).style(Style::new().fg(Color::White).bg(background).bold())));
                self.drawn.push((0, usize::from(from), usize::from(cols)));
            }
        }
        body.extend(self.window_bytes(mirror));
        if body.is_empty() {
            return Vec::new();
        }
        let mut out = b"\x1b(B\x0f".to_vec();
        out.extend(body);
        overdraw(mirror, out)
    }

    /// The `~h`/`~l` window, if open. Over the dimmed screen its rows are
    /// already among the covered ones.
    fn window_bytes(&mut self, mirror: &Terminal) -> Vec<u8> {
        use ratatui::layout::Rect;
        use ratatui::style::{Color, Style, Stylize};
        use ratatui::text::Line;
        use ratatui::widgets::{Block, BorderType, Padding, Paragraph};
        let (cols, rows) = mirror.size();
        let mut body = Vec::new();
        if let Some(lines) = self.window.as_ref().filter(|lines| !lines.is_empty()) {
            let (title, text) = (&lines[0], &lines[1..]);
            let widest = text.iter().chain([title]).map(|line| text_width(line)).max().unwrap_or(0) as u16;
            let width = (widest.clamp(24, 76) + 4).min(cols);
            let room = usize::from(rows.saturating_sub(4));
            // Scrolled so the highlighted line is shown.
            let selected = self.selected.map(|line| line.saturating_sub(1));
            let first = selected.map_or(0, |line| (line + 1).saturating_sub(room));
            let mut shown: Vec<Line> = text
                .iter()
                .enumerate()
                .skip(first)
                .take(room)
                .map(|(index, line)| {
                    let pad = usize::from(width.saturating_sub(4)).saturating_sub(text_width(line));
                    let line = Line::from(format!("{line}{}", " ".repeat(pad)));
                    if Some(index) == selected {
                        line.reversed()
                    } else {
                        line
                    }
                })
                .collect();
            if text.len() > first + room && room > 0 && selected.is_none() {
                shown.pop();
                shown.push(Line::from(format!("... {} more", text.len() - room + 1)));
            }
            let hint = if self.selected.is_some() { "  Up/Down move, Enter switch, q close" } else { "  any key closes" };
            let height = shown.len() as u16 + 2;
            if width >= 12 && height <= rows && !shown.is_empty() {
                let area = Rect::new((cols - width) / 2, (rows - height) / 2, width, height);
                // A colored outline on the default background.
                let block = Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(Color::Blue))
                    .padding(Padding::horizontal(1))
                    .title(Line::from(format!(" {title} ")).bold())
                    .title_bottom(Line::from(format!("{hint} ")).dim());
                body.extend(widget_bytes(area, Paragraph::new(shown).block(block)));
                if self.alert.is_none() {
                    for row in area.top()..area.bottom() {
                        self.drawn.push((usize::from(row), usize::from(area.left()), usize::from(area.right())));
                    }
                }
            } else {
                // Too small a terminal: drop the window, so the next key is
                // not swallowed by a window nobody sees.
                self.window = None;
                self.selected = None;
            }
        }
        body
    }

    /// The connection notice alone, at a fixed size in the middle, so a new
    /// text can replace the old without touching the dimmed screen.
    fn notice(&self, mirror: &Terminal) -> Vec<u8> {
        use ratatui::layout::Rect;
        use ratatui::style::{Color, Style, Stylize};
        use ratatui::text::Line;
        use ratatui::widgets::{Block, BorderType, Paragraph};
        let Some(lines) = &self.alert else {
            return Vec::new();
        };
        let (cols, rows) = mirror.size();
        let (width, height) = (cols.min(60), (lines.len() as u16 + 2).min(rows));
        let area = Rect::new((cols - width) / 2, (rows - height) / 2, width, height);
        let shown: Vec<Line> = lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let line = Line::from(line.as_str()).centered();
                if index == 0 {
                    line.bold()
                } else {
                    line
                }
            })
            .collect();
        let block = Block::bordered().border_type(BorderType::Rounded).border_style(Style::new().fg(Color::Red));
        widget_bytes(area, Paragraph::new(shown).block(block))
    }

    /// A new text for a notice already on screen.
    fn update_notice(&mut self, mirror: &Terminal, lines: Vec<String>) -> Vec<u8> {
        self.alert = Some(lines);
        let mut out = b"\x1b(B\x0f".to_vec();
        out.extend(self.notice(mirror));
        // A window open over the notice stays on top.
        out.extend(self.window_bytes(mirror));
        overdraw(mirror, out)
    }

}

/// The bytes that draw one widget in `area`, from ratatui's own crossterm
/// backend. Every cell is drawn, blank ones too, so nothing of the screen
/// shows through: the diff is against a buffer no cell can equal.
fn widget_bytes(area: ratatui::layout::Rect, widget: impl ratatui::widgets::Widget) -> Vec<u8> {
    use ratatui::backend::{Backend, CrosstermBackend};
    use ratatui::buffer::Buffer;
    let mut next = Buffer::empty(area);
    widget.render(area, &mut next);
    let mut unlike = Buffer::empty(area);
    for cell in &mut unlike.content {
        cell.set_symbol("\u{e000}");
    }
    let mut out = Vec::new();
    let _ = CrosstermBackend::new(&mut out).draw(unlike.diff(&next).into_iter());
    out
}

/// Bytes that draw over the program's screen without disturbing it:
/// absolute positions and overwriting whatever origin and insert modes the
/// program set, then its modes, cursor and pen back.
fn overdraw(mirror: &Terminal, body: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    if mirror.origin_mode() {
        out.extend_from_slice(b"\x1b[?6l");
    }
    if mirror.insert_mode() {
        out.extend_from_slice(b"\x1b[4l");
    }
    out.extend(body);
    if mirror.insert_mode() {
        out.extend_from_slice(b"\x1b[4h");
    }
    if mirror.origin_mode() {
        out.extend_from_slice(b"\x1b[?6h");
    }
    out.extend(mirror.cursor_sequence());
    out
}

fn text_width(text: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(text)
}

fn help_lines(key: u8, host: &str, label: &str, tmux: bool) -> Vec<String> {
    let k = escape_name(key);
    let at = if key < 0x20 { "" } else { " at the start of a line" };
    vec![
        format!("rsh {host}:{label}{}", if tmux { " (tmux)" } else { "" }),
        String::new(),
        if tmux { format!("{k}.  leave tmux; its session keeps running") } else { format!("{k}.  detach; the session keeps running") },
        format!("{k}l  sessions of every known host; Enter switches"),
        format!("{k}r  repaint the screen from the server"),
        format!("{k}h  this help"),
        format!("{k}{k}  send {k}"),
        String::new(),
        format!("Type these{at}."),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowKey {
    Up,
    Down,
    Enter,
    Close,
    Other,
}

/// What a key does in a window one picks from: arrows or k/j move, Enter
/// picks, q or Esc closes.
fn window_key(raw: &[u8], unit: &Unit) -> WindowKey {
    match unit {
        _ if raw.len() > 2 && raw[0] == 0x1b && raw.ends_with(b"A") => WindowKey::Up,
        _ if raw.len() > 2 && raw[0] == 0x1b && raw.ends_with(b"B") => WindowKey::Down,
        Unit::Key(Some('k')) => WindowKey::Up,
        Unit::Key(Some('j')) => WindowKey::Down,
        Unit::Key(Some('\r' | '\n')) => WindowKey::Enter,
        Unit::Key(Some('q' | '\x1b')) => WindowKey::Close,
        _ => WindowKey::Other,
    }
}

/// Input the terminal itself sends rather than a key: mouse and focus
/// reports, answers to queries, key releases.
pub fn human(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{secs} seconds")
    } else if secs < 3600 {
        format!("{}:{:02}", secs / 60, secs % 60)
    } else {
        format!("{}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_are_stripped() {
        let text = b"a\x1b]52;c;?\x07b\x1b]52;c;aGk=\x07c\x1b[21td\x1b[8;1;1te\x1b]52;p;?\x1b\\f";
        assert_eq!(&*strip_queries(text, false), b"ab\x1b]52;c;aGk=\x07cd\x1b[8;1;1tef");
        assert_eq!(&*strip_queries(text, true), b"a\x1b]52;c;?\x07b\x1b]52;c;aGk=\x07cd\x1b[8;1;1te\x1b]52;p;?\x1b\\f");
        assert!(matches!(strip_queries(b"plain \x1b[31mred", false), std::borrow::Cow::Borrowed(_)));
    }

    fn names(commands: Vec<Command>) -> String {
        commands
            .into_iter()
            .map(|c| match c {
                Command::Detach => '.',
                Command::Help => 'h',
                Command::Redraw => 'r',
                Command::List => 'l',
            })
            .collect()
    }

    #[test]
    fn escape_filter_commands_and_literals() {
        let mut esc = Escape::new(Some(DEFAULT_ESCAPE));
        let mut out = Vec::new();
        assert_eq!(names(esc.filter(b"~", &mut out)), "");
        assert!(esc.held.is_some() && out.is_empty(), "held until the next key");
        assert_eq!(names(esc.filter(b".", &mut out)), ".");
        assert_eq!(names(esc.filter(b"~rls", &mut out)), "r");
        assert_eq!(out, b"ls", "keys after a command go on");
        out.clear();
        assert_eq!(names(esc.filter(b"cd ~/x\r~~\r~/y a~b", &mut out)), "");
        assert_eq!(out, b"cd ~/x\r~\r~/y a~b", "~ is plain text inside a line, ~~ and ~/ send it");
        out.clear();
        assert_eq!(names(esc.filter(b"\r~h~?~l~.", &mut out)), "hhl.", "every command in one read");
        assert_eq!(out, b"\r");
        let mut ctrl = Escape::new(Some(0x1e));
        ctrl.typed = 1;
        out.clear();
        assert_eq!(names(ctrl.filter(b"ab\x1e.", &mut out)), ".", "a control key works anywhere");
        assert_eq!(out, b"ab");
        let mut off = Escape::new(None);
        out.clear();
        assert_eq!(names(off.filter(b"~.", &mut out)), "");
        assert_eq!(out, b"~.");
        assert_eq!(escape_name(0x1e), "Ctrl-^");
        assert_eq!(escape_name(b'~'), "~");
        assert_eq!(human(Duration::from_secs(75)), "1:15");
        for report in [&b"\x1b[<35;10;5M"[..], b"\x1b[I", b"\x1b[O", b"\x1b[12;40R", b"\x1b[?62c", b"\x1b]11;rgb:0/0/0\x07", b"\x1b[97;1:3u"] {
            let (keys, reports) = Escape::new(None).window_keys(report);
            assert!(keys.is_empty() && reports == report, "{report:?}");
        }
        for key in [&b"q"[..], b"\x1b[A", b"\x1b[15~", b"\x1bOP", b"\x1b[126u"] {
            let (keys, reports) = Escape::new(None).window_keys(key);
            assert!(keys.len() == 1 && reports.is_empty(), "{key:?}");
        }
    }

    #[test]
    fn escape_survives_reports_and_encoded_keys() {
        let mut out = Vec::new();
        // Focus and mouse reports between Enter and ~. do not end the line start.
        let mut esc = Escape::new(Some(DEFAULT_ESCAPE));
        esc.filter(b"ls\r", &mut out);
        esc.filter(b"\x1b[O\x1b[I", &mut out);
        esc.filter(b"\x1b[<35;10;5M", &mut out);
        assert_eq!(names(esc.filter(b"~.", &mut out)), ".");
        assert_eq!(out, b"ls\r\x1b[O\x1b[I\x1b[<35;10;5M");
        // kitty keyboard protocol: Enter, ~ (shifted) and . as CSI u, with
        // key releases in between.
        let mut kitty = Escape::new(Some(DEFAULT_ESCAPE));
        kitty.typed = 1;
        out.clear();
        kitty.filter(b"\x1b[13u\x1b[13;1:3u", &mut out);
        assert_eq!(names(kitty.filter(b"\x1b[126;2u\x1b[126;2:3u\x1b[46u", &mut out)), ".");
        assert_eq!(out, b"\x1b[13u\x1b[13;1:3u\x1b[126;2:3u", "the held ~ and the . are not sent");
        // modifyOtherKeys: ~ then x is sent as it came.
        let mut xterm = Escape::new(Some(DEFAULT_ESCAPE));
        out.clear();
        assert_eq!(names(xterm.filter(b"\x1b[27;2;126~x", &mut out)), "");
        assert_eq!(out, b"\x1b[27;2;126~x");
        // A pasted line starting with ~. is text.
        let mut paste = Escape::new(Some(DEFAULT_ESCAPE));
        out.clear();
        assert_eq!(names(paste.filter(b"\x1b[200~a\r~.\x1b[201~", &mut out)), "");
        assert_eq!(out, b"\x1b[200~a\r~.\x1b[201~");
    }

    #[test]
    fn escape_review_cases() {
        let mut out = Vec::new();
        // `~~` types a ~; once it is erased the line is empty again.
        let mut erased = Escape::new(Some(DEFAULT_ESCAPE));
        erased.filter(b"~~", &mut out);
        assert_eq!(out, b"~");
        assert_eq!(names(erased.filter(b"~h", &mut out)), "", "mid-line");
        out.clear();
        erased.filter(b"\x7f\x7f\x7f", &mut out);
        assert_eq!(names(erased.filter(b"~h", &mut out)), "h");
        erased.filter(b"abc\x15", &mut out);
        assert_eq!(names(erased.filter(b"~.", &mut out)), ".", "Ctrl-U empties the line");
        out.clear();
        // Ctrl-^ as the escape key, sent encoded (kitty, modifyOtherKeys),
        // with Num Lock on.
        for encoded in [&b"\x1b[54;5u"[..], b"\x1b[54;6u", b"\x1b[54:94;6u", b"\x1b[27;6;94~", b"\x1b[54;133u"] {
            let mut ctrl = Escape::new(Some(0x1e));
            let mut keys = encoded.to_vec();
            keys.push(b'.');
            assert_eq!(names(ctrl.filter(&keys, &mut out)), ".", "{encoded:?}");
        }
        // Enter with Caps Lock, a lone Shift press, Shift+` with the shifted
        // key reported (kitty flag 8 with alternates), and releases of
        // non-text keys do not get in the way.
        let mut esc = Escape::new(Some(DEFAULT_ESCAPE));
        esc.typed = 1;
        out.clear();
        esc.filter(b"\x1b[13;65u\x1b[13;65:3u\x1b[1;1:3A\x1b[57441;2u\x1b[96:126;2u", &mut out);
        assert_eq!(names(esc.filter(b"\x1b[46u", &mut out)), ".");
        // A ~ held before a paste goes out before it; a paste ending in
        // Enter leaves the line start.
        let mut paste = Escape::new(Some(DEFAULT_ESCAPE));
        out.clear();
        paste.filter(b"~\x1b[200~abc\r\x1b[201~", &mut out);
        assert_eq!(out, b"~\x1b[200~abc\r\x1b[201~");
        assert_eq!(names(paste.filter(b"~.", &mut out)), ".");
        // Sequences cut between reads are joined; a lone Alt+] that nothing
        // completes is sent as it is.
        let mut split = Escape::new(Some(DEFAULT_ESCAPE));
        out.clear();
        split.filter(b"ls\r\x1b[", &mut out);
        split.filter(b"I~", &mut out);
        assert_eq!(names(split.filter(b".", &mut out)), ".");
        assert_eq!(out, b"ls\r\x1b[I");
        out.clear();
        split.filter(b"\x1b[200", &mut out);
        split.filter(b"~\r~.\x1b[2", &mut out);
        assert_eq!(names(split.filter(b"01~", &mut out)), "", "a split paste start still marks the paste");
        out.clear();
        split.filter(b"\x1b]", &mut out);
        assert!(out.is_empty());
        split.expire(Instant::now() + PARTIAL_WAIT, &mut out);
        assert_eq!(out, b"\x1b]");
        // A held ~ goes out before a cut-off sequence that timed out.
        let mut order = Escape::new(Some(DEFAULT_ESCAPE));
        out.clear();
        order.filter(b"~\x1b[", &mut out);
        order.expire(Instant::now() + PARTIAL_WAIT, &mut out);
        assert_eq!(out, b"~\x1b[");
        // Shift+` with no shifted key reported is ~; a lone AltGr changes nothing.
        let mut shift = Escape::new(Some(DEFAULT_ESCAPE));
        shift.typed = 0;
        out.clear();
        assert_eq!(names(shift.filter(b"\x1b[57453;1u\x1b[96;2u\x1b[46u", &mut out)), ".");
        // Window keys: several in one read, and one cut between reads.
        let mut keys = Escape::new(Some(DEFAULT_ESCAPE));
        assert_eq!(keys.window_keys(b"\x1b[B\x1b[Bjk").0, [WindowKey::Down, WindowKey::Down, WindowKey::Down, WindowKey::Up]);
        assert!(keys.window_keys(b"\x1b").0.is_empty() || keys.window_keys(b"").0.is_empty());
        let mut cut = Escape::new(Some(DEFAULT_ESCAPE));
        assert!(cut.window_keys(b"\x1b[").0.is_empty());
        assert_eq!(cut.window_keys(b"B").0, [WindowKey::Down]);
        assert_eq!(cut.window_keys(b"q\x1b[113;1:3u"), (vec![WindowKey::Close], b"\x1b[113;1:3u".to_vec()), "the release is a report");
        // A clipboard answer cut between reads goes on whole, keys or not.
        let mut answer = Escape::new(Some(DEFAULT_ESCAPE));
        assert_eq!(answer.window_keys(b"\x1b]52;c;aGVs"), (vec![], vec![]));
        assert_eq!(answer.window_keys(b"bG8=\x1b\\j"), (vec![WindowKey::Down], b"\x1b]52;c;aGVsbG8=\x1b\\".to_vec()));
    }

    #[test]
    fn writes_follow_the_frames_the_program_draws() {
        let start = Instant::now();
        let mut screen = Vec::new();
        let mut out = Paced::new(&mut screen, true);
        // 60 frames a second: a write a little more often than that.
        for k in 0..30u32 {
            out.arrive(start + Duration::from_micros(16_667) * k);
        }
        assert!((Duration::from_millis(14)..=Duration::from_millis(16)).contains(&out.every), "{:?}", out.every);
        // Reads closer than a step (a flood, a frame in pieces) change nothing.
        let every = out.every;
        let at = start + Duration::from_micros(16_667) * 30;
        for k in 0..30u32 {
            out.arrive(at + Duration::from_millis(1) * k);
        }
        assert!(out.every.abs_diff(every) < Duration::from_millis(1), "{:?}", out.every);
        // A pause starts over; slower output never goes past `REFILL`.
        out.arrive(at + Duration::from_secs(3));
        assert_eq!(out.every, REFILL);
        for k in 1..20u32 {
            out.arrive(at + Duration::from_secs(3) + Duration::from_millis(100) * k);
        }
        assert_eq!(out.every, REFILL);
        // Faster than `MIN_REFILL` only goes that far.
        for k in 1..40u32 {
            out.arrive(at + Duration::from_secs(5) + Duration::from_millis(5) * k);
        }
        assert_eq!(out.every, MIN_REFILL);
    }

    #[test]
    fn an_idle_session_keeps_alive_slower_and_wakes_on_a_key() {
        let start = Instant::now();
        let mut wake = Wakeful::new(start);
        assert_eq!(wake.heartbeat(start), HEARTBEAT);
        let idle = start + IDLE_AFTER;
        assert_eq!(wake.heartbeat(idle), IDLE_HEARTBEAT);
        assert_eq!(wake.scaled(SHOW_SILENCE, idle), SHOW_SILENCE * 5);
        // The last reply came 4 s before the key: no silence yet.
        let key = idle + Duration::from_secs(4);
        wake.active(key);
        assert_eq!(wake.heartbeat(key), HEARTBEAT);
        assert_eq!(wake.silence(idle, key), Duration::ZERO);
        assert_eq!(wake.silence(idle, key + SHOW_SILENCE), SHOW_SILENCE);
    }

    #[test]
    fn writes_are_synchronized_and_paced() {
        let mut screen = Vec::new();
        let start = Instant::now();
        {
            let mut out = Paced::new(&mut screen, true);
            out.filled = start;
            // The program's own frame marks fold into one update per write.
            out.write_all(b"\x1b[?2026hframe\x1b[?2026l").unwrap();
            out.flush_paced(start).unwrap();
            // A frame left open stays open past the write.
            out.write_all(b"\x1b[?2026hhalf").unwrap();
            out.hold = true;
            out.flush_paced(start + Duration::from_millis(30)).unwrap();
        }
        assert_eq!(screen, b"\x1b[?2026hframe\x1b[?2026l\x1b[?2026hhalf", "the open frame is not ended for it");
        // Frames at about 30 a second with jitter go out as they come; a
        // flood is held to about 33 writes a second.
        let mut screen = Vec::new();
        let mut out = Paced::new(&mut screen, true);
        out.filled = start;
        let mut writes = 0;
        for at in [0, 28, 70, 95, 140, 160, 205] {
            out.write_all(b"f").unwrap();
            let before = out.pending.len();
            out.flush_paced(start + Duration::from_millis(at)).unwrap();
            writes += usize::from(out.pending.len() < before);
        }
        assert_eq!(writes, 7, "jittered 30 fps frames are not held");
        let mut flood = 0;
        for at in 0..100 {
            out.write_all(b"x").unwrap();
            let before = out.pending.len();
            out.flush_paced(start + Duration::from_millis(300 + at)).unwrap();
            flood += usize::from(out.pending.len() < before);
        }
        assert!((3..=6).contains(&flood), "{flood} writes in 100 ms of flood");
        drop(out);
        // A frame split over datagrams goes out whole: while the server has
        // sent more, the first part waits (at most CATCH_UP) for the rest.
        let mut screen = Vec::new();
        let mut out = Paced::new(&mut screen, false);
        out.filled = start;
        out.write_all(b"first half ").unwrap();
        out.since = Some(start);
        out.behind = true;
        out.flush_paced(start + Duration::from_millis(1)).unwrap();
        assert_eq!(out.due(), Some(start + CATCH_UP));
        out.write_all(b"second half").unwrap();
        out.behind = false;
        out.flush_paced(start + Duration::from_millis(2)).unwrap();
        out.write_all(b"; late").unwrap();
        out.since = Some(start + Duration::from_millis(3));
        out.behind = true;
        out.flush_paced(start + Duration::from_millis(3) + CATCH_UP).unwrap();
        drop(out);
        assert_eq!(screen, b"first half second half; late", "one write for the frame, and the wait is bounded");
        // A pipe gets the bytes as they came.
        let mut piped = Vec::new();
        Paced::new(&mut piped, false).write_all(b"\x1b[?2026hx").unwrap();
        assert_eq!(piped, b"\x1b[?2026hx");
    }

    #[test]
    fn badge_and_window_come_off_cleanly() {
        let start = "한글 top line that is long enough to reach the right edge\r\n$ ".as_bytes();
        // The user's terminal: the same output as the mirror got.
        let copy = |fed: &[&[u8]]| {
            let mut tty = Terminal::with_size(40, 10);
            for bytes in fed {
                tty.feed(bytes);
            }
            tty
        };
        let mut screen = copy(&[start]);
        let mut tty = copy(&[start]);
        let mut decor = Decor {
            badge: Some(" box ".into()),
            armed: false,
            selected: None,
            window: Some(vec!["title".into(), "a line".into()]),
            alert: None,
            drawn: Vec::new(),
        };
        tty.feed(&decor.draw(&screen));
        assert!(tty.screen_line(0).ends_with(" box"), "{}", tty.screen_line(0));
        assert_ne!(tty.cell(0, 37).unwrap().attr.bg, crate::terminal::Color::Default, "the badge has a background");
        assert!((0..10).any(|row| tty.screen_line(row).contains("a line")));
        assert_eq!(tty.cursor(), screen.cursor(), "the cursor is put back");
        tty.feed(&decor.undo(&screen));
        for row in 0..10 {
            assert_eq!(tty.screen_line(row), screen.screen_line(row));
        }
        assert!(decor.undo(&screen).is_empty());
        screen.feed(b"\x1b[H");
        decor.window = None;
        assert!(!decor.draw(&screen).is_empty(), "a prompt on the top row leaves room for the badge");
        decor.undo(&screen);
        screen.feed(b"\x1b[1;34H");
        assert!(decor.draw(&screen).is_empty(), "no badge where the cursor is about to type");
        screen.feed(b"\x1b[H");

        // Insert mode on: drawing and undoing must overwrite, not insert.
        screen.feed(b"\x1b[2;3H\x1b[4h");
        let mut tty = copy(&[start, b"\x1b[H", b"\x1b[2;3H\x1b[4h"]);
        tty.feed(&decor.draw(&screen));
        tty.feed(&decor.undo(&screen));
        assert_eq!(tty.screen_line(0), screen.screen_line(0));
        assert!(tty.insert_mode(), "insert mode is put back");

        // Disconnected: every row dimmed, the notice in the middle, no badge.
        decor.alert = Some(vec!["[ Session disconnected ]".into(), "Last contact 5 seconds ago".into()]);
        let mut tty = copy(&[start, b"\x1b[H", b"\x1b[2;3H\x1b[4h"]);
        tty.feed(&decor.draw(&screen));
        assert!((0..10).any(|row| tty.screen_line(row).contains("[ Session disconnected ]")));
        assert_ne!(tty.cell(1, 0).unwrap().attr.flags & crate::terminal::DIM, 0, "the screen is dimmed");
        assert!(!tty.screen_line(0).contains(" box"));
        tty.feed(&decor.undo(&screen));
        for row in 0..10 {
            assert_eq!(tty.screen_line(row), screen.screen_line(row));
            assert_eq!(tty.cell(row, 0), screen.cell(row, 0));
        }
        decor.alert = None;

        let tiny = Terminal::with_size(10, 3);
        decor.window = Some(vec!["t".into(), "x".into()]);
        assert!(decor.draw(&tiny).is_empty());
        assert!(decor.window.is_none(), "a window too big to show is dropped");
    }
}
