// Local echo prediction in the style of mosh: typed characters are drawn
// before the server echoes them when the round trip is slow.
//
// Guesses are grouped in epochs. Any key whose effect on the screen is not
// a plain character at the cursor (Enter, arrows, Ctrl keys, a paste) starts
// a new epoch, and a guess is only drawn once an earlier guess of its epoch
// was confirmed by the server's echo. So nothing typed after Enter shows
// until the program echoes it, which keeps password prompts dark. A guess
// that is not echoed in time is wrong, and so are guesses whose place the
// program's cursor has left; every guess is then dropped.

use crate::terminal::{sgr_string, Terminal};
use crate::width::char_width;
use std::fmt::Write as _;
use std::time::{Duration, Instant};

/// Round trip above which guesses are drawn, and below which they stop.
const SHOW_RTT: Duration = Duration::from_millis(30);
const HIDE_RTT: Duration = Duration::from_millis(20);
/// Round trip above which drawn guesses are underlined.
const UNDERLINE_RTT: Duration = Duration::from_millis(80);
/// A guess unconfirmed this long is drawn (and underlined) whatever the RTT.
const GLITCH: Duration = Duration::from_millis(250);
/// Input longer than this at once is a paste, not typing.
const PASTE: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Never,
    Adaptive,
    Always,
}

impl Mode {
    pub fn from_env() -> Self {
        match std::env::var("RSH_PREDICTION").as_deref() {
            Ok("never") => Mode::Never,
            Ok("always") => Mode::Always,
            _ => Mode::Adaptive,
        }
    }
}

#[derive(Clone, Debug)]
struct Guess {
    row: usize,
    col: usize,
    ch: char,
    width: usize,
    epoch: u64,
    input_end: u64,
    at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Esc {
    None,
    Start,
    Csi,
    Ss3,
}

pub struct Predictor {
    mode: Mode,
    guesses: Vec<Guess>,
    epoch: u64,
    confirmed: u64,
    blocked_until: Option<u64>,
    drawn: Vec<(usize, usize)>,
    utf8: Vec<u8>,
    esc: Esc,
    active: bool,
    /// A Backspace took back a guess: until the guesses run out, echoes of
    /// the deleted keys leave the cursor past them.
    popped: bool,
}

impl Predictor {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            guesses: Vec::new(),
            epoch: 1,
            confirmed: 0,
            blocked_until: None,
            drawn: Vec::new(),
            utf8: Vec::new(),
            esc: Esc::None,
            active: mode == Mode::Always,
            popped: false,
        }
    }

    pub fn drawn(&self) -> bool {
        !self.drawn.is_empty()
    }

    pub fn pending(&self) -> bool {
        !self.guesses.is_empty()
    }

    /// Records keystrokes; `start` is the input-stream offset of `keys[0]`.
    pub fn typed(&mut self, keys: &[u8], start: u64, mirror: &Terminal, now: Instant) {
        if self.mode == Mode::Never {
            return;
        }
        if keys.len() > PASTE {
            self.new_epoch(start + keys.len() as u64);
            return;
        }
        for (index, &byte) in keys.iter().enumerate() {
            let end = start + index as u64 + 1;
            match self.esc {
                Esc::Start => {
                    self.esc = match byte {
                        b'[' => Esc::Csi,
                        b'O' => Esc::Ss3,
                        _ => Esc::None,
                    };
                    continue;
                }
                Esc::Csi => {
                    if (0x40..=0x7e).contains(&byte) {
                        self.esc = Esc::None;
                    }
                    continue;
                }
                Esc::Ss3 => {
                    self.esc = Esc::None;
                    continue;
                }
                Esc::None => {}
            }
            if !self.utf8.is_empty() || byte >= 0x80 {
                self.utf8.push(byte);
                match std::str::from_utf8(&self.utf8) {
                    Ok(text) => {
                        let ch = text.chars().next().unwrap_or('\u{fffd}');
                        self.utf8.clear();
                        self.character(ch, end, mirror, now);
                    }
                    Err(err) if err.error_len().is_some() || self.utf8.len() >= 4 => {
                        self.utf8.clear();
                        self.new_epoch(end);
                    }
                    Err(_) => {}
                }
                continue;
            }
            match byte {
                0x20..=0x7e => self.character(byte as char, end, mirror, now),
                0x7f | 0x08 => {
                    if self.guesses.pop().is_some() {
                        self.popped = true;
                    } else {
                        self.new_epoch(end);
                    }
                }
                0x1b => {
                    self.esc = Esc::Start;
                    self.new_epoch(end);
                }
                _ => self.new_epoch(end),
            }
        }
    }

    fn new_epoch(&mut self, end: u64) {
        self.epoch += 1;
        self.blocked_until = Some(end);
    }

    fn character(&mut self, ch: char, end: u64, mirror: &Terminal, now: Instant) {
        if self.blocked_until.is_some() {
            self.blocked_until = Some(end);
            return;
        }
        let width = char_width(ch);
        let (cols, _) = mirror.size();
        let (row, col) = match self.guesses.last() {
            Some(last) => (last.row, last.col + last.width),
            None => {
                if mirror.pending_wrap() || !mirror.cursor_visible() {
                    self.new_epoch(end);
                    return;
                }
                mirror.cursor()
            }
        };
        if width == 0 || col + width >= usize::from(cols) || !mirror.blank_from(row, col) {
            self.new_epoch(end);
            return;
        }
        self.guesses.push(Guess {
            row,
            col,
            ch,
            width,
            epoch: self.epoch,
            input_end: end,
            at: now,
        });
    }

    /// Checks guesses against the screen after server output; `acked` is
    /// how much input the server has taken.
    pub fn reconcile(&mut self, mirror: &Terminal, acked: u64, srtt: Option<Duration>, now: Instant) {
        if self.mode == Mode::Never {
            return;
        }
        if let Some(srtt) = srtt {
            if self.mode == Mode::Adaptive {
                if srtt > SHOW_RTT {
                    self.active = true;
                } else if srtt <= HIDE_RTT {
                    self.active = false;
                }
            }
        }
        let patience = srtt.unwrap_or(Duration::ZERO) * 3 + GLITCH;
        let mut wrong = false;
        let mut confirmed = self.confirmed;
        self.guesses.retain(|guess| {
            let echoed = mirror.cell(guess.row, guess.col).is_some_and(|cell| cell.ch == guess.ch);
            if echoed {
                confirmed = confirmed.max(guess.epoch);
                return false;
            }
            if acked >= guess.input_end && now.duration_since(guess.at) > patience {
                wrong = true;
            }
            true
        });
        self.confirmed = confirmed;
        // The next unechoed key lands where the program's cursor is; with
        // the cursor elsewhere (the screen was cleared, the program moved
        // on) the guesses are stale and would park the cursor wrongly.
        if self.guesses.is_empty() {
            self.popped = false;
        } else if !self.popped && self.guesses.first().is_some_and(|guess| mirror.cursor() != (guess.row, guess.col)) {
            wrong = true;
        }
        if wrong {
            self.popped = false;
            self.guesses.clear();
            self.epoch += 1;
        }
        if self.blocked_until.is_some_and(|until| acked >= until) && self.guesses.is_empty() {
            self.blocked_until = None;
        }
    }

    /// Bytes that put back what the program drew under the guesses.
    pub fn undo(&mut self, mirror: &Terminal) -> Vec<u8> {
        if self.drawn.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (row, col) in self.drawn.drain(..) {
            out.extend_from_slice(&mirror.render_cell(row, col));
        }
        out.extend_from_slice(&mirror.cursor_sequence());
        out
    }

    /// Bytes that draw the guesses worth showing and park the cursor after
    /// them. Call after `undo`.
    pub fn draw(&mut self, mirror: &Terminal, srtt: Option<Duration>, now: Instant) -> Vec<u8> {
        let glitch = self
            .guesses
            .first()
            .is_some_and(|guess| now.duration_since(guess.at) >= GLITCH);
        if self.mode == Mode::Never || !(self.active || glitch) {
            return Vec::new();
        }
        let mut pen = mirror.pen();
        if glitch || srtt.is_some_and(|rtt| rtt > UNDERLINE_RTT) {
            pen.flags = (pen.flags & !(0b111 << 8)) | (1 << 8);
        }
        let mut out = String::new();
        let mut cursor = None;
        for guess in &self.guesses {
            if guess.epoch > self.confirmed {
                break;
            }
            let _ = write!(out, "\x1b[{};{}H\x1b(B\x0f\x1b[0m{}{}", guess.row + 1, guess.col + 1, sgr_string(&pen), guess.ch);
            self.drawn.push((guess.row, guess.col));
            cursor = Some((guess.row, guess.col + guess.width));
        }
        let Some((row, col)) = cursor else {
            return Vec::new();
        };
        let _ = write!(out, "\x1b[0m\x1b[{};{}H", row + 1, col + 1);
        out.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_view(local: &mut Terminal, bytes: &[u8]) {
        local.feed(bytes);
    }

    #[test]
    fn guesses_show_after_first_echo_and_hide_after_enter() {
        let start = Instant::now();
        let rtt = Some(Duration::from_millis(100));
        let mut mirror = Terminal::with_size(40, 5);
        let mut local = Terminal::with_size(40, 5);
        let prompt = b"$ ";
        mirror.feed(prompt);
        local.feed(prompt);
        let mut p = Predictor::new(Mode::Adaptive);
        p.typed(b"l", 0, &mirror, start);
        p.reconcile(&mirror, 0, rtt, start);
        assert!(p.draw(&mirror, rtt, start).is_empty(), "first key of an epoch is not drawn");
        mirror.feed(b"l");
        local.feed(b"l");
        p.reconcile(&mirror, 1, rtt, start);
        p.typed(b"s -la", 1, &mirror, start);
        let drawn = p.draw(&mirror, rtt, start);
        feed_view(&mut local, &drawn);
        assert_eq!(local.screen_line(0), "$ ls -la");
        assert_eq!(local.cursor(), (0, 8));
        assert_ne!(local.cell(0, 3).unwrap().attr.underline(), 0);
        let undo = p.undo(&mirror);
        feed_view(&mut local, &undo);
        assert_eq!(local.screen_line(0), "$ l");
        mirror.feed(b"s -la");
        local.feed(b"s -la");
        p.reconcile(&mirror, 6, rtt, start);
        assert!(p.guesses.is_empty());

        p.typed(b"\rsecret", 6, &mirror, start);
        let later = start + Duration::from_secs(5);
        p.reconcile(&mirror, 13, rtt, later);
        let drawn = p.draw(&mirror, rtt, later);
        feed_view(&mut local, &drawn);
        assert!(!local.screen_line(0).contains("secret"));
        assert!(!local.screen_line(1).contains("secret"));
    }

    #[test]
    fn wrong_guesses_are_dropped_and_controls_skip() {
        let start = Instant::now();
        let rtt = Some(Duration::from_millis(100));
        let mut mirror = Terminal::with_size(40, 5);
        mirror.feed(b"$ a");
        let mut p = Predictor::new(Mode::Always);
        p.confirmed = p.epoch;
        p.typed(b"b", 0, &mirror, start);
        assert_eq!(p.guesses.len(), 1);
        p.reconcile(&mirror, 1, rtt, start + Duration::from_secs(2));
        assert!(p.guesses.is_empty(), "never echoed: dropped");
        let mut q = Predictor::new(Mode::Always);
        q.typed(b"\x1b[A", 0, &mirror, start);
        assert!(q.guesses.is_empty());
        assert!(q.blocked_until.is_some());
        q.reconcile(&mirror, 3, rtt, start);
        q.typed(b"x\x7f", 3, &mirror, start);
        assert!(q.guesses.is_empty());
        q.typed("한".as_bytes(), 5, &mirror, start);
        assert_eq!(q.guesses[0].width, 2);
        let mut hidden = Terminal::with_size(40, 5);
        hidden.feed(b"\x1b[?25l");
        let mut r = Predictor::new(Mode::Always);
        r.typed(b"z", 0, &hidden, start);
        assert!(r.guesses.is_empty());
        let mut busy = Terminal::with_size(40, 5);
        busy.feed(b"ab\x1b[1;1H");
        let mut s = Predictor::new(Mode::Always);
        s.typed(b"z", 0, &busy, start);
        assert!(s.guesses.is_empty(), "no guesses in front of existing text");
        // Guesses whose place the program has left (the screen was cleared)
        // are dropped at once, so they are not drawn on the new screen.
        let mut cleared = Terminal::with_size(40, 5);
        cleared.feed(b"$ ");
        let mut t = Predictor::new(Mode::Always);
        t.typed(b"ab", 0, &cleared, start);
        assert_eq!(t.guesses.len(), 2);
        cleared.feed(b"\x1b[H\x1b[2Jnew $ ");
        t.reconcile(&cleared, 0, rtt, start);
        assert!(t.guesses.is_empty());
        // `ab`, Backspace, `c` before any echo: the echo of the deleted `b`
        // leaves the cursor past `c`'s place, and `c` still stands.
        let mut line = Terminal::with_size(40, 5);
        line.feed(b"$ ");
        let mut u = Predictor::new(Mode::Always);
        u.typed(b"ab\x7fc", 0, &line, start);
        assert_eq!(u.guesses.len(), 2);
        line.feed(b"ab");
        u.reconcile(&line, 2, rtt, start);
        assert_eq!(u.guesses.len(), 1, "c is kept");
        line.feed(b"\x08\x1b[Kc");
        u.reconcile(&line, 4, rtt, start);
        assert!(u.guesses.is_empty(), "and confirmed");
        let mut never = Predictor::new(Mode::Never);
        never.typed(b"abc", 0, &mirror, start);
        assert!(never.guesses.is_empty());
    }
}
