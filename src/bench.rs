// `rsh test …`: measurements of the parts a session's speed depends on.
// `render` draws an animation on the terminal it runs in as fast as that
// terminal (or the rsh session it runs in) takes it. `emulator` times this
// build's terminal emulator on typical output: how fast it takes in program
// output (server and client both do this for every byte) and how long a
// full repaint for an attaching client takes. `session` times a real
// session on a host: round trips and bulk output.

use crate::client::{self, Attached, Input, Outcome};
use crate::crypto::Key;
use crate::net::{self, HelloReply, HostMux};
use crate::proto::Action;
use crate::terminal::Terminal;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Output from a program comes in datagrams of about this size.
const CHUNK: usize = 1100;

/// Generated output in the shapes terminals usually see.
fn workloads(cols: u16, rows: u16, bytes: usize) -> Vec<(&'static str, Vec<u8>)> {
    let fill = |make: &dyn Fn(usize) -> String| {
        let mut out = Vec::with_capacity(bytes + 4096);
        let mut i = 0;
        while out.len() < bytes {
            out.extend_from_slice(make(i).as_bytes());
            i += 1;
        }
        out
    };
    let plain = fill(&|i| format!("{i:>8} plain build log line with some words in it, as a compiler prints\r\n"));
    let color = fill(&|i| {
        format!(
            "\x1b[1;32m{i:>6}\x1b[0m \x1b[36mdir-{i}\x1b[0m \x1b[38;5;208mwarn\x1b[0m \x1b[38;2;120;160;255mrgb\x1b[0m \x1b[4mlink\x1b[24m\r\n"
        )
    });
    let wide = fill(&|i| format!("{i:>6} 한글 출력과 日本語 テキスト 그리고 이모지 🚀✨ 섞인 줄\r\n"));
    // A full-screen program redrawing every row (vim, tmux, claude): cursor
    // moves, colors and erases, a row at a time.
    let screen = fill(&|i| {
        let mut frame = String::from("\x1b[?2026h\x1b[H");
        for row in 1..=rows {
            frame.push_str(&format!(
                "\x1b[{row};1H\x1b[48;5;236m\x1b[38;5;{}m{:<width$}\x1b[0m\x1b[K",
                (i + usize::from(row)) % 256,
                format!("row {row} frame {i} │ text ─ ┼"),
                width = usize::from(cols).saturating_sub(1)
            ));
        }
        frame.push_str("\x1b[?2026l");
        frame
    });
    vec![("plain", plain), ("color", color), ("wide", wide), ("fullscreen", screen)]
}

const MIB: f64 = (1 << 20) as f64;

pub fn emulator(size: (u16, u16), mebibytes: usize, file: Option<&std::path::Path>, out: &mut dyn Write) -> io::Result<()> {
    let (cols, rows) = size;
    let inputs = match file {
        Some(path) => vec![(
            "file",
            std::fs::read(path).map_err(|err| io::Error::new(err.kind(), format!("{}: {err}", path.display())))?,
        )],
        None => workloads(cols, rows, mebibytes << 20),
    };
    writeln!(out, "terminal {cols}x{rows}, input fed in {CHUNK}-byte pieces")?;
    writeln!(out, "{:<12} {:>9} {:>11} {:>14} {:>13}", "workload", "input MiB", "feed MiB/s", "repaint ms", "repaint KiB")?;
    for (name, bytes) in inputs {
        let mut term = Terminal::with_size(cols, rows);
        let start = Instant::now();
        for chunk in bytes.chunks(CHUNK) {
            term.feed_to_ground(chunk);
        }
        let feed = start.elapsed();
        // A repaint with the history an attaching client gets.
        let start = Instant::now();
        let repaint = term.render(crate::server::ATTACH_HISTORY);
        let render = start.elapsed();
        writeln!(
            out,
            "{name:<12} {:>9.1} {:>11.1} {:>14.2} {:>13.1}",
            bytes.len() as f64 / MIB,
            bytes.len() as f64 / MIB / feed.as_secs_f64().max(1e-9),
            render.as_secs_f64() * 1e3,
            repaint.len() as f64 / 1024.0
        )?;
    }
    Ok(())
}

/// Starts each frame: an OSC of rsh's own, `ESC ] 5379 ; rsh-frame=<n> BEL`.
/// The client of an rsh session counts and removes them (`FrameCount`).
pub const FRAME_MARK: &str = "\x1b]5379;rsh-frame=";

/// Frames an rsh client wrote to its terminal during `rsh test render <host>`.
#[derive(Debug, Default)]
pub struct FrameCount {
    pub frames: u64,
    /// Number of the last frame seen; frames are numbered from 0.
    pub last: u64,
    /// Bytes written to the terminal from the first frame on.
    pub bytes: u64,
    pub first: Option<Instant>,
    pub end: Option<Instant>,
    /// When each frame's mark arrived, for the gaps between frames.
    pub arrivals: Vec<Instant>,
    /// Stream datagrams with output.
    pub packets: u64,
    /// Datagrams that came ahead of a gap (an earlier one lost or late).
    pub ahead: u64,
    /// Datagrams already had (resent or duplicated).
    pub stale: u64,
    /// New generations: the server sent a fresh screen instead of output
    /// this client had not taken.
    pub repaints: u64,
    /// The longest time with no output datagram, and when the last came.
    pub longest_silence: Duration,
    pub last_packet: Option<Instant>,
}

impl FrameCount {
    /// Counts the frame marks in output about to go to the terminal and
    /// takes them out.
    pub fn take(&mut self, chunk: &[u8], now: Instant) -> Vec<u8> {
        let mark = FRAME_MARK.as_bytes();
        let mut out = Vec::with_capacity(chunk.len());
        let mut rest = chunk;
        while let Some(at) = rest.windows(mark.len()).position(|w| w == mark) {
            out.extend_from_slice(&rest[..at]);
            let after = &rest[at + mark.len()..];
            // A mark is digits and BEL; anything else is left as it came.
            let digits = after.iter().take_while(|b| b.is_ascii_digit()).count();
            if digits == 0 || after.get(digits) != Some(&0x07) {
                out.extend_from_slice(&rest[at..at + mark.len()]);
                rest = after;
                continue;
            }
            if let Ok(number) = std::str::from_utf8(&after[..digits]).unwrap_or("").parse::<u64>() {
                self.frames += 1;
                self.last = self.last.max(number);
                self.first.get_or_insert(now);
                self.end = Some(now);
                self.arrivals.push(now);
            }
            rest = &after[digits + 1..];
        }
        out.extend_from_slice(rest);
        if self.first.is_some() {
            self.bytes += out.len() as u64;
        }
        out
    }

    pub fn summary(&self) -> String {
        let (Some(first), Some(end)) = (self.first, self.end) else {
            return "no frames reached this terminal: the host's rsh may be an older build without frame marks, or the command failed".to_string();
        };
        let secs = end.duration_since(first).as_secs_f64().max(1e-3);
        let mut gaps: Vec<f64> = self.arrivals.windows(2).map(|w| w[1].duration_since(w[0]).as_secs_f64() * 1e3).collect();
        gaps.sort_by(f64::total_cmp);
        let at = |q: f64| gaps.get(((gaps.len() as f64 - 1.0) * q).round() as usize).copied().unwrap_or(0.0);
        let typical = at(0.5);
        let late = gaps.iter().filter(|gap| **gap > typical * 2.0 && **gap > 20.0).count();
        format!(
            "reached this terminal: {} of the first {} frames in {secs:.1} s, {:.1} fps, {:.1} MiB/s\n\
             frame gaps: median {:.0} ms, 95% {:.0} ms, max {:.0} ms; {late} over twice the median\n\
             datagrams: {} with output, {} ahead of a gap, {} already had; {} fresh screens from the server; longest silence {:.0} ms",
            self.frames,
            self.last + 1,
            self.frames.saturating_sub(1) as f64 / secs,
            self.bytes as f64 / MIB / secs,
            typical,
            at(0.95),
            at(1.0),
            self.packets,
            self.ahead,
            self.stale,
            self.repaints,
            self.longest_silence.as_secs_f64() * 1e3
        )
    }

    /// One output datagram, against what the receiver had.
    pub fn packet(&mut self, gen: u32, offset: u64, len: usize, have_gen: u32, have: u64, now: Instant) {
        self.packets += 1;
        if let Some(last) = self.last_packet {
            self.longest_silence = self.longest_silence.max(now.duration_since(last));
        }
        self.last_packet = Some(now);
        if gen != have_gen {
            if gen.wrapping_sub(have_gen) as i32 > 0 {
                self.repaints += 1;
            } else {
                self.stale += 1;
            }
        } else if offset + len as u64 <= have {
            self.stale += 1;
        } else if offset > have {
            self.ahead += 1;
        }
    }
}

/// 5x5 letters of the banner, `#` lit.
const BANNER: [[&str; 5]; 3] = [
    ["####.", "#...#", "####.", "#..#.", "#...#"],
    [".####", "#....", ".###.", "....#", "####."],
    ["#...#", "#...#", "#####", "#...#", "#...#"],
];

/// One frame of the animation for a `cols` x `rows` screen at time `t`:
/// a plasma of ASCII shades in 256 colors with the banner in the middle,
/// every cell drawn, so each frame is about as many bytes as a full-screen
/// program sends.
fn frame(cols: usize, rows: usize, t: f64, number: u64, status: &str, out: &mut Vec<u8>) {
    const SHADES: &[u8] = b" .:-=+*#%@";
    // The frame's number for an rsh client to count; terminals ignore it.
    out.extend_from_slice(format!("{FRAME_MARK}{number}\x07").as_bytes());
    out.extend_from_slice(b"\x1b[?2026h\x1b[H\x1b[0;7m");
    let status: String = status.chars().take(cols).collect();
    out.extend_from_slice(status.as_bytes());
    out.extend(std::iter::repeat_n(b' ', cols.saturating_sub(status.chars().count())));
    out.extend_from_slice(b"\x1b[0m");
    let (banner_w, banner_h) = (3 * 10 + 2 * 2, 5);
    let (left, top) = (cols.saturating_sub(banner_w) / 2, (rows.saturating_sub(banner_h)) / 2);
    let mut last = None;
    for row in 1..rows {
        out.extend_from_slice(format!("\x1b[{};1H", row + 1).as_bytes());
        for col in 0..cols {
            let (x, y) = (col as f64, row as f64 * 2.0);
            let v = ((x / 8.0 + t).sin()
                + (y / 6.0 + t * 1.3).sin()
                + ((x + y) / 12.0 + t * 0.7).sin()
                + ((x * x + y * y).sqrt() / 9.0 - t * 1.7).sin())
                / 8.0
                + 0.5;
            let lit = row >= top && row < top + banner_h && col >= left && col < left + banner_w && {
                let (bx, by) = (col - left, row - top);
                let letter = bx / 12;
                let px = (bx % 12) / 2;
                letter < 3 && px < 5 && BANNER[letter][by].as_bytes()[px] == b'#'
            };
            let (color, ch) = if lit {
                (231, b'@')
            } else {
                // A walk around the 6x6x6 color cube.
                let phase = v * std::f64::consts::TAU;
                let level = |shift: f64| (((phase + shift).sin() * 0.5 + 0.5) * 5.0).round() as u8;
                (16 + 36 * level(0.0) + 6 * level(2.1) + level(4.2), SHADES[((v * 9.99) as usize).min(9)])
            };
            if last != Some(color) {
                out.extend_from_slice(format!("\x1b[38;5;{color}m").as_bytes());
                last = Some(color);
            }
            out.push(ch);
        }
    }
    out.extend_from_slice(b"\x1b[0m\x1b[?2026l");
}

/// `rsh test render`: draws the animation on this terminal as fast as it
/// takes it, for `seconds` or until q, Esc or Ctrl-C, and says how many
/// frames and bytes went through. Inside an rsh session the frames take the
/// whole way: pty, server, network, client, terminal.
pub fn live(seconds: f64, fps: f64, out: &mut dyn Write) -> io::Result<()> {
    let keys = crate::pty::is_tty(0).then(|| crate::pty::RawTerm::enter(0)).transpose()?;
    let through = std::env::var("RSH_SESSION").map(|id| format!("through rsh session {id}")).unwrap_or_else(|_| "to this terminal".to_string());
    out.write_all(b"\x1b[?1049h\x1b[?25l")?;
    let start = Instant::now();
    let (mut frames, mut bytes) = (0u64, 0u64);
    let mut buf = Vec::with_capacity(1 << 20);
    let mut size = crate::pty::window_size(1);
    loop {
        // At most `fps` frames a second (0: as fast as it goes): a frame
        // waits for its turn, so nothing on the way works harder than that.
        if fps > 0.0 {
            let due = start + Duration::from_secs_f64(frames as f64 / fps);
            let now = Instant::now();
            if due > now {
                std::thread::sleep(due - now);
            }
        }
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed >= seconds {
            break;
        }
        if keys.is_some() {
            let mut fd = libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 };
            if unsafe { libc::poll(&mut fd, 1, 0) } > 0 {
                let mut key = [0u8; 64];
                let n = unsafe { libc::read(0, key.as_mut_ptr().cast(), key.len()) };
                if n > 0 && key[..n as usize].iter().any(|b| matches!(b, b'q' | 0x03 | 0x1b)) {
                    break;
                }
            }
        }
        let now = crate::pty::window_size(1);
        if now != size {
            size = now;
            out.write_all(b"\x1b[2J")?;
        }
        let (cols, rows) = (usize::from(size.0.max(1)), usize::from(size.1.max(2)));
        let status = format!(
            " rsh test render  {cols}x{rows}  frame {frames}  {:.0} fps  {:.1} MiB/s  {through}  q stops ",
            frames as f64 / elapsed.max(1e-3),
            bytes as f64 / MIB / elapsed.max(1e-3)
        );
        buf.clear();
        frame(cols, rows, elapsed * 2.0, frames, &status, &mut buf);
        out.write_all(&buf)?;
        out.flush()?;
        frames += 1;
        bytes += buf.len() as u64;
    }
    out.write_all(b"\x1b[0m\x1b[?25h\x1b[?1049l")?;
    out.flush()?;
    drop(keys);
    let elapsed = start.elapsed().as_secs_f64().max(1e-3);
    // Inside an rsh session this counts what was written, not what reached
    // the terminal: `rsh test render <host>` reports that from the client.
    let what = if std::env::var("RSH_SESSION").is_ok() { "written into the session" } else { "drawn on this terminal" };
    writeln!(
        out,
        "{what}: {frames} frames of {}x{} in {elapsed:.1} s, {:.1} fps, {:.1} MiB/s ({:.1} KiB a frame)",
        size.0,
        size.1,
        frames as f64 / elapsed,
        bytes as f64 / MIB / elapsed,
        bytes as f64 / 1024.0 / frames.max(1) as f64
    )?;
    Ok(())
}

/// Bytes written, for output nobody needs to see.
struct Count(usize);

impl Write for Count {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn session(host: &str, peer: SocketAddr, key: &Key, mebibytes: usize, out: &mut dyn Write) -> Result<(), String> {
    let mut rtts = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        net::list_with_server(peer, key, Duration::from_secs(3)).map_err(|err| format!("{host}: {err}"))?;
        rtts.push(start.elapsed());
    }
    rtts.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    writeln!(
        out,
        "{host} ({peer}): request round trip min {:.1} ms, median {:.1} ms, max {:.1} ms",
        ms(rtts[0]),
        ms(rtts[rtts.len() / 2]),
        ms(rtts[rtts.len() - 1])
    )
    .map_err(|err| err.to_string())?;

    // Bulk output through a new session, until its exit status arrives.
    let bytes = mebibytes << 20;
    let command = format!("yes 'rsh benchmark output line, 64 bytes long, with text 0123456789' | head -c {bytes}");
    let mut mux = HostMux::new();
    let sock = mux.socket_for(host, peer).map_err(|err| err.to_string())?;
    let hello = net::new_hello(host, Action::Default, (120, 40), &[command]);
    let start = Instant::now();
    let done = match net::hello(&sock, peer, key, &hello).map_err(|err| format!("{host}: {err}"))? {
        HelloReply::Ready(done) => *done,
        HelloReply::Denied(reason) => return Err(format!("{host}: {}", crate::cli::printable(&reason))),
    };
    let setup = start.elapsed();
    let attached = Attached::new(host, peer, key, done);
    let mut sink = Count(0);
    client::install_signal_handlers();
    let start = Instant::now();
    let outcome = client::run(
        &mut mux,
        &attached,
        Input::Nothing,
        &mut sink,
        client::Options {
            size: (120, 40),
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
    .map_err(|err| err.to_string())?;
    let took = start.elapsed();
    if outcome == Outcome::Detached {
        // Interrupted: the session would otherwise run on to its end.
        let _ = net::kill_sessions(peer, key, host, &attached.session.to_string());
        return Err("interrupted".to_string());
    }
    if outcome != Outcome::Exited(0) {
        return Err(format!("{host}: the benchmark session ended with {outcome:?}"));
    }
    // The server sends a repaint instead of output a slow client has not
    // taken yet, so less may arrive than the program wrote.
    writeln!(
        out,
        "handshake {:.1} ms; the program wrote {mebibytes} MiB and ended after {:.2} s ({:.1} MiB/s); \
         the client received {:.1} MiB ({:.1} MiB/s; the pty adds CR before LF, and repaints replace output a slow client missed)",
        ms(setup),
        took.as_secs_f64(),
        bytes as f64 / MIB / took.as_secs_f64().max(1e-9),
        sink.0 as f64 / MIB,
        sink.0 as f64 / MIB / took.as_secs_f64().max(1e-9)
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_cover_the_screen_with_the_banner() {
        let mut out = Vec::new();
        frame(80, 24, 1.0, 7, "status", &mut out);
        let mut count = FrameCount::default();
        let shown = count.take(&out, Instant::now());
        let mut other = FrameCount::default();
        assert_eq!(other.take(b"\x1b]5379;rsh-frame=12\x18REPAINT\x07AFTER", Instant::now()), b"\x1b]5379;rsh-frame=12\x18REPAINT\x07AFTER");
        assert_eq!(other.frames, 0, "a cut-off mark is not counted and swallows nothing");
        assert_eq!((count.frames, count.last), (1, 7));
        assert!(!shown.windows(4).any(|w| w == b"5379"), "the mark does not reach the terminal");
        let mut term = Terminal::with_size(80, 24);
        term.feed(&out);
        assert!(term.screen_line(0).starts_with("status"));
        assert!((1..24).all(|row| term.screen_line(row).chars().count() == 80 || term.screen_line(row).trim_end().len() < 80));
        let lit = (1..24).map(|row| term.screen_line(row).matches("@@@@@@@@").count()).sum::<usize>();
        assert!(lit >= 3, "the banner shows");
        let mut piped = Vec::new();
        let started = Instant::now();
        live(0.5, 10.0, &mut piped).unwrap();
        let text = String::from_utf8_lossy(&piped);
        let frames = text.matches(FRAME_MARK).count();
        assert!((4..=6).contains(&frames), "10 fps for 0.5 s: {frames} frames");
        assert!(started.elapsed() < Duration::from_secs(1));
        piped.clear();
        live(0.2, 0.0, &mut piped).unwrap();
        let text = String::from_utf8_lossy(&piped);
        assert!(text.contains("frames of") && text.contains("fps"), "{}", text.lines().last().unwrap_or(""));
        assert!(count.summary().contains("1 of the first 8 frames"));
    }

    #[test]
    fn emulator_benchmark_prints_every_workload() {
        let mut out = Vec::new();
        emulator((80, 24), 1, None, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        for name in ["plain", "color", "wide", "fullscreen"] {
            assert!(text.lines().any(|line| line.starts_with(name)), "{text}");
        }
    }
}
