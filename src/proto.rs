pub use crate::crypto::Dir;
use crate::crypto::{next_nonce, Key, NONCE_LEN, TAG_LEN};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub const MAGIC: &[u8; 4] = b"RSH3";
pub const HEADER_LEN: usize = 4 + 1 + 8 + NONCE_LEN;
/// What sealing adds to a payload.
pub const SEAL_OVERHEAD: usize = HEADER_LEN + TAG_LEN;
/// Largest stream payload per datagram. Header, tag and stream fields keep
/// the datagram under the 1280-byte IPv6 minimum MTU.
pub const CHUNK: usize = 1100;
/// Bytes a sender keeps in flight before it waits for an ack, to start
/// with; the window grows to `MAX_WINDOW` while round trips show no
/// queueing, and a queue built by a full window takes it to `MIN_WINDOW`.
pub const WINDOW: u64 = 64 * CHUNK as u64;
/// Below the server's 1 MiB `OUTPUT_BACKLOG`, which counts bytes in flight.
pub const MAX_WINDOW: u64 = 8 * WINDOW;
/// The least window a queue on the path shrinks it to: a slow link (1 Mbit/s
/// takes 0.56 s for `WINDOW`) needs less in flight to stay responsive. Only
/// a queue the window itself built goes below `WINDOW`: not loss (on radio
/// links it is no queue) and not a round-trip spike while little is sent.
pub const MIN_WINDOW: u64 = 8 * CHUNK as u64;
/// Queueing delay (round trip above the least seen) the window aims under:
/// this or a quarter of the least round trip, whichever is more.
const QUEUE_TARGET_MS: f64 = 5.0;
/// Below this (bytes a second, 2 Mbit/s) a link is slow: what is in flight
/// is held to about two round trips' delivery, screen updates wait for
/// room, and output that would take long to drain is skipped.
pub const SLOW_RATE: f64 = 250_000.0;
/// What a datagram carries besides its stream bytes: header, nonce, tag,
/// the stream fields, and IPv4 and UDP.
pub const WIRE_OVERHEAD: u64 = 86 + 28;
/// The longest a slow link's timeout grows for what it has in flight.
const SLOW_MAX_RTO: Duration = Duration::from_secs(3);
/// The least round trip is the least of about this long: a path that got
/// slower for good (the client moved to another network) is learned again
/// instead of reading as a queue forever.
const MIN_RTT_SPAN: Duration = Duration::from_secs(10);
/// Datagrams in a row whose own time on the path (see `on_ack`) is over
/// twice the least round trip, or, from a near one (under 20 ms), timed
/// round trips in a row of 20 ms and more, or, climbing (see `CLIMB_SPAN`),
/// timed round trips in a row late, from which the path's delay rose for
/// good (the same address, a longer route): the least round trip, and what
/// was measured against it, are learned again.
const ABOVE_FLOOR: u32 = 3;
/// After the least round trip was raised, not again for this long (or ten
/// of the new one, but for a climb's step): what went before the rate was
/// known may still queue, and taken for path it would raise it again and
/// again.
const REFLOOR_REST: Duration = Duration::from_secs(1);
/// How long what a capped raise of the least round trip held back (at most
/// three times the floor before it) may still be climbed to, while timed
/// round trips stay late.
const CLIMB_SPAN: Duration = Duration::from_secs(10);
/// The capacity is the median of the path's delivery samples (see
/// `on_ack`): the last `RATE_SAMPLES` of the last `RATE_SPAN`, once there
/// are `MIN_RATE_SAMPLES`; with fewer it stays as it was.
const RATE_SAMPLES: usize = 15;
const MIN_RATE_SAMPLES: usize = 3;
const RATE_SPAN: Duration = Duration::from_secs(2);
/// The least time a delivery sample spans: acks read together (the server
/// reads all that came while it was busy) share one time.
const RATE_SAMPLE_SPAN: Duration = Duration::from_millis(30);
/// A slow link that took what was sent near its capacity for a round trip
/// with no queue delivers more: the capacity goes up this much, and a
/// queue, if one forms, measures it again. (Too high costs little: what
/// is in flight stays within about two round trips' delivery.)
const PROBE_GAIN: f64 = 1.25;
/// The least capacity a link is taken for, wire bytes a second (16 kbit/s):
/// a lower estimate (samples of a stall) would hold it to a crawl.
const MIN_CAPACITY: f64 = 2_000.0;
pub const MIN_RTO: Duration = Duration::from_millis(50);
pub const MAX_RTO: Duration = Duration::from_millis(1000);
const INITIAL_RTO: Duration = Duration::from_millis(250);

pub const KIND_HELLO: u8 = 1;
pub const KIND_READY: u8 = 2;
pub const KIND_DENIED: u8 = 3;
pub const KIND_LIST: u8 = 4;
pub const KIND_LIST_REPLY: u8 = 5;
pub const KIND_STREAM: u8 = 6;
pub const KIND_KICKED: u8 = 7;
pub const KIND_CHANNEL: u8 = 10;
/// Second half of the handshake, sealed with the new attachment key.
pub const KIND_CONFIRM: u8 = 11;
pub const KIND_ATTACHED: u8 = 12;
/// Is my attachment still there? Sealed with the token key; asked when
/// the server has been silent, since a restarted server no longer knows
/// the attachment's key to say so.
pub const KIND_PING: u8 = 13;
pub const KIND_PONG: u8 = 14;
/// Largest datagram a server looks at; anything bigger is dropped before
/// any cryptography is spent on it.
pub const MAX_REQUEST: usize = 4096;

pub const FLAG_REDRAW: u8 = 1;
/// Client stream flag: this packet acks nothing the server lacks and is no
/// duplicate ack either (a heartbeat, a resize): not a loss signal. A
/// server that predates it ignores the bit.
pub const FLAG_REPEAT: u8 = 4;
pub const FLAG_DETACH: u8 = 2;
pub const FLAG_EXITED: u8 = 1;
/// Server stream flag: `data` is raw-deflated stream bytes (see `deflate`).
pub const FLAG_DEFLATE: u8 = 4;
/// HELLO features: the client inflates `FLAG_DEFLATE` datagrams.
pub const FEATURE_DEFLATE: u8 = 1;
/// HELLO features: the client writes to a terminal, so screen updates may
/// stand in for output bytes when the link falls behind.
pub const FEATURE_SCREENS: u8 = 2;
/// HELLO features: the client inflates a generation sent as one deflate
/// stream (`FLAG_ZSTREAM`).
pub const FEATURE_ZSTREAM: u8 = 4;
/// HELLO features: the user asked for compression on any link
/// (`--compress`, `compress: always`).
pub const FEATURE_COMPRESS: u8 = 8;
/// Server stream flag: this generation's stream is one raw deflate stream,
/// flushed where the server's output was; the client inflates it in order.
pub const FLAG_ZSTREAM: u8 = 2;
/// Server stream flag: the stream carries screen updates (the
/// screens-only mode): the client shows each one whole, waiting for the
/// rest of it (`end_offset`) rather than showing part of a frame. Older
/// clients ignore it.
pub const FLAG_UPDATES: u8 = 8;
/// HELLO features: the client reports when each numbered datagram arrived
/// (`FLAG_NUMBERED`, `FLAG_FEEDBACK`), for the server's path model.
pub const FEATURE_PATH: u8 = 16;
/// Server stream flag: the datagram carries its number (`pn`).
pub const FLAG_NUMBERED: u8 = 16;
/// Client stream flag: the packet carries arrival times of numbered
/// datagrams (`Feedback`).
pub const FLAG_FEEDBACK: u8 = 8;
/// Stream bytes one compressed datagram may stand for.
pub const SQUEEZED: usize = 16 * CHUNK;

/// Raw deflate, fast level: a datagram is compressed on its own, so a lost
/// one costs nothing else.
pub fn deflate(bytes: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec(bytes, 1)
}

pub fn inflate(bytes: &[u8]) -> Option<Vec<u8>> {
    miniz_oxide::inflate::decompress_to_vec_with_limit(bytes, SQUEEZED).ok()
}

/// One generation's output as a raw deflate stream (level 9: a tenth
/// smaller than 6 on a full screen of colours, for little more CPU at
/// terminal rates): its history
/// spans everything sent before, so a small screen update costs a few
/// bytes. Each `push` ends on a sync flush, so the client can show all of
/// it once it has those bytes.
pub struct Deflater(Box<miniz_oxide::deflate::core::CompressorOxide>);

impl Default for Deflater {
    fn default() -> Self {
        let flags = miniz_oxide::deflate::core::create_comp_flags_from_zip_params(1, -15, 0);
        Deflater(Box::new(miniz_oxide::deflate::core::CompressorOxide::new(flags)))
    }
}

impl Deflater {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<u8> {
        use miniz_oxide::deflate::core::{compress, TDEFLFlush, TDEFLStatus};
        let mut out = vec![0u8; bytes.len() + bytes.len() / 8 + 64];
        let (mut read, mut written) = (0, 0);
        loop {
            let (status, r, w) = compress(&mut self.0, &bytes[read..], &mut out[written..], TDEFLFlush::Sync);
            read += r;
            written += w;
            if status != TDEFLStatus::Okay || (read == bytes.len() && written < out.len()) {
                break;
            }
            out.resize(out.len() * 2, 0);
        }
        out.truncate(written);
        out
    }
}

/// The client's end of a `Deflater`.
pub struct Inflater(Box<miniz_oxide::inflate::stream::InflateState>);

impl Default for Inflater {
    fn default() -> Self {
        Inflater(miniz_oxide::inflate::stream::InflateState::new_boxed(miniz_oxide::DataFormat::Raw))
    }
}

impl Inflater {
    /// The output for the next bytes of the stream; None if it is broken.
    pub fn push(&mut self, mut bytes: &[u8]) -> Option<Vec<u8>> {
        use miniz_oxide::inflate::stream::inflate;
        use miniz_oxide::{MZError, MZFlush, MZStatus};
        let mut out = Vec::new();
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let result = inflate(&mut self.0, bytes, &mut buf, MZFlush::None);
            out.extend_from_slice(&buf[..result.bytes_written]);
            bytes = &bytes[result.bytes_consumed..];
            match result.status {
                Ok(MZStatus::Ok) if bytes.is_empty() && result.bytes_written < buf.len() => return Some(out),
                Ok(MZStatus::Ok) => {}
                Err(MZError::Buf) => return Some(out),
                _ => return None,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub kind: u8,
    pub session: u64,
    pub counter: u64,
    pub nonce: [u8; NONCE_LEN],
    pub payload: Vec<u8>,
}

pub fn seal(key: &Key, dir: Dir, kind: u8, session: u64, payload: &[u8]) -> Vec<u8> {
    let (nonce, _) = next_nonce();
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len() + TAG_LEN);
    out.extend_from_slice(MAGIC);
    out.push(kind);
    out.extend_from_slice(&session.to_be_bytes());
    out.extend_from_slice(&nonce);
    let sealed = key.seal(dir, &nonce, &out, payload);
    out.extend_from_slice(&sealed);
    out
}

/// Kind and session of a datagram before it is authenticated, so the
/// receiver can pick the key. Nothing here may be trusted yet.
pub fn peek(buf: &[u8]) -> Option<(u8, u64)> {
    if buf.len() > 65_000 {
        return None;
    }
    if buf.len() < HEADER_LEN + TAG_LEN || &buf[..4] != MAGIC {
        return None;
    }
    Some((buf[4], u64::from_be_bytes(buf[5..13].try_into().ok()?)))
}

pub fn open(key: &Key, dir: Dir, buf: &[u8]) -> Option<Packet> {
    let (kind, session) = peek(buf)?;
    let nonce: [u8; NONCE_LEN] = buf[13..HEADER_LEN].try_into().ok()?;
    let payload = key.open(dir, &nonce, &buf[..HEADER_LEN], &buf[HEADER_LEN..])?;
    let counter = u64::from_be_bytes(nonce[16..].try_into().ok()?);
    Some(Packet {
        kind,
        session,
        counter,
        nonce,
        payload,
    })
}

/// Remembers which nonces of one sender were seen, so a replayed or
/// duplicated packet is dropped while reordered ones still count. The
/// sender is pinned by the random nonce prefix of its first packet.
#[derive(Clone, Debug)]
pub struct ReplayWindow {
    prefix: Option<[u8; 16]>,
    top: u64,
    seen: u128,
}

impl Default for ReplayWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayWindow {
    pub fn new() -> Self {
        Self {
            prefix: None,
            top: 0,
            seen: 0,
        }
    }

    pub fn pinned(nonce: &[u8; NONCE_LEN]) -> Self {
        let mut window = Self::new();
        window.check(nonce);
        window
    }

    /// True the first time a nonce is seen; also true for a newer counter.
    pub fn check(&mut self, nonce: &[u8; NONCE_LEN]) -> bool {
        let prefix: [u8; 16] = nonce[..16].try_into().expect("prefix");
        let counter = u64::from_be_bytes(nonce[16..].try_into().expect("counter"));
        match self.prefix {
            None => self.prefix = Some(prefix),
            Some(known) if known != prefix => return false,
            Some(_) => {}
        }
        if counter > self.top {
            let shift = counter - self.top;
            self.seen = if shift >= 128 { 0 } else { self.seen << shift };
            self.seen |= 1;
            self.top = counter;
            return true;
        }
        let age = self.top - counter;
        if age >= 128 || self.seen & (1u128 << age) != 0 {
            return false;
        }
        self.seen |= 1u128 << age;
        true
    }

    /// Writes the window so another process can continue it.
    pub fn save(&self, w: &mut Writer) {
        let prefix = self.prefix.unwrap_or([0; 16]);
        w.u8(u8::from(self.prefix.is_some())).rest(&prefix).u64(self.top).rest(&self.seen.to_be_bytes());
    }

    pub fn load(r: &mut Reader) -> Option<Self> {
        let pinned = r.u8()? != 0;
        let prefix: [u8; 16] = r.take(16)?.try_into().ok()?;
        let top = r.u64()?;
        let seen = u128::from_be_bytes(r.take(16)?.try_into().ok()?);
        Some(Self {
            prefix: pinned.then_some(prefix),
            top,
            seen,
        })
    }

    /// The highest counter accepted so far.
    pub fn top(&self) -> u64 {
        self.top
    }
}

#[derive(Default)]
pub struct Writer {
    pub buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn u8(&mut self, value: u8) -> &mut Self {
        self.buf.push(value);
        self
    }

    pub fn u16(&mut self, value: u16) -> &mut Self {
        self.buf.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.buf.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u64(&mut self, value: u64) -> &mut Self {
        self.buf.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn i32(&mut self, value: i32) -> &mut Self {
        self.buf.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn str(&mut self, value: &str) -> &mut Self {
        let bytes = &value.as_bytes()[..value.len().min(u16::MAX as usize)];
        self.u16(bytes.len() as u16);
        self.buf.extend_from_slice(bytes);
        self
    }

    pub fn strs(&mut self, values: &[String]) -> &mut Self {
        self.u16(values.len().min(255) as u16);
        for value in values.iter().take(255) {
            self.str(value);
        }
        self
    }

    /// Bytes with a 32-bit length in front.
    pub fn bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.u32(bytes.len() as u32);
        self.buf.extend_from_slice(bytes);
        self
    }

    pub fn rest(&mut self, bytes: &[u8]) -> &mut Self {
        self.buf.extend_from_slice(bytes);
        self
    }

    pub fn finish(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.buf)
    }
}

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(len)?;
        let slice = self.buf.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    pub fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }

    pub fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|b| u16::from_be_bytes([b[0], b[1]]))
    }

    pub fn u32(&mut self) -> Option<u32> {
        self.take(4).map(|b| u32::from_be_bytes(b.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> Option<u64> {
        self.take(8).map(|b| u64::from_be_bytes(b.try_into().unwrap()))
    }

    pub fn i32(&mut self) -> Option<i32> {
        self.take(4).map(|b| i32::from_be_bytes(b.try_into().unwrap()))
    }

    pub fn str(&mut self) -> Option<String> {
        let len = usize::from(self.u16()?);
        self.take(len).map(|b| String::from_utf8_lossy(b).into_owned())
    }

    pub fn strs(&mut self) -> Option<Vec<String>> {
        let count = self.u16()?;
        (0..count).map(|_| self.str()).collect()
    }

    pub fn bytes_fixed(&mut self, len: usize) -> Option<&'a [u8]> {
        self.take(len)
    }

    pub fn bytes(&mut self) -> Option<&'a [u8]> {
        let len = self.u32()? as usize;
        self.take(len)
    }

    pub fn rest(&mut self) -> &'a [u8] {
        let slice = &self.buf[self.pos.min(self.buf.len())..];
        self.pos = self.buf.len();
        slice
    }
}

/// The first `remote-<n>` name not taken, from 1: the name a session gets
/// when none is asked for.
pub fn free_name(taken: impl Fn(&str) -> bool) -> String {
    (1..).map(|n| format!("remote-{n}")).find(|name| !taken(name)).expect("a free name")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Attach the oldest live session of the host, or start one with the
    /// first free `remote-<n>` name.
    Default,
    /// Attach the live session with this name, or start it.
    Named(String),
    /// Attach this session id.
    Attach(u64),
    /// End the sessions whose id or name is this; no attachment follows.
    Kill(String),
}

/// First packet of a handshake, sealed with the token key. Nothing happens
/// on the server until the client proves, with CONFIRM, that it holds the
/// secret behind `client_pub`; a replayed HELLO therefore does nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hello {
    pub time_ms: u64,
    pub host: String,
    pub action: Action,
    pub cols: u16,
    pub rows: u16,
    pub env: Vec<String>,
    pub command: Vec<String>,
    /// Remote forwards (`-R`) as `bind:port:host:port`.
    pub forwards: Vec<String>,
    pub client_pub: [u8; 32],
    /// `FEATURE_*` bits; a server that predates them ignores the byte.
    pub features: u8,
}

impl Hello {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u64(self.time_ms).str(&self.host);
        match &self.action {
            Action::Default => w.u8(0).str(""),
            Action::Named(name) => w.u8(1).str(name),
            Action::Attach(id) => w.u8(2).str(&id.to_string()),
            Action::Kill(target) => w.u8(3).str(target),
        };
        w.u16(self.cols)
            .u16(self.rows)
            .strs(&self.env)
            .strs(&self.command)
            .strs(&self.forwards)
            .rest(&self.client_pub)
            .u8(self.features);
        w.finish()
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        let mut r = Reader::new(buf);
        let time_ms = r.u64()?;
        let host = r.str()?;
        let tag = r.u8()?;
        let arg = r.str()?;
        let action = match tag {
            0 => Action::Default,
            1 => Action::Named(arg),
            2 => Action::Attach(arg.parse().ok()?),
            3 => Action::Kill(arg),
            _ => return None,
        };
        Some(Self {
            time_ms,
            host,
            action,
            cols: r.u16()?,
            rows: r.u16()?,
            env: r.strs()?,
            command: r.strs()?,
            forwards: r.strs()?,
            client_pub: r.take(32)?.try_into().ok()?,
            // A client that predates features sends none.
            features: r.u8().unwrap_or(0),
        })
    }
}

/// Answer to HELLO, sealed with the token key: the server's half of the
/// exchange and the id the attachment's packets will carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ready {
    /// The HELLO's nonce, so an old READY cannot be passed off as this one.
    pub echo: [u8; 24],
    pub attach: u64,
    pub server_pub: [u8; 32],
    /// The server's `rsh --version` line.
    pub server: String,
}

impl Ready {
    pub fn encode(&self) -> Vec<u8> {
        Writer::new()
            .rest(&self.echo)
            .u64(self.attach)
            .rest(&self.server_pub)
            .str(&self.server)
            .finish()
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        let mut r = Reader::new(buf);
        Some(Self {
            echo: r.take(24)?.try_into().ok()?,
            attach: r.u64()?,
            server_pub: r.take(32)?.try_into().ok()?,
            server: r.str()?,
        })
    }
}

/// A refusal, sealed with the token key, echoing the request's nonce.
pub fn encode_denied(echo: &[u8; 24], reason: &str) -> Vec<u8> {
    Writer::new().rest(echo).str(reason).finish()
}

pub fn decode_denied(buf: &[u8]) -> Option<([u8; 24], String)> {
    let mut r = Reader::new(buf);
    Some((r.take(24)?.try_into().ok()?, r.str()?))
}

/// What CONFIRM did, sealed with the attachment's key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attached {
    pub session: u64,
    pub label: String,
    pub created: bool,
    /// Problems the client should show, such as a remote forward that
    /// could not listen.
    pub notes: Vec<String>,
    /// Sessions a Kill ended.
    pub killed: Vec<u64>,
    /// Set instead of the rest when the action could not be done.
    pub denied: String,
}

impl Attached {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u64(self.session)
            .str(&self.label)
            .u8(u8::from(self.created))
            .strs(&self.notes)
            .u16(self.killed.len() as u16);
        for id in &self.killed {
            w.u64(*id);
        }
        w.str(&self.denied);
        w.finish()
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        let mut r = Reader::new(buf);
        let session = r.u64()?;
        let label = r.str()?;
        let created = r.u8()? != 0;
        let notes = r.strs()?;
        let killed = (0..r.u16()?).map(|_| r.u64()).collect::<Option<Vec<_>>>()?;
        Some(Self {
            session,
            label,
            created,
            notes,
            killed,
            denied: r.str()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: u64,
    pub host: String,
    pub label: String,
    pub alive: bool,
    pub attached: bool,
    pub cols: u16,
    pub rows: u16,
    pub created_unix: u64,
    pub idle_secs: u64,
    pub command: String,
    pub title: String,
}

/// A tmux session on the server's host (`tmux::Found` as listed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TmuxInfo {
    pub name: String,
    pub clients: u32,
    pub cols: u16,
    pub rows: u16,
    pub idle_secs: u64,
    /// The live rsh session joined to it, 0 if none.
    pub session: u64,
}

/// What a LIST answers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    pub sessions: Vec<SessionInfo>,
    /// The server's `rsh --version` line.
    pub server: String,
    /// Empty unless asked for (`LIST_TMUX`) and the server knows them.
    pub tmux: Vec<TmuxInfo>,
}

/// LIST flag: add the host's tmux sessions. A server that predates it
/// ignores the byte and answers without them.
pub const LIST_TMUX: u8 = 1;

/// The LIST's nonce, the session list, the server's `rsh --version` line,
/// then the tmux sessions (a clients that predates them stops reading
/// before).
pub fn encode_list(echo: &[u8; 24], sessions: &[SessionInfo], server: &str, tmux: &[TmuxInfo]) -> Vec<u8> {
    let mut w = Writer::new();
    w.rest(echo).u16(sessions.len() as u16);
    for s in sessions {
        w.u64(s.id)
            .str(&s.host)
            .str(&s.label)
            .u8(u8::from(s.alive) | (u8::from(s.attached) << 1))
            .u16(s.cols)
            .u16(s.rows)
            .u64(s.created_unix)
            .u64(s.idle_secs)
            .str(&s.command)
            .str(&s.title);
    }
    w.str(server).u16(tmux.len() as u16);
    for t in tmux {
        w.str(&t.name).u32(t.clients).u16(t.cols).u16(t.rows).u64(t.idle_secs).u64(t.session);
    }
    w.finish()
}

pub fn decode_list(buf: &[u8]) -> Option<([u8; 24], Listing)> {
    let mut r = Reader::new(buf);
    let echo: [u8; 24] = r.take(24)?.try_into().ok()?;
    let count = r.u16()?;
    let sessions = (0..count)
        .map(|_| {
            let id = r.u64()?;
            let host = r.str()?;
            let label = r.str()?;
            let flags = r.u8()?;
            Some(SessionInfo {
                id,
                host,
                label,
                alive: flags & 1 != 0,
                attached: flags & 2 != 0,
                cols: r.u16()?,
                rows: r.u16()?,
                created_unix: r.u64()?,
                idle_secs: r.u64()?,
                command: r.str()?,
                title: r.str()?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let server = r.str()?;
    // A server that predates tmux listing ends here.
    let tmux = match r.u16() {
        None => Vec::new(),
        Some(count) => (0..count)
            .map(|_| {
                Some(TmuxInfo {
                    name: r.str()?,
                    clients: r.u32()?,
                    cols: r.u16()?,
                    rows: r.u16()?,
                    idle_secs: r.u64()?,
                    session: r.u64()?,
                })
            })
            .collect::<Option<Vec<_>>>()?,
    };
    Some((echo, Listing { sessions, server, tmux }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientStream<'a> {
    pub flags: u8,
    pub cols: u16,
    pub rows: u16,
    pub out_gen: u32,
    pub out_ack: u64,
    pub in_offset: u64,
    /// An encoded `Feedback` (`FLAG_FEEDBACK`), or empty.
    pub feedback: &'a [u8],
    pub data: &'a [u8],
}

impl<'a> ClientStream<'a> {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        let flags = if self.feedback.is_empty() { self.flags & !FLAG_FEEDBACK } else { self.flags | FLAG_FEEDBACK };
        w.u8(flags).u16(self.cols).u16(self.rows).u32(self.out_gen).u64(self.out_ack).u64(self.in_offset);
        if !self.feedback.is_empty() {
            w.bytes(self.feedback);
        }
        w.rest(self.data).finish()
    }

    pub fn decode(buf: &'a [u8]) -> Option<Self> {
        let mut r = Reader::new(buf);
        let flags = r.u8()?;
        let (cols, rows, out_gen, out_ack, in_offset) = (r.u16()?, r.u16()?, r.u32()?, r.u64()?, r.u64()?);
        let feedback = if flags & FLAG_FEEDBACK != 0 { r.bytes()? } else { &[] };
        Some(Self { flags, cols, rows, out_gen, out_ack, in_offset, feedback, data: r.rest() })
    }
}

/// When numbered datagrams arrived, by the client's clock (microseconds,
/// wrapping), and when the packet carrying this left it: the server learns
/// the path's delay changes and how fast datagrams came off its bottleneck
/// without the time either end spent on its work.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Feedback {
    pub sent_us: u32,
    /// (datagram number, arrival), in arrival order.
    pub arrivals: Vec<(u32, u32)>,
}

impl Feedback {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.sent_us);
        for &(pn, at) in &self.arrivals {
            w.u32(pn).u32(at);
        }
        w.finish()
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        let mut r = Reader::new(buf);
        let sent_us = r.u32()?;
        let mut arrivals = Vec::new();
        while let Some(pn) = r.u32() {
            arrivals.push((pn, r.u32()?));
        }
        Some(Self { sent_us, arrivals })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerStream<'a> {
    pub flags: u8,
    pub in_ack: u64,
    pub out_gen: u32,
    pub out_offset: u64,
    pub exit_code: i32,
    pub end_offset: u64,
    /// The datagram's number (`FLAG_NUMBERED`), for the client's
    /// `Feedback`.
    pub pn: Option<u32>,
    pub data: &'a [u8],
}

impl<'a> ServerStream<'a> {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        let flags = if self.pn.is_some() { self.flags | FLAG_NUMBERED } else { self.flags & !FLAG_NUMBERED };
        w.u8(flags).u64(self.in_ack).u32(self.out_gen).u64(self.out_offset).i32(self.exit_code).u64(self.end_offset);
        if let Some(pn) = self.pn {
            w.u32(pn);
        }
        w.rest(self.data).finish()
    }

    pub fn decode(buf: &'a [u8]) -> Option<Self> {
        let mut r = Reader::new(buf);
        let flags = r.u8()?;
        let (in_ack, out_gen, out_offset, exit_code, end_offset) = (r.u64()?, r.u32()?, r.u64()?, r.i32()?, r.u64()?);
        let pn = if flags & FLAG_NUMBERED != 0 { Some(r.u32()?) } else { None };
        Some(Self { flags, in_ack, out_gen, out_offset, exit_code, end_offset, pn, data: r.rest() })
    }
}

/// Shrinks `deque` to what it holds when it holds well under what it has
/// room for (and more than a little room).
pub fn shrink<T>(deque: &mut VecDeque<T>) {
    if deque.capacity() * std::mem::size_of::<T>() > 64 * 1024 && deque.len() < deque.capacity() / 4 {
        deque.shrink_to(deque.len() * 2);
    }
}

/// Reliable, ordered byte stream over datagrams: cumulative byte-offset acks
/// and a window that grows while the link shows no queueing and shrinks
/// when it does or a datagram is lost (below `WINDOW` only for a queue the
/// window built, see `MIN_WINDOW`). A hole is repaired by resending one chunk after three
/// duplicate acks or a partial ack. An RFC 6298 timeout resends the first
/// unacked chunk only (again, if the repair times out too with no ack in
/// between). When the ack for a resent chunk stops right at its end (the
/// next chunk is missing too: a run of losses, as a queue dropped when the
/// path changed), or the repair times out while acks come, everything
/// unacked goes out again. A new generation discards unacked
/// bytes and restarts at zero.
pub struct Sender {
    pub gen: u32,
    base: u64,
    buf: VecDeque<u8>,
    sent: u64,
    high: u64,
    /// The most sent when this generation last went back over sent bytes:
    /// acks up to it may answer resends, so they time nothing.
    rewound: u64,
    srtt: Option<f64>,
    rttvar: f64,
    rto: Duration,
    timing: Option<(u64, Instant)>,
    deadline: Option<Instant>,
    dup_acks: u32,
    resend_base: bool,
    recover: Option<u64>,
    /// The end of the chunk last resent, until an ack reaches it.
    probe: Option<u64>,
    /// An ack of this generation came since the last timeout.
    heard: bool,
    /// When the last timeout fired, and the timeout and window before it,
    /// so an ack too soon to answer the resend can undo it.
    timed_out: Option<(Instant, Duration, u64)>,
    /// Length of the stream this generation started with.
    first: u64,
    window: u64,
    /// The least round trip seen, in ms: the link without queueing.
    min_rtt: Option<f64>,
    /// The least round trip of the span under way, and when it began.
    span_min: Option<(f64, Instant)>,
    /// The last round trip sample, ms.
    last_rtt: Option<f64>,
    /// Whether the window held data back since the last round trip sample.
    limited: bool,
    /// The most in flight since the last round trip sample.
    peak: u64,
    /// The last chunk handed out if it was new data: its offset, and `high`
    /// and `peak` before it (what `take_back` restores).
    fresh: Option<(u64, u64, u64)>,
    /// Share of recent datagrams that were resends (moving average over
    /// about 500 sends).
    resent: f64,
    /// Where each datagram of new data in flight ends, oldest first, when
    /// it went, its wire bytes, and the wire bytes in flight up to and
    /// with it then.
    flight: VecDeque<(u64, Instant, u64, u64)>,
    /// The least time from send to ack, ms, seen for a datagram sent onto
    /// an empty path and acked alone,
    /// with its wire bytes, by size (128 bytes a step): the extra time a
    /// larger one takes over the smallest is the link's time for the extra
    /// bytes. The least, as a queue only adds time.
    by_size: [Option<(f64, u64)>; 10],
    resend_seen: bool,
    /// Bytes earlier generations had in flight at their reset, still taking
    /// room in the link until this generation's first ack.
    carried: u64,
    /// Bytes acked over the sender's life, and the datagrams that carried
    /// the new ones as they went on the wire (`WIRE_OVERHEAD` each).
    delivered: u64,
    delivered_wire: u64,
    /// When an ack last moved the stream.
    last_ack: Option<Instant>,
    /// As acks came (over the last second, or two least round trips if
    /// longer): when, the acked wire bytes (`delivered_wire`), and the wire
    /// bytes still in flight after it, for the ack clock (see
    /// `next_chunk_max`).
    acked_at: VecDeque<(Instant, u64, u64)>,
    /// The delivery sample under way: since which ack the path stayed
    /// busy, and the wire bytes acked since (see `on_ack`).
    rate_span: Option<(Instant, u64)>,
    /// The path's delivery rate, wire bytes a second, as acks of datagrams
    /// that waited on it showed it, latest last (see `on_ack`). (No round
    /// trip samples are needed, which a slow link's queue longer than
    /// `MAX_RTO` keeps from coming.)
    rates: VecDeque<(Instant, f64)>,
    /// The path's capacity: the median of `rates`, or raised by a probe.
    capacity: Option<f64>,
    /// Delivery samples taken on this path so far (see `rate_seen`).
    sampled: usize,
    /// On a slow link, since when acks came with no queue, and the acked
    /// wire bytes then (see `PROBE_GAIN`).
    probe_from: Option<(Instant, u64)>,
    /// The least round trip ever seen, ms: a slow link's own delay. Unlike
    /// `min_rtt` it is not learned again each span, as on a slow link this
    /// sender's own queue would be learned in.
    floor_rtt: Option<f64>,
    /// Datagrams in a row whose own time on the path was well over
    /// `floor_rtt`, and the least of their whole times (see `ABOVE_FLOOR`).
    above_floor: (u32, f64),
    /// Timed round trips in a row of 20 ms or more over a near floor, or,
    /// climbing, late ones (see `on_ack`), and the least of them.
    near_over: (u32, f64),
    /// What the round trips showed when the cap held a raise of the least
    /// round trip short of it (at most three times the floor before), and
    /// when: the rest is climbed to while they stay late (see `CLIMB_SPAN`).
    climb: Option<(f64, Instant)>,
    /// When the least round trip was last raised (see `REFLOOR_REST`).
    refloored_at: Option<Instant>,
    /// How long the datagrams acked alone lately took, ms (the last four),
    /// and such acks in a row that took well over the quickest of those
    /// before (see `waited`).
    took_lately: VecDeque<f64>,
    waited: u32,
    /// The path model's bottleneck rate (wire bytes a second; None when
    /// nothing waits on it) and the round trip windows are sized by, ms
    /// (`PathModel::window_rtt`), when the client reports arrivals
    /// (`set_path`): they stand in for the rate and floor this sender would
    /// read from its acks, which carry both ends' work. With them a slow
    /// link is one over a near path too.
    path: Option<(Option<f64>, Option<f64>)>,
    /// With no bottleneck in the path model, the least window (wire bytes)
    /// it allows: twice what the path took lately over its round trip, so
    /// a sender held by its window doubles each round trip (see
    /// `set_path_window`).
    path_window: u64,
    /// The link loses nothing (TCP): no timeout or duplicate-ack resends,
    /// which would only queue copies ahead of new data. A new connection
    /// `rewind`s instead.
    lossless: bool,
}

impl Sender {
    pub fn new(gen: u32) -> Self {
        Self {
            gen,
            base: 0,
            buf: VecDeque::new(),
            sent: 0,
            high: 0,
            rewound: 0,
            srtt: None,
            rttvar: 0.0,
            rto: INITIAL_RTO,
            timing: None,
            deadline: None,
            dup_acks: 0,
            resend_base: false,
            recover: None,
            probe: None,
            heard: false,
            timed_out: None,
            first: 0,
            window: WINDOW,
            min_rtt: None,
            span_min: None,
            last_rtt: None,
            limited: false,
            peak: 0,
            fresh: None,
            // Taken for a lossy path until some hundreds of sends show
            // otherwise (between the server's on and off marks), so the
            // first loss of an attachment costs no stall either.
            resent: 0.003,
            flight: VecDeque::new(),
            by_size: [None; 10],
            resend_seen: false,
            carried: 0,
            delivered: 0,
            delivered_wire: 0,
            last_ack: None,
            acked_at: VecDeque::new(),
            rate_span: None,
            rates: VecDeque::new(),
            capacity: None,
            sampled: 0,
            probe_from: None,
            floor_rtt: None,
            above_floor: (0, f64::MAX),
            near_over: (0, f64::MAX),
            climb: None,
            refloored_at: None,
            took_lately: VecDeque::new(),
            waited: 0,
            path: None,
            path_window: 0,
            lossless: false,
        }
    }

    /// The path's capacity when this sender has seen it below `SLOW_RATE`:
    /// a slow link. Seen as the rate acks came while datagrams queued, or,
    /// until they have, as the extra time large datagrams take over small
    /// ones.
    pub fn slow_rate(&self) -> Option<f64> {
        if self.path.is_some() {
            return self.rate_seen();
        }
        // Only over a far path (20 ms or more) and a clear difference (5 ms
        // or more): near, the client's own work on a large datagram (or a
        // timer's slack) would look like a slow link.
        if !self.floor_rtt.is_some_and(|floor| floor >= 20.0) {
            return None;
        }
        self.rate_seen()
    }

    /// The capacity measured, or seen by size, when it is below `SLOW_RATE`,
    /// over any path: what a datagram's time on the link is reckoned with
    /// (`on_ack`), also before a path that turned far is known to be far.
    fn rate_seen(&self) -> Option<f64> {
        if let Some((rate, _)) = self.path {
            return rate.map(|rate| rate.max(MIN_CAPACITY)).filter(|rate| *rate < SLOW_RATE);
        }
        // Until a few delivery samples were taken on this path (the first
        // ones of a session come low, the client busy with its start), the
        // larger of the two; then the samples alone, which follow a link
        // that got slower (the sizes' least times keep the fastest it was).
        let rate = match (self.capacity, self.size_rate()) {
            (Some(measured), Some(by_size)) if self.sampled < RATE_SAMPLES => Some(measured.max(by_size)),
            (measured, by_size) => measured.or(by_size),
        };
        // Never below what the path delivered lately while kept busy, nor
        // `MIN_CAPACITY`: an estimate off that far only starves the link.
        let delivered = self.busy_delivery_rate();
        rate.map(|rate| rate.max(delivered.unwrap_or(0.0)).max(MIN_CAPACITY)).filter(|rate| *rate < SLOW_RATE)
    }

    /// What the path delivered over its last two least round trips, wire
    /// bytes a second, when this sender kept it busy then.
    fn busy_delivery_rate(&self) -> Option<f64> {
        let floor = self.floor_rtt?;
        match self.recent_delivery(Instant::now())? {
            (delivered, true) => Some(delivered as f64 / (2.0 * floor / 1000.0)),
            _ => None,
        }
    }

    /// The retransmission timeout from now. On a slow link at least the
    /// least round trip plus the time the link takes for what is in
    /// flight (and a margin): there the last datagram of a few waits that
    /// long behind the others, longer than a timeout learned from single
    /// ones, and a timeout that fires too soon queues resends in front of
    /// everything after it.
    fn timeout(&self, now: Instant) -> Duration {
        // The path model's least round trip, when it speaks: a path that
        // turned far (LAN to RTT 100 ms) is rebased there within a round
        // trip, while this sender's own floor and timeout are the old
        // path's for longer, and timed out spuriously (twice in the first
        // 300 ms of LAN to 1 Mbit/s: a frame held 80 to 118 ms).
        let model = self.path.and_then(|(_, rtt)| rtt);
        let least = model.map_or(Duration::ZERO, |rtt| Duration::from_secs_f64((rtt * 1.25 + 30.0) / 1000.0)).min(SLOW_MAX_RTO);
        let floor = match (model, self.floor_rtt) {
            (Some(model), Some(own)) => Some(model.max(own)),
            (model, own) => model.or(own),
        };
        let timeout = match (self.slow_rate(), floor) {
            (Some(rate), Some(floor)) => {
                // What a slow link holds is at most twice its delivery per
                // round trip (and a datagram); more in flight (sent before
                // the rate was known) may be lost rather than waiting.
                let held = self.in_flight_wire().min(2 * self.slow_bdp().unwrap_or(0) + CHUNK as u64 + WIRE_OVERHEAD);
                let drain = held as f64 / rate * 1000.0;
                self.rto.max(Duration::from_secs_f64((floor + drain * 1.25 + 30.0) / 1000.0).min(SLOW_MAX_RTO))
            }
            (None, Some(floor)) => match self.recent_delivery(now) {
                // A far path whose acks slowed down (a link that turned
                // slow before its rate is known): what is in flight takes
                // longer than a round trip at the rate they came lately,
                // and a timeout before it is through would queue resends.
                Some((delivered, true)) if delivered > 0 => {
                    let drain = self.flight_wire() as f64 * 2.0 * floor / delivered as f64;
                    if drain > floor {
                        self.rto.max(Duration::from_secs_f64((floor + drain * 1.25 + 30.0) / 1000.0).min(SLOW_MAX_RTO))
                    } else {
                        self.rto
                    }
                }
                _ => self.rto,
            },
            _ => self.rto,
        };
        timeout.max(least)
    }

    /// What the path delivered over its last two least round trips, wire
    /// bytes, and whether this sender kept
    /// it busy then (a datagram's worth or more in flight: delivery the
    /// program held back, as keys, says little of the path). After a pause
    /// (nothing in flight since an ack that long ago) nothing: the ack
    /// clock starts again from `least_flight`, so a burst after it puts at
    /// most that in a link that may have got slower meanwhile, and grows as
    /// acks show the path takes more.
    fn recent_delivery(&self, now: Instant) -> Option<(u64, bool)> {
        // Over a near path too (two round trips of 5 ms at least): one that
        // turns slow (a LAN to a tethered link) is no different.
        let floor = self.floor_rtt?.max(5.0);
        let since = now.checked_sub(Duration::from_secs_f64(2.0 * floor / 1000.0))?;
        let &(at, last, after) = self.acked_at.back()?;
        if after == 0 && at <= since {
            return Some((0, false));
        }
        let &(_, then, busy) = self.acked_at.iter().rev().find(|(at, _, _)| *at <= since)?;
        Some((last - then, busy >= CHUNK as u64 + WIRE_OVERHEAD))
    }

    /// The least the ack clock lets in flight: `MIN_WINDOW` for each 100 ms
    /// of the least round trip. A far path carries little a round trip from
    /// `MIN_WINDOW` alone: a program drawing frames after a pause would
    /// wait a round trip each time a frame did not fit, for the first round
    /// trips of every burst.
    fn least_flight(&self) -> u64 {
        (MIN_WINDOW as f64 * (self.floor_rtt.unwrap_or(0.0) / 100.0).max(1.0)) as u64
    }

    /// How fast output drains now, wire bytes a second, to tell whether a
    /// backlog will take long before the link's capacity is known: a slow
    /// link's capacity, else what the ack clock lets through (the path's
    /// delivery over two least round trips, `MIN_WINDOW` at least: a guess
    /// on the low side, which starts the screens-only mode sooner).
    pub fn drain_rate(&self, now: Instant) -> Option<f64> {
        if let Some(rate) = self.slow_rate() {
            return Some(rate);
        }
        let floor = self.floor_rtt?;
        let (delivered, _) = self.recent_delivery(now)?;
        Some(delivered.max(MIN_WINDOW) as f64 / (2.0 * floor / 1000.0))
    }

    /// Wire bytes in flight as they went (a datagram compressed on its
    /// own carries more stream bytes than wire): up to `sent`, as after a
    /// go-back what lies past it is to go again, not on its way.
    fn flight_wire(&self) -> u64 {
        self.flight.iter().take_while(|(end, _, _, _)| *end <= self.sent).map(|(_, _, wire, _)| wire).sum()
    }

    /// The link's rate by size (see `by_size`): the most any larger size
    /// shows against the smallest, where the difference is clear (5 ms and
    /// 200 bytes or more; less is the client's own work, a timer's slack,
    /// or that noise over too few bytes).
    fn size_rate(&self) -> Option<f64> {
        // Against the quickest step, not the smallest: one late sample of a
        // small size must not make every larger one look fast.
        let &(small, a) = self.by_size.iter().flatten().min_by(|x, y| x.0.total_cmp(&y.0))?;
        self.by_size
            .iter()
            .flatten()
            .filter(|(took, wire)| *took >= small + 5.0 && *wire >= a + 200)
            .map(|(took, wire)| (wire - a) as f64 / ((took - small) / 1000.0))
            .reduce(f64::max)
    }

    /// What a slow link delivers in its least round trip.
    pub fn slow_bdp(&self) -> Option<u64> {
        let floor = self.path.and_then(|(_, rtt)| rtt).or(self.floor_rtt)?;
        Some((self.slow_rate()? * floor / 1000.0) as u64)
    }

    /// The path model's rate and least round trip (see `path`).
    pub fn set_path(&mut self, rate: Option<f64>, rtt: Option<f64>) {
        self.path = Some((rate, rtt));
    }

    /// The least window while the path model sees no bottleneck: twice
    /// `rate` (bytes a second) over the round trip.
    /// The delay-based window grows a quarter a round trip, too slow for
    /// 60 frames a second of a full screen over a far path; the model knows
    /// nothing waits.
    pub fn set_path_window(&mut self, delivered: f64, rtt_ms: f64) {
        self.path_window = (2.0 * delivered * rtt_ms / 1000.0) as u64 + 4 * CHUNK as u64;
    }

    /// Another path (the client moved): its capacity is learned again.
    pub fn forget_capacity(&mut self) {
        self.capacity = None;
        self.rates.clear();
        self.sampled = 0;
        self.probe_from = None;
        self.last_ack = None;
        self.rate_span = None;
        self.acked_at.clear();
        self.by_size = [None; 10];
        self.floor_rtt = None;
        self.above_floor = (0, f64::MAX);
        self.near_over = (0, f64::MAX);
        self.climb = None;
        self.took_lately.clear();
        self.waited = 0;
        // The path model speaks again once the new path's reports come
        // (over TCP never).
        self.path = None;
        self.path_window = 0;
    }

    pub fn set_lossless(&mut self, on: bool) {
        self.lossless = on;
    }

    pub fn push(&mut self, bytes: &[u8]) {
        self.buf.extend(bytes);
    }

    /// Bytes not yet acknowledged, sent or not.
    pub fn pending(&self) -> usize {
        self.buf.len()
    }

    pub fn end(&self) -> u64 {
        self.base + self.buf.len() as u64
    }

    pub fn acked(&self) -> u64 {
        self.base
    }

    pub fn rto(&self) -> Duration {
        self.rto
    }

    /// Bytes allowed in flight now.
    pub fn window(&self) -> u64 {
        // With no bottleneck in the path model, its window when larger.
        if matches!(self.path, Some((None, _))) {
            return self.window.max(self.path_window.min(MAX_WINDOW));
        }
        self.window
    }

    /// How much longer the last round trip took than the least: the queue
    /// this stream's data waits in on the path.
    pub fn queueing(&self) -> Duration {
        match (self.last_rtt, self.min_rtt) {
            (Some(last), Some(least)) => Duration::from_micros(((last - least).max(0.0) * 1000.0) as u64),
            _ => Duration::ZERO,
        }
    }

    /// About how many datagrams the path loses lately: the share of sends
    /// that were resends.
    pub fn loss(&self) -> f64 {
        self.resent
    }

    /// Whether a queue grows on the path, so it has no room to spare: the
    /// last two datagrams acked alone took well over the quickest of the
    /// four before. Against those, not the least round trip: a path that
    /// turned far keeps a near one for a while, and every ack would look
    /// late; here only its first few do. (Growth against the ack before
    /// alone, by 0 to 10 ms, missed the queue of a LAN turned 256 kbit/s:
    /// 26 fps against 30.)
    pub fn waited(&self) -> bool {
        self.waited >= 2
    }

    /// Whether a resend went out since the last call.
    pub fn take_resend(&mut self) -> bool {
        std::mem::take(&mut self.resend_seen)
    }

    /// Bytes sent and not yet acknowledged.
    pub fn in_flight(&self) -> u64 {
        self.sent - self.base
    }

    /// What is in flight as it went on the wire (each new datagram with its
    /// own bytes, `WIRE_OVERHEAD`).
    pub fn in_flight_wire(&self) -> u64 {
        let on_way = self.flight.iter().take_while(|(end, _, _, _)| *end <= self.sent).count();
        self.in_flight() + on_way as u64 * WIRE_OVERHEAD
    }

    /// Gives back what the buffers grew to beyond what they hold now: a
    /// burst of output (up to `OUTPUT_BACKLOG` in the server) leaves its
    /// high-water mark allocated for the life of the session otherwise.
    pub fn trim(&mut self) {
        shrink(&mut self.buf);
        shrink(&mut self.flight);
        shrink(&mut self.acked_at);
    }

    /// Lets this generation use the whole window at once: its caller paces
    /// it otherwise (the screens-only mode sends small updates).
    pub fn forget_carried(&mut self) {
        self.carried = 0;
    }

    /// Whether all that was pushed has gone out at least once.
    pub fn all_sent(&self) -> bool {
        self.sent >= self.end()
    }

    /// Whether the peer has what this generation started with.
    pub fn first_acked(&self) -> bool {
        self.base >= self.first
    }

    pub fn srtt(&self) -> Option<Duration> {
        self.srtt.map(|ms| Duration::from_micros((ms * 1000.0) as u64))
    }

    /// Starts a new generation whose stream is `bytes`.
    pub fn reset(&mut self, bytes: &[u8]) {
        self.last_ack = None;
        self.rate_span = None;
        self.probe_from = None;
        self.carried = (self.carried + (self.sent - self.base)).min(self.window);
        self.flight.clear();
        self.gen = self.gen.wrapping_add(1);
        self.base = 0;
        self.sent = 0;
        self.high = 0;
        self.rewound = 0;
        // A burst can leave megabytes of capacity behind; give it back.
        self.buf = VecDeque::from(bytes.to_vec());
        self.first = bytes.len() as u64;
        self.timing = None;
        self.deadline = None;
        self.dup_acks = 0;
        self.resend_base = false;
        self.recover = None;
        self.probe = None;
        self.timed_out = None;
    }

    /// Applies the peer's cumulative ack. `carries_data` says the packet
    /// was sent for its own data rather than to ack ours, so a repeated ack
    /// in it is no sign of loss.
    pub fn on_ack(&mut self, gen: u32, offset: u64, now: Instant, carries_data: bool) -> bool {
        if gen != self.gen {
            return false;
        }
        self.heard = true;
        if offset == self.base && self.sent > self.base {
            if carries_data || self.lossless {
                return false;
            }
            self.dup_acks += 1;
            // Early retransmit (RFC 5827): with only a few datagrams out (a
            // thin stream, one per frame) fewer duplicates are ever coming.
            // After a rewind the resends below `high` are not counted: then
            // no early retransmit.
            let out = if self.flight.is_empty() { 4 } else { self.flight.len() as u32 };
            let enough = out.saturating_sub(1).clamp(1, 3);
            if self.dup_acks >= enough && self.recover.is_none() {
                self.enter_recovery(now);
            }
            return false;
        }
        // Up to the most ever sent: after a rewind, originals still arrive.
        if offset <= self.base || offset > self.high {
            return false;
        }
        self.sent = self.sent.max(offset);
        self.carried = 0;
        self.dup_acks = 0;
        // An ack sooner after a timeout than any round trip has taken
        // answers the original send, not the resend: the timeout was
        // spurious (Eifel-style undo).
        let spurious = self.timed_out.take().is_some_and(|(at, rto, window)| {
            let quick = self.min_rtt.is_some_and(|least| now.duration_since(at).as_secs_f64() * 1000.0 < least / 2.0);
            if quick {
                self.rto = rto;
                self.window = window;
            }
            quick
        });
        // Not while repairing (as Karn's rule): a resend's ack would add its
        // wait to the original's time. Taken before a go-back below ends
        // the repair: the ack that starts it answers a resend too, as do
        // those of what it sends again (sent before, timed from then).
        let repairing = self.recover.is_some() || self.probe.is_some() || offset <= self.rewound;
        if let Some(end) = self.probe.filter(|end| offset >= *end) {
            self.probe = None;
            // After a spurious timeout the originals are still arriving;
            // holes, if any, are repaired one partial ack at a time.
            if offset == end && !spurious {
                // The receiver holds nothing right after the resent chunk:
                // the loss runs on, so everything after it goes again.
                self.sent = offset;
                self.recover = None;
                self.rewound = self.high;
            }
        }
        let advanced = (offset - self.base) as usize;
        // Datagrams acked now that were sent well over the least round trip
        // ago waited on the path: it is busy, and how fast acks come is how
        // fast it delivers.
        let mut queued = None;
        let late = self.floor_rtt.map(|floor| Duration::from_secs_f64(floor * 1.25 / 1000.0) + Duration::from_millis(5));
        // On a slow link a datagram's own time on the wire is no wait.
        let known = self.rate_seen();
        let mut popped = Vec::new();
        while let Some(&(_, at, wire, ahead)) = self.flight.front().filter(|(end, _, _, _)| *end <= offset) {
            self.flight.pop_front();
            popped.push((at, wire, ahead));
            self.delivered_wire += wire;
            let own = known.map_or(Duration::ZERO, |rate| Duration::from_secs_f64(wire as f64 / rate));
            queued = Some(queued.unwrap_or(true) && late.is_some_and(|late| now.duration_since(at) > late + own));
        }
        // One datagram an ack: one that covers several (a loss's hole
        // filled) waited on the repair, not on a queue. A repair's acks of
        // one datagram count, its wait in them: a LAN turned slow repairs
        // spurious timeouts for a second while its queue grows, and lossy
        // paths measured no worse for it.
        if let [(at, _, _)] = popped[..] {
            let took = now.duration_since(at).as_secs_f64() * 1000.0;
            let quickest = self.took_lately.iter().copied().reduce(f64::min);
            let grew = quickest.is_some_and(|least| took > 1.25 * least + 10.0);
            self.waited = if grew { self.waited.saturating_add(1) } else { 0 };
            self.took_lately.push_back(took);
            if self.took_lately.len() > 4 {
                self.took_lately.pop_front();
            }
        }
        let mut refloor = None;
        if let ([(at, wire, ahead)], false) = (&popped[..], repairing) {
            // One datagram acked alone: its time on the way, by size. Sent
            // onto an empty path, or not long on the way (three least round
            // trips): one that waited seconds behind others (a link that
            // turned slow) would read as a link of a few bytes a second.
            let took = now.duration_since(*at).as_secs_f64() * 1000.0;
            if *ahead == *wire || self.floor_rtt.is_some_and(|floor| took <= 3.0 * floor) {
                let step = &mut self.by_size[(*wire as usize / 128).min(9)];
                if step.is_none_or(|(least, _)| took < least) {
                    *step = Some((took, *wire));
                }
            }
        }
        if let (Some(&(at, wire, ahead)), false) = (popped.first(), repairing) {
            let took = now.duration_since(at).as_secs_f64() * 1000.0;
            // The first acked datagram's own time on the path: less the
            // time what went before it takes on the link at the known
            // rate, or, with none known, that of a small one sent alone (a
            // key's echo). Over twice the floor again and again, the path's
            // delay rose: a floor from an earlier, nearer route reads
            // every ack as late. (Twice, not `late`: a capacity a probe set
            // a little high, or jitter, leaves some time unaccounted for.)
            // At the slower of the rate known and the latest delivery
            // samples: a link that got slower (a capacity from before it)
            // shows it in them at once, and queues what goes; a path that
            // got longer does not (after one pause its acks come as
            // before), so that wait is path.
            let latest = {
                let mut last: Vec<f64> = self.rates.iter().rev().take(3).map(|(_, rate)| *rate).collect();
                last.sort_by(f64::total_cmp);
                (last.len() == 3).then(|| last[1])
            };
            let known = known.map(|rate| latest.map_or(rate, |latest| rate.min(latest)));
            if known.is_some() || (ahead == wire && wire < 2 * 128) {
                // Only what is past a round trip's delivery waited in a
                // queue; the rest was on its way (a far path holds much).
                let floor = self.floor_rtt.unwrap_or(0.0);
                let own = took - known.map_or(0.0, |rate| (ahead as f64 / rate.max(1.0) * 1000.0 - floor).max(0.0));
                let far_over = self.floor_rtt.is_some_and(|floor| own > 2.0 * floor + 5.0);
                // The new floor is the least of their whole times: too high
                // it comes down with the next quicker sample, too low it
                // would read every ack as late again.
                let (count, least) = self.above_floor;
                self.above_floor = if far_over { (count + 1, least.min(took)) } else { (0, f64::MAX) };
                if self.above_floor.0 >= ABOVE_FLOOR {
                    refloor = Some(self.above_floor.1);
                }
            }
        }
        self.buf.drain(..advanced);
        self.base = offset;
        self.delivered += advanced as u64;
        self.acked_at.push_back((now, self.delivered_wire, self.flight_wire()));
        let keep = Duration::from_secs_f64((2.0 * self.floor_rtt.unwrap_or(0.0) / 1000.0 + 0.1).max(1.0));
        // The newest of those older than that stays: after a stall (acks
        // stopped behind a queue) the ack clock measures from it, and does
        // not take the stall's end for a start.
        while self.acked_at.get(1).is_some_and(|(at, _, _)| now.duration_since(*at) > keep) {
            self.acked_at.pop_front();
        }
        // A datagram that waited on the path, and was on its way when the
        // ack before came, left it right after the one before: acks of
        // such datagrams in a row come as fast as the link delivers, so
        // their wire bytes over the time since the ack before the first
        // are a delivery sample with no idle time in it. (Not while
        // repairing: a resend's ack would add its wait.)
        if !popped.is_empty() {
            let first_at = popped[0].0;
            let busy = queued == Some(true) && !repairing && self.last_ack.is_some_and(|prev| first_at < prev);
            let wire: u64 = popped.iter().map(|(_, wire, _)| wire).sum();
            self.rate_span = match (busy, self.rate_span, self.last_ack) {
                (true, Some((from, sum)), _) => Some((from, sum + wire)),
                (true, None, Some(prev)) => Some((prev, wire)),
                _ => None,
            };
            if let Some((from, sum)) = self.rate_span.filter(|(from, _)| now.duration_since(*from) >= RATE_SAMPLE_SPAN) {
                self.rates.push_back((now, sum as f64 / now.duration_since(from).as_secs_f64()));
                self.sampled += 1;
                self.rate_span = Some((now, 0));
                while self.rates.len() > RATE_SAMPLES || self.rates.front().is_some_and(|(at, _)| now.duration_since(*at) > RATE_SPAN) {
                    self.rates.pop_front();
                }
                if self.rates.len() >= MIN_RATE_SAMPLES {
                    let mut sorted: Vec<f64> = self.rates.iter().map(|(_, rate)| *rate).collect();
                    sorted.sort_by(f64::total_cmp);
                    self.capacity = Some(sorted[sorted.len() / 2]);
                }
            }
            self.last_ack = Some(now);
        }
        // Sent near a slow link's capacity for a round trip with no queue: it delivers more than measured (it got faster, or the
        // samples were low).
        let floor = self.floor_rtt.unwrap_or(100.0).max(50.0);
        if let (Some(rate), Some(queue)) = (self.slow_rate(), queued) {
            self.probe_from = match self.probe_from {
                _ if queue => None,
                None => Some((now, self.delivered_wire)),
                Some((from, count)) if now.duration_since(from).as_secs_f64() * 1000.0 >= floor => {
                    let took = (self.delivered_wire - count) as f64 / now.duration_since(from).as_secs_f64();
                    if took >= 0.8 * rate {
                        self.capacity = Some(rate * PROBE_GAIN);
                        for (_, sample) in self.rates.iter_mut() {
                            *sample *= PROBE_GAIN;
                        }
                    }
                    Some((now, self.delivered_wire))
                }
                kept => kept,
            };
        }
        // A near floor (a LAN) that the path left for a far one: three
        // round trips in a row of 20 ms and more, timed (one datagram a
        // round trip, never a resend: acks a stall held back come together
        // and are one). The rule above needs a rate, and what an ack clock
        // held to a near floor lets through reads as a slow link (every ack
        // late, the waits between round trips in its samples): LAN to RTT
        // 300 ms stayed at 6 fps. Should it be a slow link's queue, the
        // floor is high by that queue and comes down with quicker timed
        // round trips: in the lab, LAN to 1 Mbit/s and RTT 100 ms took it to
        // 148 ms, then to 107 ms within 3 s.
        //
        // The same, after a raise the cap held short (`climb`): three timed
        // round trips in a row late (over 1.25 times the floor and 5 ms) go
        // on to what they and the raise showed, three times the floor
        // before the raise at most. Else the rest of a tripled path
        // (1 Mbit/s and RTT 100 ms to 300 ms: 106 ms raised to 211) is
        // under what raises it by the rule above (twice the floor past the
        // queue a slow link's window holds, about a floor) and over what
        // leaves acks on time, so the capacity is never probed up either:
        // it stayed at 4 to 8 fps. A slow link's own queue passes as late
        // too, so the climb is bounded, and a round trip on time ends it.
        self.climb = self.climb.filter(|(_, since)| now.duration_since(*since) < CLIMB_SPAN);
        let mut climbed = false;
        if let Some((_, at)) = self.timing.filter(|(end, _)| offset >= *end) {
            if let Some(floor) = self.floor_rtt.filter(|floor| *floor < 20.0 || self.climb.is_some()) {
                let r = now.duration_since(at).as_secs_f64() * 1000.0;
                let (count, least) = self.near_over;
                let late = match self.climb {
                    Some(_) => r > 1.25 * floor + 5.0,
                    None => r >= 20.0 && r > 2.0 * floor + 5.0,
                };
                self.near_over = if late { (count + 1, least.min(r)) } else { (0, f64::MAX) };
                if !late {
                    self.climb = None;
                }
                if self.near_over.0 >= ABOVE_FLOOR {
                    climbed = self.climb.is_some();
                    let least = self.climb.map_or(self.near_over.1, |(shown, _)| self.near_over.1.min(shown));
                    refloor = Some(refloor.map_or(least, |other: f64| other.min(least)));
                }
            }
        }
        let old = self.floor_rtt.filter(|old| *old >= 20.0);
        // A climb's step rests the least: what was there before the rate was
        // known went in the first raise.
        let rest = if climbed {
            REFLOOR_REST
        } else {
            REFLOOR_REST.max(Duration::from_secs_f64(10.0 * self.floor_rtt.unwrap_or(0.0) / 1000.0))
        };
        let rested = self.refloored_at.is_none_or(|at| now.duration_since(at) >= rest);
        if let Some((shown, floor)) = refloor.filter(|_| rested).map(|floor| (floor, old.map_or(floor, |old| floor.min(2.0 * old)))) {
            self.refloored_at = Some(now);
            // The path's delay rose: a capacity measured against the old
            // floor counted every ack as late, so it is learned again. From
            // a far floor at most twice it at a time: a wait in a queue
            // taken for path must not raise it far (from a near one, as
            // LAN to WAN, any). What the cap held back, to three times the
            // floor before, is climbed to while round trips stay late.
            let bound = if climbed { shown } else { old.map_or(shown, |old| shown.min(3.0 * old)) };
            self.climb = (floor < bound).then_some((bound, now));
            self.floor_rtt = Some(floor);
            self.min_rtt = Some(floor);
            self.span_min = Some((floor, now));
            self.capacity = None;
            self.rates.clear();
            self.sampled = 0;
            self.rate_span = None;
            self.probe_from = None;
            self.by_size = [None; 10];
            self.above_floor = (0, f64::MAX);
            self.near_over = (0, f64::MAX);
        }

        if let Some((end, at)) = self.timing {
            if offset >= end {
                self.sample(now.duration_since(at), now);
                self.timing = None;
            }
        }
        if let Some(point) = self.recover {
            if offset < point {
                self.resend_base = true;
            } else {
                self.recover = None;
            }
        }
        self.deadline = if self.sent > self.base { Some(now + self.timeout(now)) } else { None };
        true
    }

    fn enter_recovery(&mut self, now: Instant) {
        self.window = (self.window / 2).max(self.window.min(WINDOW));
        self.recover = Some(self.sent);
        self.resend_base = true;
        self.timing = None;
        self.deadline = Some(now + self.timeout(now));
    }

    fn sample(&mut self, rtt: Duration, now: Instant) {
        let r = rtt.as_secs_f64() * 1000.0;
        match self.srtt {
            None => {
                self.srtt = Some(r);
                self.rttvar = r / 2.0;
            }
            Some(srtt) => {
                self.rttvar = 0.75 * self.rttvar + 0.25 * (srtt - r).abs();
                self.srtt = Some(0.875 * srtt + 0.125 * r);
            }
        }
        let rto = self.srtt.unwrap_or(r) + 4.0 * self.rttvar;
        self.rto = Duration::from_micros((rto * 1000.0) as u64).clamp(MIN_RTO, MAX_RTO);
        // Delay-based window: grow by a quarter per round trip while the
        // window, not the data, held sending back and nothing queued up;
        // shrink by a quarter once a queue builds.
        let (span, began) = self.span_min.map_or((r, now), |(m, began)| (m.min(r), began));
        let least = if now.duration_since(began) >= MIN_RTT_SPAN {
            // A span ended: its least is the base from now on.
            self.span_min = Some((r, now));
            span
        } else {
            self.span_min = Some((span, began));
            self.min_rtt.map_or(r, |m| m.min(r))
        };
        self.min_rtt = Some(least);
        // A much shorter least round trip is another path: the least times
        // by size from the longer one would read as a slow link.
        if self.floor_rtt.is_some_and(|floor| r < 0.8 * floor - 5.0) {
            self.by_size = [None; 10];
        }
        self.floor_rtt = Some(self.floor_rtt.map_or(r, |floor| floor.min(r)));
        self.last_rtt = Some(r);
        let target = (least / 4.0).max(QUEUE_TARGET_MS);
        let limited = std::mem::take(&mut self.limited);
        // Bulk in flight, not keystroke echoes: a queue may be this sender's.
        let busy = std::mem::take(&mut self.peak) >= MIN_WINDOW / 2;
        if r - least > 2.0 * target {
            // Below `WINDOW` only when this sender filled the queue.
            let floor = if busy { MIN_WINDOW } else { WINDOW };
            self.window = (self.window - self.window / 4).max(floor.min(self.window));
        } else if r - least < target && limited {
            self.window = (self.window + self.window / 4).min(MAX_WINDOW);
        }
    }

    /// Next datagram to transmit, if the window and timers allow one.
    pub fn next_chunk(&mut self, now: Instant) -> Option<(u64, Vec<u8>)> {
        self.next_chunk_max(now, CHUNK)
    }

    /// Gives back the tail of the chunk `next_chunk_max` just returned
    /// (`len` bytes at `offset`), keeping `keep`: it did not fit a datagram.
    pub fn take_back(&mut self, offset: u64, len: usize, keep: usize) {
        let end = offset + len as u64;
        // A resend ends where its datagram did: its ack is read against that.
        if offset == self.base && keep < len && self.probe == Some(end) {
            self.probe = Some(offset + keep as u64);
        }
        // Only new data just handed out: a resend did not move `sent`.
        let Some((fresh, high, peak)) = self.fresh.filter(|(fresh, _, _)| *fresh == offset) else {
            return;
        };
        if self.sent != end || keep >= len || fresh != offset {
            return;
        }
        self.sent = offset + keep as u64;
        if self.flight.back().map(|(last, _, _, _)| *last) == Some(end) {
            let last = self.flight.back_mut().expect("last");
            last.0 = self.sent;
            // A compressed datagram carried more stream bytes than wire.
            let wire = (keep as u64).min(CHUNK as u64) + WIRE_OVERHEAD;
            last.3 = last.3 - last.2 + wire;
            last.2 = wire;
        }
        self.high = high.max(self.sent);
        self.peak = peak.max(self.sent - self.base);
        if let Some((timed, at)) = self.timing {
            if timed == end {
                self.timing = Some((self.sent, at));
            }
        }
    }

    /// `next_chunk` with up to `max` bytes (more than `CHUNK` when the
    /// caller compresses them into one datagram).
    pub fn next_chunk_max(&mut self, now: Instant, max: usize) -> Option<(u64, Vec<u8>)> {
        if !self.lossless && self.deadline.is_some_and(|deadline| now >= deadline) && self.sent > self.base {
            self.timed_out = Some((now, self.rto, self.window));
            self.rto = (self.rto * 2).min(MAX_RTO);
            self.window = self.window.min(WINDOW);
            self.timing = None;
            self.dup_acks = 0;
            if self.recover.is_some() && std::mem::take(&mut self.heard) {
                // A repair timed out too, while acks came: the loss is
                // heavy, so everything unacked goes out again.
                self.sent = self.base;
                self.rewound = self.high;
                self.recover = None;
                self.probe = None;
                self.deadline = None;
            } else if self.recover.is_some() {
                // A repair timed out with no ack at all: its probe goes
                // again, alone. Everything at once would queue behind what
                // holds the path up (a link that turned slow, full of an
                // earlier generation).
                self.resend_base = true;
                self.deadline = Some(now + self.timeout(now));
            } else {
                self.heard = false;
                self.resend_base = true;
                self.recover = Some(self.sent);
                self.probe = Some(self.base + (self.sent - self.base).min(CHUNK as u64));
                self.deadline = Some(now + self.timeout(now));
            }
        }
        self.fresh = None;
        if self.resend_base && self.sent > self.base {
            self.resend_base = false;
            // A probe (a timeout's, or a repair resent with no ack between)
            // is one plain chunk: its ack tells what follows.
            let max = if self.probe.is_some() { CHUNK } else { max };
            let len = (self.sent - self.base).min(max as u64) as usize;
            self.resent = self.resent * 0.998 + 0.002;
            self.resend_seen = true;
            // A delivery sample across a loss would count the wait for the
            // resend as the link's: the next starts after the repair.
            self.rate_span = None;
            self.last_ack = None;
            // Each repair, not only a timeout's: one chunk a round trip
            // would take seconds over a run of losses.
            self.probe = Some(self.base + len as u64);
            let chunk: Vec<u8> = self.buf.range(..len).copied().collect();
            return Some((self.base, chunk));
        }
        self.resend_base = false;
        let end = self.end();
        if self.sent >= end {
            return None;
        }
        let mut window = self.window.saturating_sub(self.carried).max(CHUNK as u64);
        if matches!(self.path, Some((None, _))) {
            window = window.max(self.path_window.min(MAX_WINDOW));
        }
        if let Some(bdp) = self.slow_bdp() {
            // A slow link: twice its delivery per round trip keeps it busy
            // with a queue of at most one round trip.
            window = window.min((2 * bdp).max(CHUNK as u64));
        } else if let Some((delivered, _)) = self.recent_delivery(now) {
            // The ack clock: no more in flight (wire bytes)
            // than the path delivered over its last two least round trips
            // (twice a round trip's delivery), so a link that turns slow
            // (acks slow down or stop) is not handed a window to queue
            // before its rate is known. After a pause it starts again from
            // `least_flight`; at a start (no ack yet) the window alone holds.
            // The path model's window too, when it sees no bottleneck.
            let open = if matches!(self.path, Some((None, _))) { self.path_window } else { 0 };
            if self.flight_wire() >= delivered.max(self.least_flight()).max(open) {
                self.limited = true;
                return None;
            }
        }
        if self.sent - self.base >= window {
            self.limited = true;
            return None;
        }
        let len = (end - self.sent).min(max as u64).min(window - (self.sent - self.base)) as usize;
        let start = (self.sent - self.base) as usize;
        let chunk: Vec<u8> = self.buf.range(start..start + len).copied().collect();
        let offset = self.sent;
        let high = self.high;
        self.fresh = Some((offset, self.high, self.peak));
        // Going back over sent bytes (after a timeout) is a resend too.
        self.resent = self.resent * 0.998 + if offset < self.high { 0.002 } else { 0.0 };
        if offset < self.high {
            self.rate_span = None;
            self.last_ack = None;
        }
        self.sent += len as u64;
        self.peak = self.peak.max(self.sent - self.base);
        // New data, also the part of a go-back-N chunk past what went before.
        if self.sent > high {
            let wire = (len as u64).min(CHUNK as u64) + WIRE_OVERHEAD;
            // What is ahead of it on the link: this generation's in flight,
            // and what earlier ones left there.
            self.flight.push_back((self.sent, now, wire, self.in_flight_wire() + WIRE_OVERHEAD + self.carried));
        }
        let fresh = offset >= self.high;
        self.high = self.high.max(self.sent);
        if self.timing.is_none() && fresh && self.recover.is_none() {
            self.timing = Some((self.sent, now));

        }
        if self.deadline.is_none() {
            self.deadline = Some(now + self.timeout(now));
        }
        Some((offset, chunk))
    }

    /// When the retransmission timer fires, if anything is in flight.
    pub fn deadline(&self) -> Option<Instant> {
        if self.sent > self.base && !self.lossless {
            self.deadline
        } else {
            None
        }
    }

    /// Forgets the in-flight state so everything unacked goes out again now,
    /// used after the peer was unreachable for a while.
    pub fn rewind(&mut self) {
        self.last_ack = None;
        self.acked_at.clear();
        self.rate_span = None;
        self.probe_from = None;
        self.carried = 0;
        self.flight.clear();
        self.sent = self.base;
        self.timing = None;
        self.deadline = None;
        self.dup_acks = 0;
        self.resend_base = false;
        self.recover = None;
        self.probe = None;
        self.timed_out = None;
    }
}

/// Bytes a receiver holds past a gap, waiting for the gap to be filled.
const MAX_AHEAD: u64 = 2 * MAX_WINDOW;

pub struct Receiver {
    pub gen: u32,
    pub next: u64,
    ahead: std::collections::BTreeMap<u64, Vec<u8>>,
    ahead_bytes: usize,
}

impl Receiver {
    pub fn new(gen: u32) -> Self {
        Self {
            gen,
            next: 0,
            ahead: std::collections::BTreeMap::new(),
            ahead_bytes: 0,
        }
    }

    /// The bytes that become deliverable in order, at most `limit` of them.
    /// Datagrams past a gap are held until the gap fills; datagrams from an
    /// older generation are ignored.
    pub fn accept(&mut self, gen: u32, offset: u64, data: &[u8], limit: usize) -> Vec<u8> {
        if gen != self.gen {
            if gen.wrapping_sub(self.gen) as i32 <= 0 {
                return Vec::new();
            }
            self.gen = gen;
            self.next = 0;
            self.ahead.clear();
            self.ahead_bytes = 0;
        }
        let Some(end) = offset.checked_add(data.len() as u64) else {
            return Vec::new();
        };
        if end <= self.next || data.is_empty() {
            return Vec::new();
        }
        if offset > self.next {
            if offset - self.next < MAX_AHEAD && self.ahead_bytes + data.len() <= MAX_AHEAD as usize {
                self.hold(offset, data.to_vec());
            }
            return Vec::new();
        }
        let mut out = Vec::new();
        self.take(offset, data, limit, &mut out);
        while out.len() < limit {
            let Some((&at, _)) = self.ahead.first_key_value() else {
                break;
            };
            if at > self.next {
                break;
            }
            let chunk = self.ahead.remove(&at).expect("ahead chunk");
            self.ahead_bytes -= chunk.len();
            if at + chunk.len() as u64 > self.next {
                let before = self.next;
                self.take(at, &chunk, limit - out.len(), &mut out);
                let used = (self.next - at) as usize;
                if used < chunk.len() && self.next > before {
                    self.hold(self.next, chunk[used..].to_vec());
                }
            }
        }
        out
    }

    /// Keeps the longer of two chunks held at the same offset, counting bytes exactly.
    /// Bytes held past a gap.
    pub fn held(&self) -> usize {
        self.ahead_bytes
    }

    fn hold(&mut self, offset: u64, chunk: Vec<u8>) {
        if self.ahead.get(&offset).is_some_and(|old| old.len() >= chunk.len()) {
            return;
        }
        self.ahead_bytes += chunk.len();
        if let Some(old) = self.ahead.insert(offset, chunk) {
            self.ahead_bytes -= old.len();
        }
    }

    fn take(&mut self, offset: u64, data: &[u8], limit: usize, out: &mut Vec<u8>) {
        let skip = (self.next - offset) as usize;
        let take = (data.len() - skip).min(limit);
        out.extend_from_slice(&data[skip..skip + take]);
        self.next += take as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_seal_open_and_codecs() {
        let key = Key::from_token("t");
        let bytes = seal(&key, Dir::ToClient, KIND_STREAM, 42, b"payload");
        assert_eq!(peek(&bytes), Some((KIND_STREAM, 42)));
        let packet = open(&key, Dir::ToClient, &bytes).unwrap();
        assert_eq!(packet.payload, b"payload");
        assert!(open(&key, Dir::ToServer, &bytes).is_none(), "a reflected packet is rejected");
        assert!(open(&Key::from_token("x"), Dir::ToClient, &bytes).is_none());
        let mut forged = bytes.clone();
        forged[5] ^= 1;
        assert!(open(&key, Dir::ToClient, &forged).is_none());
        let first = open(&key, Dir::ToClient, &seal(&key, Dir::ToClient, KIND_STREAM, 1, b"")).unwrap();
        let second = open(&key, Dir::ToClient, &seal(&key, Dir::ToClient, KIND_STREAM, 1, b"")).unwrap();
        let mut window = ReplayWindow::new();
        assert!(window.check(&second.nonce));
        assert!(window.check(&first.nonce), "reordered but new");
        assert!(!window.check(&first.nonce), "replayed");
        assert!(!window.check(&second.nonce), "replayed");
        let mut other = first.nonce;
        other[0] ^= 1;
        assert!(!window.check(&other), "another sender");

        let hello = Hello {
            time_ms: 5,
            host: "h".into(),
            action: Action::Attach(9),
            cols: 100,
            rows: 30,
            env: vec!["TERM=xterm".into()],
            command: vec!["ls".into(), "-l".into()],
            forwards: vec!["127.0.0.1:80:h:81".into()],
            client_pub: [7; 32],
            features: FEATURE_DEFLATE,
        };
        assert_eq!(Hello::decode(&hello.encode()).as_ref(), Some(&hello));
        let kill = Hello {
            action: Action::Kill("all".into()),
            ..hello
        };
        assert_eq!(Hello::decode(&kill.encode()), Some(kill));
        let ready = Ready {
            echo: [1; 24],
            attach: 9,
            server_pub: [2; 32],
            server: "rsh 1".into(),
        };
        assert_eq!(Ready::decode(&ready.encode()), Some(ready));
        assert_eq!(decode_denied(&encode_denied(&[3; 24], "no")), Some(([3; 24], "no".to_string())));
        let attached = Attached {
            session: 4,
            label: "l".into(),
            created: true,
            notes: vec!["n".into()],
            killed: vec![1, 2],
            denied: String::new(),
        };
        assert_eq!(Attached::decode(&attached.encode()), Some(attached));
        let info = SessionInfo {
            id: 1,
            host: "h".into(),
            label: "l".into(),
            alive: true,
            attached: false,
            cols: 80,
            rows: 24,
            created_unix: 3,
            idle_secs: 4,
            command: String::new(),
            title: "t".into(),
        };
        let tmux = TmuxInfo { name: "main".into(), clients: 2, cols: 80, rows: 24, idle_secs: 9, session: 123456 };
        let listing = Listing { sessions: vec![info.clone()], server: "rsh 1".into(), tmux: vec![tmux] };
        assert_eq!(decode_list(&encode_list(&[5; 24], &listing.sessions, "rsh 1", &listing.tmux)), Some(([5; 24], listing.clone())));
        // A reply from a server that predates tmux listing ends after the version line.
        let mut old = encode_list(&[5; 24], &listing.sessions, "rsh 1", &[]);
        old.truncate(old.len() - 2);
        assert_eq!(decode_list(&old), Some(([5; 24], Listing { tmux: Vec::new(), ..listing })));
        let stream = ServerStream {
            flags: FLAG_EXITED,
            in_ack: 2,
            out_gen: 3,
            out_offset: 4,
            exit_code: -1,
            end_offset: 6,
            pn: None,
            data: b"xy",
        };
        assert_eq!(ServerStream::decode(&stream.encode()), Some(stream.clone()));
        let numbered = ServerStream { pn: Some(7), ..stream };
        assert_eq!(ServerStream::decode(&numbered.encode()).map(|s| s.flags & FLAG_NUMBERED != 0), Some(true));
        assert_eq!(ServerStream::decode(&numbered.encode()).and_then(|s| s.pn), Some(7));
        let feedback = Feedback { sent_us: 9, arrivals: vec![(1, 100), (2, 140)] }.encode();
        let client = ClientStream { flags: 0, cols: 80, rows: 24, out_gen: 1, out_ack: 2, in_offset: 3, feedback: &feedback, data: b"ab" };
        let encoded = client.encode();
        let back = ClientStream::decode(&encoded).unwrap();
        assert_eq!((back.feedback, back.data), (&feedback[..], &b"ab"[..]));
        assert_eq!(Feedback::decode(back.feedback), Some(Feedback { sent_us: 9, arrivals: vec![(1, 100), (2, 140)] }));
    }

    #[test]
    fn the_timeout_is_no_shorter_than_the_path_models_round_trip() {
        let start = Instant::now();
        let mut tx = Sender::new(1);
        tx.push(&[0; 100]);
        tx.set_path(None, Some(300.0));
        assert!(tx.next_chunk(start).is_some());
        let deadline = tx.deadline().expect("a timeout");
        assert!(deadline >= start + Duration::from_millis(400), "{:?}", deadline - start);
    }

    #[test]
    fn sender_windows_acks_and_retransmits() {
        let start = Instant::now();
        let mut tx = Sender::new(1);
        let mut rx = Receiver::new(1);
        let data: Vec<u8> = (0..200_000u32).map(|i| i as u8).collect();
        tx.push(&data);
        let mut flight = Vec::new();
        while let Some(chunk) = tx.next_chunk(start) {
            flight.push(chunk);
        }
        assert_eq!(flight.len() as u64, WINDOW / CHUNK as u64);
        let mut got = Vec::new();
        for (index, (offset, chunk)) in flight.iter().enumerate() {
            if index == 3 {
                continue;
            }
            got.extend(rx.accept(1, *offset, chunk, usize::MAX));
        }
        assert_eq!(got.len(), 3 * CHUNK);
        assert_eq!(rx.next, 3 * CHUNK as u64);
        assert!(tx.on_ack(1, rx.next, start + Duration::from_millis(20), false));
        for _ in 0..5 {
            assert!(!tx.on_ack(1, rx.next, start + Duration::from_millis(21), true));
        }
        let mut fresh = Vec::new();
        while let Some(chunk) = tx.next_chunk(start + Duration::from_millis(21)) {
            assert!(chunk.0 > rx.next, "data packets are not duplicate acks");
            fresh.push(chunk);
        }
        for _ in 0..3 {
            assert!(!tx.on_ack(1, rx.next, start + Duration::from_millis(21), false));
        }
        let (offset, chunk) = tx.next_chunk(start + Duration::from_millis(22)).unwrap();
        assert_eq!(offset, 3 * CHUNK as u64, "fast retransmit resends the hole");
        got.extend(rx.accept(1, offset, &chunk, usize::MAX));
        assert_eq!(got.len(), flight.len() * CHUNK, "held chunks are delivered once the hole fills");
        for (offset, chunk) in fresh {
            got.extend(rx.accept(1, offset, &chunk, usize::MAX));
        }
        let now = start + Duration::from_millis(23);
        tx.on_ack(1, rx.next, now, false);
        let mut now = now;
        while got.len() < data.len() {
            now += Duration::from_millis(5);
            let mut sent = false;
            while let Some((offset, chunk)) = tx.next_chunk(now) {
                sent = true;
                got.extend(rx.accept(1, offset, &chunk, usize::MAX));
            }
            tx.on_ack(1, rx.next, now, false);
            assert!(sent || tx.pending() == 0 || tx.deadline().is_some());
        }
        assert_eq!(got, data);
        assert_eq!(tx.pending(), 0);
        assert!(tx.srtt().is_some());

        tx.push(b"old");
        tx.reset(b"new");
        assert_eq!(tx.gen, 2);
        let (offset, chunk) = tx.next_chunk(now).unwrap();
        assert_eq!((offset, chunk.as_slice()), (0, &b"new"[..]));
        assert_eq!(rx.accept(2, 0, &chunk, 2), b"ne");
        assert_eq!(rx.accept(2, 0, &chunk, 10), b"w");
        assert!(rx.accept(1, 0, b"stale", 10).is_empty());
        assert!(rx.accept(2, 5, b"later", 10).is_empty());
        assert!(rx.accept(2, 4, b"x", 10).is_empty());
        assert_eq!(rx.accept(2, 3, b"ab", 10), b"ablater");
        assert_eq!(rx.next, 10);
    }

    #[test]
    fn timeout_resends_one_chunk_then_what_the_ack_shows_lost() {
        let start = Instant::now();
        let chunk = CHUNK as u64;
        let mut tx = Sender::new(1);
        tx.push(&vec![7u8; 10 * CHUNK]);
        while tx.next_chunk(start).is_some() {}
        // Nothing arrived: after the timeout only the first chunk goes out.
        let fire = tx.deadline().unwrap();
        assert_eq!(tx.next_chunk(fire).map(|(offset, _)| offset), Some(0));
        assert!(tx.next_chunk(fire).is_none(), "one chunk per timeout");
        // Its ack shows the receiver held nothing more: the rest follows.
        let later = fire + Duration::from_millis(400);
        assert!(tx.on_ack(1, chunk, later, false));
        let offsets: Vec<u64> = std::iter::from_fn(|| tx.next_chunk(later)).map(|(offset, _)| offset).collect();
        assert_eq!(offsets.first(), Some(&chunk));
        assert_eq!(offsets.len(), 9);

        // Only chunk 0 was lost: the ack jumps past the probe and the timeout
        // resends nothing more than the holes the acks point at.
        let mut tx = Sender::new(1);
        tx.push(&vec![7u8; 10 * CHUNK]);
        while tx.next_chunk(start).is_some() {}
        let fire = tx.deadline().unwrap();
        assert_eq!(tx.next_chunk(fire).map(|(offset, _)| offset), Some(0));
        let later = fire + Duration::from_millis(400);
        assert!(tx.on_ack(1, 10 * chunk, later, false));
        assert!(tx.next_chunk(later).is_none());
        assert_eq!(tx.pending(), 0);

        // An ack right after the timeout answers the first send: the backoff is undone.
        let mut tx = Sender::new(1);
        tx.push(&vec![7u8; 4 * CHUNK]);
        assert!(tx.next_chunk(start).is_some());
        assert!(tx.on_ack(1, chunk, start + Duration::from_millis(200), false));
        let rto = tx.rto();
        while tx.next_chunk(start).is_some() {}
        let fire = tx.deadline().unwrap();
        assert_eq!(tx.next_chunk(fire).map(|(offset, _)| offset), Some(chunk));
        assert!(tx.rto() > rto);
        assert!(tx.on_ack(1, 4 * chunk, fire + Duration::from_millis(1), false));
        assert_eq!(tx.rto(), rto);
    }

    #[test]
    fn window_grows_without_queueing_and_shrinks_with_it() {
        let mut now = Instant::now();
        let mut tx = Sender::new(1);
        // Always more to send than the window allows; each round trip acks it all.
        let mut round = |tx: &mut Sender, rtt: u64| {
            tx.push(&vec![0u8; 2 * MAX_WINDOW as usize]);
            while tx.next_chunk(now).is_some() {}
            now += Duration::from_millis(rtt);
            tx.on_ack(1, tx.sent, now, false);
        };
        for _ in 0..30 {
            round(&mut tx, 40);
        }
        assert_eq!(tx.window(), MAX_WINDOW, "a link without a queue lets the window grow");
        for _ in 0..30 {
            round(&mut tx, 100);
        }
        assert_eq!(tx.window(), MIN_WINDOW, "a queue shrinks it, down to the floor");
    }

    #[test]
    fn a_reset_does_not_double_what_is_in_flight() {
        let now = Instant::now();
        let mut tx = Sender::new(1);
        tx.push(&vec![0u8; 4 * WINDOW as usize]);
        while tx.next_chunk(now).is_some() {}
        tx.reset(&vec![0u8; 4 * WINDOW as usize]);
        // The old window is still on its way: the new generation waits for
        // its first ack, sending one chunk to get it.
        let sent: usize = std::iter::from_fn(|| tx.next_chunk(now)).map(|(_, chunk)| chunk.len()).sum();
        assert_eq!(sent, CHUNK);
        assert!(tx.on_ack(2, CHUNK as u64, now + Duration::from_millis(50), false));
        let sent: u64 = std::iter::from_fn(|| tx.next_chunk(now)).map(|(_, chunk)| chunk.len() as u64).sum();
        assert!(sent >= WINDOW, "the first ack frees the window");
    }

    #[test]
    fn a_misjudged_timeout_still_repairs_on_every_ack() {
        let start = Instant::now();
        let chunk = CHUNK as u64;
        let mut tx = Sender::new(1);
        // Round trips of 280 ms so far, then the path gets fast and the
        // tail is lost: the resend's ack, 50 ms later, looks too quick to
        // answer it, so the timeout is taken for spurious.
        tx.push(&vec![0u8; CHUNK]);
        tx.next_chunk(start);
        assert!(tx.on_ack(1, chunk, start + Duration::from_millis(280), false));
        let now = start + Duration::from_millis(300);
        tx.push(&vec![0u8; 20 * CHUNK]);
        while tx.next_chunk(now).is_some() {}
        let mut at = tx.deadline().unwrap();
        assert_eq!(tx.next_chunk(at).map(|(offset, _)| offset), Some(chunk));
        // The ack still brings the next hole at once, no timeout between.
        at += Duration::from_millis(50);
        assert!(tx.on_ack(1, 2 * chunk, at, false));
        let next: Vec<u64> = std::iter::from_fn(|| tx.next_chunk(at)).map(|(offset, _)| offset).collect();
        assert_eq!(next, vec![2 * chunk]);
        // Its ack stops right at its end too: the losses run on, so the
        // rest goes at once, not a chunk a round trip.
        at += Duration::from_millis(50);
        assert!(tx.on_ack(1, 3 * chunk, at, false));
        let next: Vec<u64> = std::iter::from_fn(|| tx.next_chunk(at)).map(|(offset, _)| offset).collect();
        assert_eq!(next, (3..21).map(|k| k * chunk).collect::<Vec<_>>());
        at += Duration::from_millis(50);
        assert!(tx.on_ack(1, 21 * chunk, at, false));
        assert_eq!(tx.pending(), 0);
    }

    #[test]
    fn round_trip_spikes_while_typing_keep_the_window() {
        let mut now = Instant::now();
        let mut tx = Sender::new(1);
        // Keystroke-sized sends, every other round trip slow (a busy Wi-Fi).
        for round in 0..40u64 {
            tx.push(b"k");
            while tx.next_chunk(now).is_some() {}
            now += Duration::from_millis(if round % 2 == 0 { 40 } else { 400 });
            tx.on_ack(1, tx.sent, now, false);
        }
        assert_eq!(tx.window(), WINDOW, "little in flight: a slow round trip is not this sender's queue");
    }

    #[test]
    fn a_path_that_gets_slower_is_learned_again() {
        let mut now = Instant::now();
        let mut tx = Sender::new(1);
        let round = |tx: &mut Sender, now: &mut Instant, rtt: u64| {
            tx.push(&vec![0u8; 2 * MAX_WINDOW as usize]);
            while tx.next_chunk(*now).is_some() {}
            *now += Duration::from_millis(rtt);
            tx.on_ack(1, tx.sent, *now, false);
        };
        for _ in 0..20 {
            round(&mut tx, &mut now, 40);
        }
        // Another network: 200 ms for good. At first it reads as a queue.
        for _ in 0..10 {
            round(&mut tx, &mut now, 200);
        }
        assert!(tx.window() < WINDOW);
        for _ in 0..100 {
            round(&mut tx, &mut now, 200);
        }
        assert!(tx.window() > WINDOW, "after a span the new round trip is the base, and the window grows");
    }

    #[test]
    fn a_deflate_stream_inflates_in_any_cuts_and_costs_little_per_update() {
        let mut z = Deflater::default();
        let updates: Vec<Vec<u8>> = (0..200).map(|i| format!("\x1b[{};1H\x1b[32mload {:>5} ok\x1b[0m", i % 40 + 1, i * 7).into_bytes()).collect();
        let packed: Vec<Vec<u8>> = updates.iter().map(|u| z.push(u)).collect();
        // The stream, cut anywhere (as datagrams arrive), gives back each
        // update once its bytes are in.
        let all: Vec<u8> = packed.concat();
        let mut back = Vec::new();
        let mut inflater = Inflater::default();
        for piece in all.chunks(7) {
            back.extend(inflater.push(piece).unwrap());
        }
        assert_eq!(back, updates.concat());
        // Each push ends on a flush: its own bytes show all of it.
        let mut inflater = Inflater::default();
        for (piece, update) in packed.iter().zip(&updates) {
            assert_eq!(&inflater.push(piece).unwrap(), update);
        }
        // History across updates: later ones cost a fraction of their size.
        let late: usize = packed[100..].iter().map(Vec::len).sum();
        let raw: usize = updates[100..].iter().map(Vec::len).sum();
        assert!(late * 2 < raw, "{late} of {raw}");
        assert!(Inflater::default().push(b"\xff\xff\xff\xff").is_none(), "a broken stream is told");
    }

    #[test]
    fn a_slow_link_is_measured_and_held_to_two_round_trips() {
        // 64 kbit/s, 100 ms: each datagram (its bytes and `WIRE_OVERHEAD`)
        // takes its time on the link, arrives 50 ms later, and its ack 50
        // ms after that. A prompt's worth first, as a session starts.
        let rate = 8_000.0;
        let start = Instant::now();
        let mut tx = Sender::new(0);
        tx.push(&[b'$'; 20]);
        let mut queue: VecDeque<(u64, f64)> = VecDeque::new();
        let mut acks: VecDeque<(f64, u64)> = VecDeque::new();
        let mut link_free = 0.0f64;
        let mut most = 0u64;
        let mut t = 0.0f64;
        let (mut highest, mut resent) = (0u64, 0);
        while t < 30.0 {
            let now = start + Duration::from_secs_f64(t);
            if t >= 1.0 && tx.end() == 20 {
                tx.push(&vec![b'x'; 400_000]);
            }
            while let Some((offset, data)) = tx.next_chunk(now) {
                if offset < highest && t > 12.0 {
                    resent += 1;
                }
                highest = highest.max(offset + data.len() as u64);
                let leaves = link_free.max(t) + (data.len() as u64 + WIRE_OVERHEAD) as f64 / rate;
                link_free = leaves;
                queue.push_back((offset + data.len() as u64, leaves + 0.05));
            }
            while queue.front().is_some_and(|(_, at)| *at <= t) {
                let (end, at) = queue.pop_front().unwrap();
                acks.push_back((at + 0.05, end));
            }
            while acks.front().is_some_and(|(at, _)| *at <= t) {
                let (_, end) = acks.pop_front().unwrap();
                tx.on_ack(0, end, now, false);
            }
            // Once the first window (sent before the rate was known, 9 s
            // at this rate) is through.
            if t > 12.0 {
                most = most.max(tx.in_flight());
            }
            t += 0.001;
        }
        let measured = tx.slow_rate().expect("a slow link");
        assert!((rate * 0.7..rate * 1.3).contains(&measured), "{measured}");
        // Two round trips' delivery in flight (the least round trip has a
        // datagram's 152 ms on the wire: 252 ms, 2 KB), not the window.
        assert!(most <= 2 * 2_000 + 100, "{most}");
        // Nothing is lost on this link: no timeout fired too soon.
        assert_eq!(resent, 0);
    }

    #[test]
    fn a_slow_link_gives_an_update_its_time_on_the_wire_before_a_timeout() {
        // 64 kbit/s, 100 ms. Single small updates teach a round trip of
        // about 133 ms and a timeout not far above it. Then an update of 300
        // bytes (52 ms on the wire) is acked 35 ms later than that alone
        // would take, as seen in containers: on a slow link that must not
        // time out, or its resend queues in front of every update after it.
        let rate = 8_000.0;
        let start = Instant::now();
        let mut tx = Sender::new(0);
        let mut t = 0.0f64;
        let wire = |len: usize| (len as u64 + WIRE_OVERHEAD) as f64 / rate;
        for _ in 0..8 {
            tx.push(&[b'u'; 150]);
            let (offset, data) = tx.next_chunk(start + Duration::from_secs_f64(t)).unwrap();
            t += 0.1 + wire(data.len());
            tx.on_ack(0, offset + data.len() as u64, start + Duration::from_secs_f64(t), false);
            t += 0.5;
        }
        // The rate as the server has it by then.
        tx.by_size[1] = Some((120.0, 160));
        tx.by_size[3] = Some((120.0 + 300.0 / rate * 1000.0, 460));
        assert!(tx.slow_rate().is_some());
        tx.push(&[b'v'; 300]);
        let (offset, data) = tx.next_chunk(start + Duration::from_secs_f64(t)).unwrap();
        let acked = t + 0.1 + wire(data.len()) + 0.035;
        let mut resent = 0;
        while t < acked {
            if tx.next_chunk(start + Duration::from_secs_f64(t)).is_some() {
                resent += 1;
            }
            t += 0.001;
        }
        tx.on_ack(0, offset + data.len() as u64, start + Duration::from_secs_f64(t), false);
        assert_eq!(resent, 0);
    }

    #[test]
    fn the_rate_by_size_ignores_close_sizes() {
        // Least times to ack alone by size on a 64 kbit/s, 100 ms path: a
        // 50-byte difference timed 5.4 ms apart says 9.3 KB/s; the larger
        // ones say about 7.8.
        let mut tx = Sender::new(0);
        tx.floor_rtt = Some(120.7);
        for (took, wire) in [(120.7, 121u64), (126.0, 171), (153.7, 377), (233.4, 903)] {
            tx.by_size[(wire as usize / 128).min(9)] = Some((took, wire));
        }
        let rate = tx.size_rate().unwrap();
        assert!((7_000.0..8_000.0).contains(&rate), "{rate}");
        // A late first sample of a smaller size (waited behind an update)
        // is not the base the others are measured from.
        tx.by_size = [None; 10];
        for (took, wire) in [(140.0, 120u64), (120.0, 160), (157.5, 460)] {
            tx.by_size[(wire as usize / 128).min(9)] = Some((took, wire));
        }
        let rate = tx.size_rate().unwrap();
        assert!((7_000.0..9_000.0).contains(&rate), "{rate}");
    }

    #[test]
    fn a_path_that_got_far_is_learned_again() {
        // Started on a LAN (0.3 ms), then the route took 100 ms more: every
        // ack is late against the old floor, so a capacity was read from
        // what this sender happened to send. Three datagrams in a row that
        // take that long on their own show the new delay; fewer are jitter.
        let start = Instant::now();
        let mut tx = Sender::new(0);
        let mut at = start;
        let echo = |tx: &mut Sender, took: f64, at: &mut Instant| {
            tx.push(b"k");
            let (offset, data) = tx.next_chunk(*at).unwrap();
            *at += Duration::from_secs_f64(took / 1000.0);
            tx.on_ack(0, offset + data.len() as u64, *at, false);
            *at += Duration::from_millis(500);
        };
        echo(&mut tx, 0.3, &mut at);
        tx.capacity = Some(66_000.0);
        echo(&mut tx, 100.0, &mut at);
        echo(&mut tx, 101.0, &mut at);
        assert!(tx.capacity.is_some() && tx.floor_rtt < Some(1.0));
        echo(&mut tx, 101.0, &mut at);
        assert_eq!(tx.capacity, None);
        assert_eq!(tx.floor_rtt, Some(100.0));
        // Then RTT 300 ms (1 Mbit/s to a far, fast link): twice the floor
        // at once, the rest climbed to by three late round trips after a
        // rest; one round trip on time ends the climb.
        for took in [300.0, 302.0, 301.0] {
            echo(&mut tx, took, &mut at);
        }
        assert_eq!(tx.floor_rtt, Some(200.0));
        assert_eq!(tx.climb.map(|(shown, _)| shown), Some(300.0));
        for took in [305.0, 305.0, 305.0] {
            echo(&mut tx, took, &mut at);
        }
        assert_eq!(tx.floor_rtt, Some(300.0));
        assert_eq!(tx.climb, None);
        // A queue taken for path: twice the floor at once, the climb to
        // three times at most.
        for took in [1500.0, 1500.0, 1500.0] {
            echo(&mut tx, took, &mut at);
        }
        assert_eq!(tx.floor_rtt, Some(600.0));
        assert_eq!(tx.climb.map(|(shown, _)| shown), Some(900.0));
        for took in [1500.0, 1500.0, 1500.0] {
            echo(&mut tx, took, &mut at);
        }
        assert_eq!(tx.floor_rtt, Some(900.0));
        assert_eq!(tx.climb, None);
        echo(&mut tx, 300.0, &mut at);
        assert_eq!(tx.floor_rtt, Some(300.0), "a round trip on time");
    }

    #[test]
    fn acks_that_waited_on_the_path_say_it_has_no_room() {
        // A LAN that turned slow before its rate is known: acks come well
        // later than lately, and the server stops sending copies (`waited`).
        let mut tx = Sender::new(0);
        let mut at = Instant::now();
        let send_and_ack = |tx: &mut Sender, took: u64, at: &mut Instant| {
            tx.push(&[0u8; 300]);
            let (offset, data) = tx.next_chunk(*at).expect("a chunk");
            *at += Duration::from_millis(took);
            tx.on_ack(0, offset + data.len() as u64, *at, false);
            *at += Duration::from_millis(30);
        };
        for _ in 0..3 {
            send_and_ack(&mut tx, 1, &mut at);
        }
        assert!(!tx.waited(), "a LAN");
        send_and_ack(&mut tx, 200, &mut at);
        send_and_ack(&mut tx, 1, &mut at);
        assert!(!tx.waited(), "one late ack alone");
        // Acks that cover several datagrams (a loss's hole filled) do not
        // count, however late.
        for _ in 0..2 {
            tx.push(&[0u8; 2 * CHUNK]);
            while tx.next_chunk(at).is_some() {}
            at += Duration::from_millis(300);
            tx.on_ack(0, tx.end(), at, false);
        }
        assert!(!tx.waited(), "acks of several datagrams");
        send_and_ack(&mut tx, 200, &mut at);
        send_and_ack(&mut tx, 260, &mut at);
        assert!(tx.waited(), "acks well later than the quickest lately");
        // Once the four are of the new delay, a path that is far but not
        // queued has room again.
        for _ in 0..4 {
            send_and_ack(&mut tx, 260, &mut at);
        }
        assert!(!tx.waited(), "far, not queued");
    }

    #[test]
    fn a_lan_that_turned_far_is_learned_from_late_round_trips() {
        // From a LAN to RTT 300 ms under a steady stream: nothing goes
        // alone, and a capacity read from the ack clock's bursts makes every
        // wait look like a queue. Round trips of 20 ms and more, three in a
        // row, raise the floor; a LAN's jitter (two late ones then a quick
        // one) or one stall that lets many acks through at once does not.
        let start = Instant::now();
        let mut tx = Sender::new(0);
        let mut at = start;
        // Every 100 ms four chunks go and what went `rtt` ago is acked.
        let mut sent: VecDeque<(Instant, u64)> = VecDeque::new();
        let mut step = |tx: &mut Sender, rtt: u64, at: &mut Instant| {
            tx.push(&vec![0u8; 4 * CHUNK]);
            while tx.next_chunk(*at).is_some() {}
            sent.push_back((*at, tx.sent));
            *at += Duration::from_millis(100);
            let rtt = Duration::from_millis(rtt);
            while let Some((when, end)) = sent.front().copied().filter(|(when, _)| *when + rtt <= *at) {
                sent.pop_front();
                tx.on_ack(0, end, when + rtt, false);
            }
        };
        for rtt in [12, 30, 30, 12, 30, 30, 12, 12] {
            step(&mut tx, rtt, &mut at);
        }
        assert!(tx.floor_rtt.is_some_and(|floor| floor < 20.0), "a LAN's jitter, two late round trips at most in a row");
        // One stall (the server's, a radio's) lets six acks through at once,
        // each 40 ms after its datagram: one round trip, not six.
        tx.push(&vec![0u8; 6 * CHUNK]);
        let ends: Vec<u64> = std::iter::from_fn(|| tx.next_chunk(at)).map(|(offset, data)| offset + data.len() as u64).collect();
        assert_eq!(ends.len(), 6);
        at += Duration::from_millis(40);
        for end in ends {
            tx.on_ack(0, end, at, false);
        }
        assert!(tx.floor_rtt.is_some_and(|floor| floor < 20.0), "one stall");
        step(&mut tx, 12, &mut at);
        // The samples as such a stream gave them (14~17 KB/s): each takes
        // in the wait between the ack clock's bursts.
        for _ in 0..15 {
            if tx.floor_rtt.is_some_and(|floor| floor < 20.0) {
                tx.capacity = Some(16_000.0);
                tx.rates = std::iter::repeat_n((at, 16_000.0), 3).collect();
            }
            step(&mut tx, 300, &mut at);
        }
        assert!(tx.floor_rtt.is_some_and(|floor| (290.0..=310.0).contains(&floor)), "{:?}", tx.floor_rtt);
    }

    #[test]
    fn late_acks_apart_do_not_make_a_fast_link_slow() {
        // 10 ms path; single datagrams acked late (a busy client, jitter),
        // minutes apart: no busy stretch, so no rate.
        let start = Instant::now();
        let mut tx = Sender::new(0);
        let mut at = start;
        let send_ack = |tx: &mut Sender, delay: u64, at: &mut Instant| {
            tx.push(b"x".repeat(500).as_slice());
            let (offset, data) = tx.next_chunk(*at).unwrap();
            *at += Duration::from_millis(delay);
            tx.on_ack(0, offset + data.len() as u64, *at, false);
        };
        send_ack(&mut tx, 10, &mut at);
        for _ in 0..5 {
            send_ack(&mut tx, 40, &mut at);
            at += Duration::from_secs(600);
        }
        assert_eq!(tx.slow_rate(), None);
    }

    #[test]
    fn a_lossless_link_never_resends_until_rewound() {
        let start = Instant::now();
        let mut s = Sender::new(0);
        s.set_lossless(true);
        s.push(&[7u8; 3 * CHUNK]);
        let mut sent = 0;
        while s.next_chunk(start).is_some() {
            sent += 1;
        }
        assert_eq!(sent, 3);
        assert_eq!(s.deadline(), None);
        // Long past any timeout, and duplicate acks: nothing goes again.
        let late = start + Duration::from_secs(5);
        for _ in 0..4 {
            s.on_ack(0, 0, late, false);
        }
        assert!(s.next_chunk(late).is_none());
        // A new connection: all of it again, from the first byte unacked.
        s.on_ack(0, CHUNK as u64, late, false);
        s.rewind();
        assert_eq!(s.next_chunk(late).map(|(offset, _)| offset), Some(CHUNK as u64));
    }

    #[test]
    fn a_taken_back_tail_goes_out_again() {
        let now = Instant::now();
        let mut tx = Sender::new(1);
        tx.push(&vec![1u8; 5 * CHUNK]);
        let (offset, chunk) = tx.next_chunk_max(now, SQUEEZED).unwrap();
        assert_eq!((offset, chunk.len()), (0, 5 * CHUNK));
        tx.take_back(0, chunk.len(), CHUNK);
        assert_eq!(tx.next_chunk_max(now, CHUNK).map(|(offset, _)| offset), Some(CHUNK as u64));
        // An ack for what went out is taken; one past it is not.
        assert!(!tx.on_ack(1, 3 * CHUNK as u64, now, false));
        assert!(tx.on_ack(1, 2 * CHUNK as u64, now, false));
        // A resend (`sent` unmoved) gives nothing back.
        let mut tx = Sender::new(1);
        tx.push(&vec![1u8; 3 * CHUNK]);
        while tx.next_chunk(now).is_some() {}
        for _ in 0..3 {
            tx.on_ack(1, 0, now, false);
        }
        let (offset, chunk) = tx.next_chunk_max(now, SQUEEZED).unwrap();
        assert_eq!(offset, 0);
        tx.take_back(0, chunk.len(), 10);
        assert_eq!(tx.sent, 3 * CHUNK as u64);
        assert_eq!(tx.high, 3 * CHUNK as u64);
        // But its ack is read against what went: one that stops right
        // there sends the rest again.
        assert!(tx.on_ack(1, 10, now, false));
        let offsets: Vec<u64> = std::iter::from_fn(|| tx.next_chunk(now)).map(|(offset, _)| offset).collect();
        assert_eq!(offsets, vec![10, 10 + CHUNK as u64, 10 + 2 * CHUNK as u64]);
    }

    #[test]
    fn a_repair_goes_back_only_over_a_run_of_losses_and_times_nothing() {
        let start = Instant::now();
        let chunk = CHUNK as u64;
        // Chunks 3 and 5 lost, apart: the repair of 3 is acked past its end
        // (to 5), so only 5 goes again, not all after it.
        let mut tx = Sender::new(1);
        tx.push(&vec![0u8; 10 * CHUNK]);
        while tx.next_chunk(start).is_some() {}
        let now = start + Duration::from_millis(100);
        assert!(tx.on_ack(1, 3 * chunk, now, false));
        for _ in 0..3 {
            tx.on_ack(1, 3 * chunk, now, false);
        }
        assert_eq!(tx.next_chunk(now).map(|(offset, _)| offset), Some(3 * chunk));
        assert!(tx.next_chunk(now).is_none());
        let later = now + Duration::from_millis(100);
        assert!(tx.on_ack(1, 5 * chunk, later, false));
        let next: Vec<u64> = std::iter::from_fn(|| tx.next_chunk(later)).map(|(offset, _)| offset).collect();
        assert_eq!(next, vec![5 * chunk]);

        // Nothing arrived: the probe's ack stops at its end and all goes
        // again. That ack and those of the resends answer resends: the
        // original send times would read as a path seconds long.
        let mut tx = Sender::new(1);
        tx.push(&vec![0u8; 4 * CHUNK]);
        while tx.next_chunk(start).is_some() {}
        let fire = tx.deadline().unwrap();
        assert_eq!(tx.next_chunk(fire).map(|(offset, _)| offset), Some(0));
        let later = fire + Duration::from_millis(400);
        assert!(tx.on_ack(1, chunk, later, false));
        assert_eq!(std::iter::from_fn(|| tx.next_chunk(later)).count(), 3);
        assert!(tx.on_ack(1, 4 * chunk, later + Duration::from_millis(100), false));
        assert!(tx.by_size.iter().all(Option::is_none));
        assert_eq!(tx.above_floor.0, 0);
    }
}
