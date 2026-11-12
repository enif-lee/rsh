//! Local terminal host for ordinary sessions. The remote grid and its
//! history never contain the header; the physical terminal stays in its
//! alternate screen while this host paints a separate header and body.
use super::{next_unit, Unit, PARTIAL_WAIT};
use crate::terminal::Terminal;
use alacritty_terminal::event::{Event, WindowSize};
use alacritty_terminal::vte::{Params, Parser, Perform};
use std::fmt::Write as _;
use std::time::Instant;

pub(super) struct Viewport {
    terminal: Terminal,
    parser: Parser,
    effects: Effects,
    rows: Vec<Vec<u8>>,
    pub badge: String,
    pub armed: bool,
    pub dirty: bool,
    pub interactive: bool,
    pub partial_expired: bool,
    pub header_rows: u16,
    started: bool,
    partial: Vec<u8>,
    partial_since: Option<Instant>,
    pasting: bool,
    pub replies: Vec<u8>,
    cell_size: (u16, u16),
    cell_size_requested: bool,
    pixel_mouse: bool,
}

impl Viewport {
    pub fn new(size: (u16, u16)) -> Self {
        let mut terminal = Terminal::with_size(size.0, size.1);
        terminal.capture_events();
        Self {
            terminal,
            parser: Parser::new(),
            effects: Effects::default(),
            rows: Vec::new(),
            badge: String::new(),
            armed: false,
            dirty: true,
            interactive: false,
            partial_expired: false,
            header_rows: 1,
            started: false,
            partial: Vec::new(),
            partial_since: None,
            pasting: false,
            replies: Vec::new(),
            cell_size: (0, 0),
            cell_size_requested: false,
            pixel_mouse: false,
        }
    }

    pub fn resize(&mut self, size: (u16, u16)) {
        self.terminal.resize(size.0, size.1);
        self.rows.clear();
        self.dirty = true;
        self.refresh_cell_size();
    }

    pub fn refresh_cell_size(&mut self) {
        if !self.cell_size_requested {
            self.cell_size = (0, 0);
            self.cell_size_requested = true;
            self.effects.bytes.extend_from_slice(b"\x1b[16t");
            self.dirty = true;
        }
    }

    pub fn input_deadline(&self) -> Option<Instant> {
        self.partial_since.map(|since| since + PARTIAL_WAIT)
    }

    pub fn input(&mut self, bytes: &[u8], window: bool, now: Instant) -> Vec<u8> {
        self.partial_expired = false;
        let mut input = std::mem::take(&mut self.partial);
        let mut since = self.partial_since.take();
        input.extend_from_slice(bytes);
        let mut out = Vec::new();
        let mut rest = input.as_slice();
        while !rest.is_empty() {
            let (len, unit) = next_unit(rest);
            if unit == Unit::Partial {
                if since.is_some_and(|at| now.duration_since(at) >= PARTIAL_WAIT) {
                    if !window && rest == b"\x1b" && self.terminal.display_offset() > 0 {
                        self.live();
                    } else {
                        out.extend_from_slice(rest);
                        self.partial_expired = true;
                    }
                } else {
                    self.partial.extend_from_slice(rest);
                    self.partial_since = Some(since.unwrap_or(now));
                }
                break;
            }
            let raw = &rest[..len];
            rest = &rest[len..];
            // A later partial unit must not inherit the completed prefix's
            // deadline, even when both arrive in the same read.
            since = None;
            if self.cell_size_requested {
                if let Some(size) = cell_size_report(raw) {
                    self.cell_size = size;
                    self.cell_size_requested = false;
                    self.dirty = true;
                    continue;
                }
            }
            if raw == b"\x1b[200~" {
                self.pasting = true;
            }
            if self.pasting {
                self.live();
                out.extend_from_slice(raw);
                if raw == b"\x1b[201~" {
                    self.pasting = false;
                }
                continue;
            }
            if let Some((button, x, y, released)) = mouse(raw) {
                let header_height = self.header_rows.saturating_mul(if self.pixel_mouse {
                    self.cell_size.1
                } else {
                    1
                });
                if y <= header_height {
                    continue;
                }
                let y = y - header_height;
                if !window && self.terminal.display_offset() > 0 {
                    if button & 64 != 0 && !released {
                        self.scroll(if button & 1 == 0 { 3 } else { -3 });
                    }
                } else if self.terminal.mouse_mode() {
                    if !self.terminal.pixel_mouse() || self.pixel_mouse {
                        out.extend(self.terminal.mouse_report(button, x, y, released));
                    }
                } else if !window && button & 64 != 0 && !released {
                    if !self.terminal.alt_screen() {
                        self.scroll(if button & 1 == 0 { 3 } else { -3 });
                    } else {
                        // Full-screen programs without mouse reporting retain
                        // the usual alternate-screen wheel-as-arrow behavior.
                        let arrow = if button & 1 == 0 { b'A' } else { b'B' };
                        let prefix = if self.terminal.application_cursor() {
                            b'O'
                        } else {
                            b'['
                        };
                        for _ in 0..3 {
                            out.extend_from_slice(&[0x1b, prefix, arrow]);
                        }
                    }
                }
                continue;
            }
            let page = i32::from(self.terminal.size().1.saturating_sub(1).max(1));
            let scrolled = self.terminal.display_offset() > 0;
            if !window && (raw == b"\x1b[5;2~" || scrolled && raw == b"\x1b[5~") {
                self.scroll(page);
            } else if !window && (raw == b"\x1b[6;2~" || scrolled && raw == b"\x1b[6~") {
                self.scroll(-page);
            } else if !window && scrolled && raw == b"\x1b" {
                self.live();
            } else {
                if !matches!(unit, Unit::Report) {
                    self.live();
                }
                out.extend_from_slice(raw);
            }
        }
        out
    }

    fn scroll(&mut self, lines: i32) {
        self.terminal.scroll_display(lines);
        self.dirty = true;
        self.interactive = true;
    }

    pub fn live(&mut self) {
        if self.terminal.display_offset() > 0 {
            self.terminal.scroll_display(i32::MIN);
            self.dirty = true;
            self.interactive = true;
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.terminal.feed(bytes);
        self.parser.advance(&mut self.effects, bytes);
        let (cols, rows) = self.terminal.size();
        for event in self.terminal.take_events() {
            match event {
                Event::PtyWrite(reply) => self.replies.extend_from_slice(reply.as_bytes()),
                Event::TextAreaSizeRequest(format) => self.replies.extend_from_slice(
                    format(WindowSize {
                        num_lines: rows,
                        num_cols: cols,
                        cell_width: self.cell_size.0,
                        cell_height: self.cell_size.1,
                    })
                    .as_bytes(),
                ),
                _ => {}
            }
        }
        self.dirty |= !bytes.is_empty();
    }

    pub fn draw(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        if !self.started {
            out.extend_from_slice(b"\x1b[?1049h\x1b[2J");
            // Cell dimensions are needed to remove the header from pixel
            // mouse coordinates. This report is consumed locally.
            out.extend_from_slice(b"\x1b[16t");
            self.cell_size_requested = true;
            self.started = true;
        }
        out.append(&mut self.effects.bytes);
        // Neither a remote origin mode nor its scroll region can move the
        // header. Every body row is addressed in physical screen coordinates.
        out.extend_from_slice(b"\x1b[?25l\x1b[r\x1b[?6l\x1b[4l\x1b[?7l\x1b(B\x0f");
        let rows = self.terminal.display_rows();
        for (row, bytes) in rows.iter().enumerate() {
            if self.rows.get(row) != Some(bytes) {
                out.extend_from_slice(
                    format!("\x1b[{};1H", row + 1 + usize::from(self.header_rows)).as_bytes(),
                );
                out.extend_from_slice(bytes);
            }
        }
        self.rows = rows;
        if self.header_rows > 0 {
            out.extend_from_slice(b"\x1b[1;1H\x1b[0m\x1b[2K");
        }
        let offset = self.terminal.display_offset();
        let mut badge = if offset > 0 {
            format!(" [scroll +{offset}]{}", self.badge)
        } else {
            self.badge.clone()
        };
        let cols = usize::from(self.terminal.size().0);
        if unicode_width::UnicodeWidthStr::width(badge.as_str()) > cols {
            badge.clone_from(&self.badge);
        }
        let width = unicode_width::UnicodeWidthStr::width(badge.as_str());
        if self.header_rows > 0 && width > 0 && width <= cols {
            out.extend_from_slice(
                format!(
                    "\x1b[1;{}H\x1b[1;37;48;5;{}m{badge}\x1b[0m",
                    cols - width + 1,
                    if self.armed { 130 } else { 18 }
                )
                .as_bytes(),
            );
        }
        self.pixel_mouse = self.terminal.pixel_mouse() && self.cell_size.1 > 0;
        out.extend(self.terminal.display_modes(self.pixel_mouse));
        if offset == 0 && self.terminal.cursor_visible() {
            let (row, col) = self.terminal.cursor();
            out.extend_from_slice(
                format!(
                    "\x1b[{};{}H\x1b[?25h",
                    row + 1 + usize::from(self.header_rows),
                    col + 1
                )
                .as_bytes(),
            );
        }
        self.dirty = false;
        self.interactive = false;
        out
    }

    pub fn finish(&mut self) -> Vec<u8> {
        if !self.started {
            return Vec::new();
        }
        self.started = false;
        b"\x1b[?2026l\x1b[<99u\x1b[>4;0m\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1005l\x1b[?1006l\x1b[?1015l\x1b[?1016l\x1b[?1004l\x1b[?2004l\x1b[?1l\x1b>\x1b[0 q\x1b[?7h\x1b[?25h\x1b[?1049l".to_vec()
    }
}

fn mouse(raw: &[u8]) -> Option<(u16, u16, u16, bool)> {
    let body = raw.strip_prefix(b"\x1b[<")?;
    let (&last, body) = body.split_last()?;
    if !matches!(last, b'M' | b'm') {
        return None;
    }
    let text = std::str::from_utf8(body).ok()?;
    let mut fields = text.split(';');
    let result = (
        fields.next()?.parse().ok()?,
        fields.next()?.parse().ok()?,
        fields.next()?.parse().ok()?,
        last == b'm',
    );
    fields.next().is_none().then_some(result)
}

fn cell_size_report(raw: &[u8]) -> Option<(u16, u16)> {
    let body = raw.strip_prefix(b"\x1b[6;")?.strip_suffix(b"t")?;
    let text = std::str::from_utf8(body).ok()?;
    let (height, width) = text.split_once(';')?;
    let size = (width.parse().ok()?, height.parse().ok()?);
    (size.0 > 0 && size.1 > 0).then_some(size)
}

/// Non-grid effects still belong to the real terminal: clipboard, palette,
/// shell integration, title and bell. Cursor/status replies come from the
/// virtual terminal so they cannot expose the physical header offset.
#[derive(Default)]
struct Effects {
    bytes: Vec<u8>,
}

impl Perform for Effects {
    fn execute(&mut self, byte: u8) {
        if byte == 7 {
            self.bytes.push(byte);
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell: bool) {
        if params.first() == Some(&b"8".as_slice()) {
            return;
        }
        self.bytes.extend_from_slice(b"\x1b]");
        for (index, param) in params.iter().enumerate() {
            if index > 0 {
                self.bytes.push(b';');
            }
            self.bytes.extend_from_slice(param);
        }
        self.bytes
            .extend_from_slice(if bell { b"\x07" } else { b"\x1b\\" });
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _: bool, action: char) {
        // Preserve window operations other than size queries, answered above.
        if action != 't'
            || !intermediates.is_empty()
            || params
                .iter()
                .next()
                .is_some_and(|p| matches!(p, [14] | [18]))
        {
            return;
        }
        let mut sequence = String::from("\x1b[");
        for (index, param) in params.iter().enumerate() {
            if index > 0 {
                sequence.push(';');
            }
            for (sub, value) in param.iter().enumerate() {
                if sub > 0 {
                    sequence.push(':');
                }
                let _ = write!(sequence, "{value}");
            }
        }
        sequence.push(action);
        self.bytes.extend_from_slice(sequence.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn populated() -> Viewport {
        let mut view = Viewport::new((80, 5));
        view.badge = " shell:host ".into();
        for n in 0..30 {
            view.feed(format!("line {n:02}\r\n").as_bytes());
        }
        view
    }

    #[test]
    fn scrolling_moves_only_the_body_and_output_keeps_the_view_anchored() {
        let mut view = populated();
        let mut tty = Terminal::with_size(80, 6);
        tty.feed(&view.draw());
        assert!(tty.alt_screen());
        assert!(tty.screen_line(0).ends_with("shell:host"));
        assert_eq!(tty.screen_line(1), "line 26");
        let now = Instant::now();
        assert!(view.input(b"\x1b[<64;3;4M", false, now).is_empty());
        tty.feed(&view.draw());
        assert!(tty.screen_line(0).contains("[scroll +3]"));
        assert!(tty.screen_line(0).ends_with("shell:host"));
        let body = (1..6).map(|row| tty.screen_line(row)).collect::<Vec<_>>();
        assert_eq!(body[0], "line 23");
        view.feed(b"new output\r\n");
        tty.feed(&view.draw());
        assert_eq!(
            body,
            (1..6).map(|row| tty.screen_line(row)).collect::<Vec<_>>()
        );
        assert_eq!(view.input(b"x", false, now), b"x");
        tty.feed(&view.draw());
        assert_eq!(view.terminal.display_offset(), 0);
        assert!(tty.screen_line(4).contains("new output"));
        assert_eq!(tty.scrollback_len(), 0);
        assert!((0..view.terminal.scrollback_len())
            .all(|n| !view.terminal.scrollback_line(n).contains("shell:host")));
    }

    #[test]
    fn fragmented_wheel_pages_escape_and_paste_are_local_only_where_expected() {
        let mut view = populated();
        let now = Instant::now();
        assert!(view.input(b"\x1b[<64;", false, now).is_empty());
        assert!(view.input(b"3;4M", false, now).is_empty());
        assert_eq!(view.terminal.display_offset(), 3);
        assert!(view.input(b"\x1b[5;2~", false, now).is_empty());
        assert_eq!(view.terminal.display_offset(), 7);
        assert!(view.input(b"\x1b[6~", false, now).is_empty());
        assert_eq!(view.terminal.display_offset(), 3);
        assert!(view.input(b"\x1b", false, now).is_empty());
        assert!(view.input(b"", false, now + PARTIAL_WAIT).is_empty());
        assert_eq!(view.terminal.display_offset(), 0);
        let paste = b"\x1b[200~literal\x1b[5;2~\x1b[<64;3;4M\x1b[201~";
        assert_eq!(view.input(paste, false, now), paste);
        assert_eq!(view.terminal.display_offset(), 0);
        assert_eq!(
            view.input(b"\x1b[5;2~", true, now),
            b"\x1b[5;2~",
            "modal navigation goes to the modal"
        );
        assert!(
            view.input(b"\x1b[<64;3;4M\x1b[<0;3;4M", true, now)
                .is_empty(),
            "local mouse capture must not inject mouse reports into a shell behind a modal"
        );
        assert!(view.input(b"\x1b[<64;", false, now).is_empty());
        assert!(view
            .input(b"3;4M\x1b", false, now + PARTIAL_WAIT)
            .is_empty());
        assert_eq!(
            view.input(
                b"[A",
                false,
                now + PARTIAL_WAIT + std::time::Duration::from_millis(5)
            ),
            b"\x1b[A"
        );
    }

    #[test]
    fn mouse_and_cursor_reports_use_remote_body_coordinates() {
        let mut view = Viewport::new((80, 5));
        view.feed(b"\x1b[?1002h\x1b[?1006h\x1b[3;4H\x1b[6n\x1b[18t");
        assert!(view.replies.windows(6).any(|bytes| bytes == b"\x1b[3;4R"));
        assert!(view
            .replies
            .windows(9)
            .any(|bytes| bytes == b"\x1b[8;5;80t"));
        let now = Instant::now();
        assert_eq!(view.input(b"\x1b[<64;10;3M", false, now), b"\x1b[<64;10;2M");
        assert_eq!(view.input(b"\x1b[<0;10;3m", false, now), b"\x1b[<0;10;2m");
        assert!(view.input(b"\x1b[<0;10;1M", false, now).is_empty());
        let mut tty = Terminal::with_size(80, 6);
        tty.feed(&view.draw());
        assert_eq!(tty.cursor(), (3, 3));
        assert!(String::from_utf8_lossy(&tty.render(0)).contains("?1002h"));
        view.feed(b"\x1b[?1049h\x1b[H\x1b[?1006l\x1b[?1000l\x1b[?1002lFULL SCREEN");
        tty.feed(&view.draw());
        assert!(tty.screen_line(1).contains("FULL SCREEN"));
        assert_eq!(
            view.input(b"\x1b[<64;10;3M", false, now),
            b"\x1b[A\x1b[A\x1b[A"
        );
        view.feed(b"\x1b[?1h");
        assert_eq!(
            view.input(b"\x1b[<64;10;3M", false, now),
            b"\x1bOA\x1bOA\x1bOA"
        );
        view.feed(b"\x1b[?1049l");
        tty.feed(&view.draw());
        assert_eq!(tty.cursor(), (3, 3));
    }

    #[test]
    fn resize_and_remote_reset_cannot_put_the_header_into_history() {
        let mut view = populated();
        let mut tty = Terminal::with_size(80, 6);
        tty.feed(&view.draw());
        tty.resize(40, 3);
        view.resize((40, 2));
        tty.feed(&view.draw());
        assert!(tty.screen_line(0).ends_with("shell:host"));
        view.feed(b"\x1bc\x1b[2Jreset\r\n");
        tty.feed(&view.draw());
        assert!(tty.alt_screen());
        assert!(tty.screen_line(0).ends_with("shell:host"));
        assert!(tty.screen_line(1).contains("reset"));
        tty.resize(40, 1);
        view.header_rows = 0;
        view.resize((40, 1));
        tty.feed(&view.draw());
        assert!(!tty.screen_line(0).contains("shell:host"));
        tty.feed(&view.finish());
        assert!(!tty.alt_screen());
        assert_eq!(tty.scrollback_len(), 0);
    }

    #[test]
    fn clipboard_title_colors_and_bell_still_reach_the_terminal() {
        let mut view = Viewport::new((80, 5));
        let controls = b"\x1b]52;c;aGVsbG8=\x07\x1b]10;?\x1b\\\x1b]2;remote title\x07\x07";
        // The remote stream may split anywhere, including inside an OSC.
        for byte in controls {
            view.feed(&[*byte]);
        }
        let output = view.draw();
        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("\x1b]52;c;aGVsbG8=\x07"));
        assert!(output.contains("\x1b]10;?\x1b\\"));
        assert!(output.contains("\x1b]2;remote title\x07\x07"));
    }

    #[test]
    fn pixel_mouse_remeasures_the_header_after_a_font_resize() {
        let mut view = Viewport::new((80, 5));
        view.feed(b"\x1b[?1000h\x1b[?1016h");
        let now = Instant::now();
        view.draw();
        assert!(view.input(b"\x1b[6;20;10t", false, now).is_empty());
        view.draw();
        assert_eq!(
            view.input(b"\x1b[<0;100;61M", false, now),
            b"\x1b[<0;100;41M"
        );
        view.resize((70, 4));
        assert!(String::from_utf8_lossy(&view.draw()).contains("\x1b[16t"));
        assert!(view.input(b"\x1b[<0;100;61M", false, now).is_empty());
        assert!(view.input(b"\x1b[6;30;10t", false, now).is_empty());
        view.draw();
        assert_eq!(
            view.input(b"\x1b[<0;100;61M", false, now),
            b"\x1b[<0;100;31M"
        );
        assert!(view.input(b"\x1b[<0;100;29M", false, now).is_empty());
        view.feed(b"\x1b[14t");
        assert!(String::from_utf8_lossy(&view.replies).contains("\x1b[4;120;700t"));
        assert!(String::from_utf8_lossy(&view.finish()).contains("\x1b[?1016l"));
    }
}
