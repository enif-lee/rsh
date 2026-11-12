//! Local terminal host for ordinary sessions. The remote grid and its
//! history never contain the header; the physical terminal stays in its
//! alternate screen while the header overlays the first body row.
use super::{next_unit, Unit, PARTIAL_WAIT};
use crate::proto::{ScrollRequest, ScrollStatus};
use crate::terminal::{scroll_change, Effects, Terminal};
use alacritty_terminal::event::{Event, WindowSize};
use alacritty_terminal::vte::Parser;
#[cfg(test)]
use std::fmt::Write as _;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowUpdate {
    Unchanged,
    Moved,
    Redrawn,
}

impl RowUpdate {
    fn background(self) -> u8 {
        match self {
            Self::Unchanged => 236,
            Self::Moved => 22,
            Self::Redrawn => 52,
        }
    }
}

pub(super) struct Viewport {
    terminal: Terminal,
    pub request: ScrollRequest,
    offset: usize,
    shown: ScrollStatus,
    scrolling: bool,
    parser: Parser,
    effects: Effects,
    rows: Vec<Vec<u8>>,
    profiling: bool,
    profile_painted: Vec<Option<RowUpdate>>,
    pub badge: String,
    pub armed: bool,
    pub dirty: bool,
    pub interactive: bool,
    pub scroll_until: Option<Instant>,
    pub partial_expired: bool,
    badge_drawn: bool,
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
        let mut terminal = Terminal::with_history(size.0, size.1, 0);
        terminal.capture_events();
        Self {
            terminal,
            request: ScrollRequest::default(),
            offset: 0,
            shown: ScrollStatus::default(),
            scrolling: false,
            parser: Parser::new(),
            effects: Effects::default(),
            rows: Vec::new(),
            profiling: false,
            profile_painted: Vec::new(),
            badge: String::new(),
            armed: false,
            dirty: true,
            interactive: false,
            scroll_until: None,
            partial_expired: false,
            badge_drawn: false,
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

    pub fn set_profiling(&mut self, enabled: bool) {
        if self.profiling == enabled {
            return;
        }
        self.profiling = enabled;
        // Diagnostic backgrounds never enter the model or its row cache.
        // Disabling them restores every original cell, including app colors.
        if !enabled {
            self.invalidate();
        } else {
            self.dirty = true;
        }
        self.interactive = true;
    }

    pub fn profiling(&self) -> bool {
        self.profiling
    }

    pub fn resize(&mut self, size: (u16, u16)) {
        self.terminal.resize(size.0, size.1);
        self.invalidate();
        self.refresh_cell_size();
    }

    pub fn invalidate(&mut self) {
        self.rows.clear();
        self.profile_painted.clear();
        self.dirty = true;
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
        if !self.pasting && matches!(mouse_input(&self.partial), Some(MouseInput::Incomplete)) {
            return None;
        }
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
            let mouse_unit = if self.pasting {
                None
            } else {
                mouse_input(rest)
            };
            let (len, unit) = match mouse_unit {
                Some(MouseInput::Incomplete) => {
                    if rest.len() <= 64 {
                        self.partial.extend_from_slice(rest);
                        self.partial_since = Some(since.unwrap_or(now));
                    }
                    break;
                }
                Some(MouseInput::Invalid(at)) => {
                    // Discard only the malformed mouse prefix, preserving
                    // the key (including Enter/Esc) that interrupted it.
                    rest = &rest[at..];
                    since = None;
                    continue;
                }
                Some(MouseInput::Ready(len)) => (len, Unit::Report),
                None => next_unit(rest),
            };
            if unit == Unit::Partial {
                if since.is_some_and(|at| now.duration_since(at) >= PARTIAL_WAIT) {
                    if !window && rest == b"\x1b" && self.scrolling {
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
                let pixel_mouse = self.pixel_mouse && raw.starts_with(b"\x1b[<");
                if !window && button & 64 != 0 && !released {
                    self.scroll_until = Some(now + std::time::Duration::from_millis(100));
                }
                if !window && self.scrolling {
                    if button & 64 != 0 && !released {
                        self.scroll(if button & 1 == 0 { 3 } else { -3 });
                    }
                } else if self.terminal.mouse_mode() {
                    if !self.terminal.pixel_mouse() || pixel_mouse {
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
            if matches!(mouse_unit, Some(MouseInput::Ready(_))) {
                continue;
            }
            let page = i32::from(self.terminal.size().1.saturating_sub(1).max(1));
            let scrolled = self.scrolling;
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
        self.request.serial += 1;
        self.request.position = self.request.position.saturating_add(i64::from(lines));
        self.scrolling |= lines > 0;
    }

    pub fn live(&mut self) {
        if self.scrolling {
            self.request.serial += 1;
            self.request.reset = self.request.serial;
            self.request.position = 0;
            self.scrolling = false;
        }
    }

    pub fn status(&mut self, status: ScrollStatus) {
        if status.serial == self.request.serial && status != self.shown {
            self.shown = status;
            self.offset = status.offset as usize;
            self.scrolling = self.offset > 0;
            self.dirty = true;
            self.interactive = true;
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        let alternate = self.terminal.alt_screen();
        self.terminal.feed(bytes);
        if alternate != self.terminal.alt_screen() {
            self.invalidate();
        }
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
            // Cell dimensions answer pixel size queries from remote apps.
            // This report is consumed locally.
            out.extend_from_slice(b"\x1b[16t");
            self.cell_size_requested = true;
            self.started = true;
        }
        out.append(&mut self.effects.bytes);
        // Body coordinates fill the physical screen. Restore the old
        // overlay before any row movement so it never scrolls into the body.
        out.extend_from_slice(b"\x1b[?25l\x1b[r\x1b[?6l\x1b[4l\x1b[?7l\x1b(B\x0f");
        if self.badge_drawn {
            if let Some(row) = self.rows.first() {
                out.extend_from_slice(b"\x1b[1;1H");
                out.extend_from_slice(row);
                if let Some(painted) = self.profile_painted.first_mut() {
                    *painted = None;
                }
            }
            self.badge_drawn = false;
        }
        let rows = self.terminal.display_rows();
        let mut updates = self
            .profiling
            .then(|| vec![RowUpdate::Unchanged; rows.len()]);
        if self.profiling {
            self.profile_painted.resize(rows.len(), None);
        }
        let moved = self.shift_rows(&rows).map(|(control, moved)| {
            out.extend(control);
            moved
        });
        for (row, bytes) in rows.iter().enumerate() {
            let repaint = self.rows.get(row) != Some(bytes);
            if let Some(updates) = &mut updates {
                updates[row] = if repaint {
                    RowUpdate::Redrawn
                } else if moved.as_ref().is_some_and(|range| range.contains(&row)) {
                    RowUpdate::Moved
                } else {
                    RowUpdate::Unchanged
                };
            }
            if repaint {
                out.extend_from_slice(format!("\x1b[{};1H", row + 1).as_bytes());
                if self.profiling {
                    out.extend(
                        self.terminal
                            .render_profiled_row(row, RowUpdate::Redrawn.background()),
                    );
                    self.profile_painted[row] = Some(RowUpdate::Redrawn);
                } else {
                    out.extend_from_slice(bytes);
                }
            }
        }
        self.rows = rows;
        if let Some(updates) = updates {
            for (row, update) in updates.into_iter().enumerate() {
                if self.profile_painted[row] != Some(update) {
                    out.extend_from_slice(format!("\x1b[{};1H", row + 1).as_bytes());
                    out.extend(self.terminal.render_profiled_row(row, update.background()));
                    self.profile_painted[row] = Some(update);
                }
            }
        }
        let offset = self.offset;
        let mut badge = if offset > 0 {
            format!(" [scroll +{offset}]{}", self.badge)
        } else {
            self.badge.clone()
        };
        if self.profiling {
            badge.insert_str(0, " [profile] ");
        }
        let cols = usize::from(self.terminal.size().0);
        if unicode_width::UnicodeWidthStr::width(badge.as_str()) > cols {
            badge.clone_from(&self.badge);
        }
        let width = unicode_width::UnicodeWidthStr::width(badge.as_str());
        let mut from = cols.saturating_sub(width);
        let (cursor_row, cursor_col) = self.terminal.cursor();
        let in_the_way = offset == 0
            && self.terminal.cursor_visible()
            && cursor_row == 0
            && cursor_col + 2 >= from;
        if width > 0 && cols >= width + 20 && !in_the_way {
            if from > 0
                && self
                    .terminal
                    .cell(0, from)
                    .is_some_and(|cell| cell.is_spacer())
            {
                from -= 1;
                badge.insert(0, ' ');
            }
            out.extend_from_slice(
                format!(
                    "\x1b[1;{}H\x1b[1;37;48;5;{}m{badge}\x1b[0m",
                    from + 1,
                    if self.armed { 130 } else { 18 }
                )
                .as_bytes(),
            );
            self.badge_drawn = true;
        }
        self.pixel_mouse = self.terminal.pixel_mouse() && self.cell_size.1 > 0;
        out.extend(self.terminal.display_modes(self.pixel_mouse));
        if offset == 0 && self.terminal.cursor_visible() {
            let (row, col) = self.terminal.cursor();
            out.extend_from_slice(format!("\x1b[{};{}H\x1b[?25h", row + 1, col + 1).as_bytes());
        }
        self.dirty = false;
        self.interactive = false;
        out
    }

    /// Infer movement from frame contents, including an application's
    /// scrolling transcript between stationary header/prompt/footer rows.
    /// No wheel event or history/display_offset hint is required.
    fn shift_rows(&mut self, next: &[Vec<u8>]) -> Option<(Vec<u8>, std::ops::Range<usize>)> {
        use std::hash::Hasher;
        if next.len() != self.rows.len()
            || next
                .iter()
                .zip(&self.rows)
                .filter(|(a, b)| a != b)
                .take(2)
                .count()
                < 2
        {
            return None;
        }
        let hash = |row: &Vec<u8>| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            h.write(row);
            h.finish()
        };
        let current: Vec<_> = next.iter().map(hash).collect();
        let previous: Vec<_> = self.rows.iter().map(hash).collect();
        let Some((k, a, b)) = scroll_change(&current, &previous) else {
            return None;
        };
        let (top, bottom) = if k > 0 {
            (a, b - 1 + k as usize)
        } else {
            ((a as isize + k) as usize, b - 1)
        };
        let n = k.unsigned_abs();
        let physical = |row: usize| row + 1;
        let control = format!(
            "\x1b[0m\x1b[{};{}r\x1b[{}{}\x1b[r",
            physical(top),
            physical(bottom),
            n,
            if k > 0 { 'S' } else { 'T' }
        );
        let mut before = 0;
        let mut after = control.len();
        for row in top..=bottom {
            let cost = next[row].len() + format!("\x1b[{};1H", physical(row)).len();
            if next[row] != self.rows[row] {
                before += cost;
            }
            let from = row as isize + k;
            if from < top as isize
                || from > bottom as isize
                || next[row] != self.rows[from as usize]
            {
                after += cost;
            }
        }
        // Exact byte comparisons cover collisions and extra changed rows
        // at either end. Do not scroll when it would increase the payload.
        if after >= before {
            return None;
        }
        let region = &mut self.rows[top..=bottom];
        if k > 0 {
            region.rotate_left(n);
            let exposed = region.len() - n;
            for row in &mut region[exposed..] {
                row.clear();
            }
        } else {
            region.rotate_right(n);
            for row in &mut region[..n] {
                row.clear();
            }
        }
        if self.profiling {
            let painted = &mut self.profile_painted[top..=bottom];
            if k > 0 {
                painted.rotate_left(n);
                let exposed = painted.len() - n;
                painted[exposed..].fill(None);
            } else {
                painted.rotate_right(n);
                painted[..n].fill(None);
            }
        }
        Some((control.into_bytes(), a..b))
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
    if let Some(body) = raw.strip_prefix(b"\x1b[M") {
        let [button, x, y] = body else {
            return None;
        };
        let button = u16::from(button.checked_sub(32)?);
        return Some((
            button,
            u16::from(x.checked_sub(32)?),
            u16::from(y.checked_sub(32)?),
            button & 3 == 3 && button & 64 == 0,
        ));
    }
    let (body, sgr) = match raw.strip_prefix(b"\x1b[<") {
        Some(body) => (body, true),
        None => (raw.strip_prefix(b"\x1b[")?, false),
    };
    let (&last, body) = body.split_last()?;
    if !matches!(last, b'M' | b'm') {
        return None;
    }
    let text = std::str::from_utf8(body).ok()?;
    let mut fields = text.split(';');
    let button: u16 = fields.next()?.parse().ok()?;
    let button = if sgr { button } else { button.checked_sub(32)? };
    let result = (
        button,
        fields.next()?.parse().ok()?,
        fields.next()?.parse().ok()?,
        last == b'm' || !sgr && button & 3 == 3 && button & 64 == 0,
    );
    fields.next().is_none().then_some(result)
}

#[derive(Clone, Copy)]
enum MouseInput {
    Ready(usize),
    Incomplete,
    Invalid(usize),
}

/// Recognize reports before the generic key parser so malformed or delayed
/// mouse bytes cannot be reinterpreted as shell input.
fn mouse_input(raw: &[u8]) -> Option<MouseInput> {
    if raw.starts_with(b"\x1b[M") {
        for at in 3..6 {
            match raw.get(at) {
                None => return Some(MouseInput::Incomplete),
                Some(byte) if *byte < 32 => return Some(MouseInput::Invalid(at)),
                _ => {}
            }
        }
        return Some(MouseInput::Ready(6));
    }
    let sgr = raw.starts_with(b"\x1b[<");
    let start = if sgr {
        3
    } else {
        let body = raw.strip_prefix(b"\x1b[")?;
        let end = body.iter().position(|byte| *byte == b';')?;
        let button = std::str::from_utf8(&body[..end])
            .ok()?
            .parse::<u16>()
            .ok()?;
        if !(32..=255).contains(&button) {
            return None;
        }
        2
    };
    let mut field = 0;
    let mut digits = 0;
    for (at, &byte) in raw.iter().enumerate().skip(start) {
        match byte {
            b'0'..=b'9' => digits += 1,
            b';' if field < 2 && digits > 0 => {
                field += 1;
                digits = 0;
            }
            b'M' | b'm' if field == 2 && digits > 0 && (sgr || byte == b'M') => {
                return Some(MouseInput::Ready(at + 1))
            }
            // Numeric CSI prefixes overlap kitty/function keys and reports;
            // once identified, let the ordinary parser preserve those units.
            b'u' if !sgr && digits > 0 => return None,
            b'~' | b'R' if !sgr && field == 1 && digits > 0 => return None,
            b':' if !sgr && digits > 0 => return None,
            _ => return Some(MouseInput::Invalid(at)),
        }
    }
    Some(MouseInput::Incomplete)
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
#[cfg(test)]
mod tests {
    use super::*;
    use alacritty_terminal::vte::{Params, Perform};

    fn tui_frame(first: usize, rows: u16) -> Vec<u8> {
        let mut out = String::from("\x1b[?2026h\x1b[H\x1b[2J");
        for row in 0..rows {
            let _ = write!(out, "\x1b[{};1H\x1b[0m", row + 1);
            match row {
                0 => out.push_str("Coding assistant — transcript"),
                1 => out.push_str("──────────────────────────────"),
                row if row == rows - 2 => out.push_str("> pending input stays fixed"),
                row if row == rows - 1 => out.push_str("status: ready"),
                row => {
                    let id = first + usize::from(row - 2);
                    let _ = write!(
                        out,
                        "\x1b[38;5;{}mline {id:04}: 한글 e\u{301} ",
                        30 + id % 12
                    );
                    let _ = write!(
                        out,
                        "\x1b]8;id=line-{id};https://example.invalid/{id}\x1b\\link\x1b]8;;\x1b\\"
                    );
                }
            }
            out.push_str("\x1b[0m\x1b[K");
        }
        let _ = write!(out, "\x1b[{};3H\x1b[?2026l", rows - 1);
        out.into_bytes()
    }

    #[test]
    fn scrolling_past_repeated_blank_rows_reuses_the_whole_block_in_both_directions() {
        use crate::terminal::Color;
        let mut view = Viewport::new((100, 20));
        let mut reference = Viewport::new((100, 20));
        view.set_profiling(true);
        reference.set_profiling(true);
        let mut tty = Terminal::with_size(100, 20);
        let mut full = Terminal::with_size(100, 20);
        for (step, first) in [100, 102, 100].into_iter().enumerate() {
            let mut frame = String::from("\x1b[1;1Hfixed title\x1b[K");
            for row in 1..18 {
                let id = first + row - 1;
                let _ = write!(frame, "\x1b[{};1H\x1b[2K", row + 1);
                if id % 2 != 0 {
                    let _ = write!(frame, "message {id}: paragraph 한글");
                }
            }
            frame.push_str("\x1b[19;1Hfixed prompt\x1b[K\x1b[20;1Hfixed footer\x1b[K\x1b[19;3H");
            view.feed(frame.as_bytes());
            reference.feed(frame.as_bytes());
            let output = view.draw();
            tty.feed(&output);
            reference.invalidate();
            full.feed(&reference.draw());
            if step > 0 {
                let moved = if step == 1 { 1..16 } else { 3..18 };
                assert!(String::from_utf8_lossy(&output).contains(if step == 1 {
                    "\x1b[2S"
                } else {
                    "\x1b[2T"
                }));
                for row in moved {
                    assert_eq!(
                        tty.cell(row, 0).unwrap().attr.bg,
                        Color::Indexed(22),
                        "frame {step}, row {row} must reuse even a repeated blank row"
                    );
                }
                for row in if step == 1 { 16..18 } else { 1..3 } {
                    assert_eq!(tty.cell(row, 0).unwrap().attr.bg, Color::Indexed(52));
                }
            }
            for row in 0..20 {
                for col in 0..100 {
                    assert_eq!(
                        tty.cell(row, col).unwrap().ch,
                        full.cell(row, col).unwrap().ch
                    );
                }
            }
        }
    }

    #[test]
    fn profiling_marks_reused_redrawn_and_unchanged_rows_without_changing_the_model() {
        use crate::terminal::Color;
        let mut view = Viewport::new((100, 12));
        let mut reference = Viewport::new((100, 12));
        view.badge = " session:host ".into();
        reference.badge.clone_from(&view.badge);
        view.set_profiling(true);
        let mut tty = Terminal::with_size(100, 12);
        let mut normal = Terminal::with_size(100, 12);
        for (step, first) in [100, 101, 100].into_iter().enumerate() {
            let frame = tui_frame(first, 12);
            view.feed(&frame);
            reference.feed(&frame);
            tty.feed(&view.draw());
            normal.feed(&reference.draw());
            assert_eq!(
                view.rows, reference.rows,
                "diagnostic colors must not enter diff cache"
            );
            let colors = if step == 0 {
                [52, 52, 52, 52]
            } else if step == 1 {
                [236, 22, 52, 236]
            } else {
                [236, 52, 22, 236]
            };
            for (row, background) in [0, 2, 9, 10].into_iter().zip(colors) {
                assert_eq!(
                    tty.cell(row, 0).unwrap().attr.bg,
                    Color::Indexed(background)
                );
            }
            for row in 0..12 {
                for col in 0..70 {
                    assert_eq!(
                        tty.cell(row, col).unwrap().ch,
                        normal.cell(row, col).unwrap().ch
                    );
                    assert_eq!(
                        tty.cell(row, col).unwrap().attr.fg,
                        normal.cell(row, col).unwrap().attr.fg
                    );
                }
            }
        }
        tty.feed(&view.draw());
        assert_eq!(
            tty.cell(2, 0).unwrap().attr.bg,
            Color::Indexed(236),
            "an idle frame reuses its rows in place"
        );
        let idle = view.draw();
        assert!(
            !String::from_utf8_lossy(&idle).contains("line "),
            "stable diagnostic colors must not repaint unchanged body rows"
        );
        view.resize((80, 10));
        reference.resize((80, 10));
        tty.resize(80, 10);
        normal.resize(80, 10);
        view.feed(b"\x1b[3;1H\x1b[7;44mreverse text\x1b[0m");
        reference.feed(b"\x1b[3;1H\x1b[7;44mreverse text\x1b[0m");
        tty.feed(&view.draw());
        normal.feed(&reference.draw());
        assert_eq!(
            tty.cell(2, 0).unwrap().attr.bg,
            Color::Indexed(52),
            "inverse video must not hide profiling backgrounds"
        );
        view.set_profiling(false);
        tty.feed(&view.draw());
        assert!(
            tty.same_screen(&normal),
            "turning profiling off restores original colors, inverse video, cursor and header"
        );
    }

    #[test]
    fn overlay_restores_wide_colored_cells_before_both_scroll_directions_and_hiding() {
        let mut view = Viewport::new((80, 8));
        let mut reference = Viewport::new((80, 8));
        view.badge = " header:host ".into();
        reference.badge.clone_from(&view.badge);
        let mut tty = Terminal::with_size(80, 8);
        let mut full_tty = Terminal::with_size(80, 8);
        let mut underlying = Terminal::with_history(80, 8, 0);
        for first in [100, 99, 100, 101, 100] {
            let mut frame = String::from("\x1b[?2026h");
            for row in 0..8 {
                let id = first + row;
                let _ = write!(
                    frame,
                    "\x1b[{};1H\x1b[0m{id:04}{}\x1b[38;5;31m한글\x1b[0m\x1b[K",
                    row + 1,
                    "x".repeat(62)
                );
            }
            frame.push_str("\x1b[8;1H\x1b[?2026l");
            view.feed(frame.as_bytes());
            reference.feed(frame.as_bytes());
            underlying.feed(frame.as_bytes());
            reference.invalidate();
            tty.feed(&view.draw());
            full_tty.feed(&reference.draw());
            assert!(tty.same_screen(&full_tty));
            assert!(tty.screen_line(0).contains("header:host"));
            assert!((1..8).all(|row| !tty.screen_line(row).contains("header:host")));
        }
        // Hide an overlay without body output; restore every covered cell,
        // including a wide glyph crossed by the label's left boundary.
        view.badge.clear();
        tty.feed(&view.draw());
        for row in 0..8 {
            for col in 0..80 {
                assert_eq!(tty.cell(row, col), underlying.cell(row, col));
            }
        }
        view.badge = " header:host ".into();
        tty.feed(&view.draw());
        view.feed(b"\x1b[1;75H");
        underlying.feed(b"\x1b[1;75H");
        tty.feed(&view.draw());
        assert!(
            !tty.screen_line(0).contains("header:host"),
            "typing near the overlay reveals its underlay"
        );
        for col in 0..80 {
            assert_eq!(tty.cell(0, col), underlying.cell(0, col));
        }
        assert_eq!(tty.cursor(), (0, 74));
    }

    #[test]
    fn tui_frame_diff_detects_up_and_down_without_scroll_input() {
        let mut view = Viewport::new((100, 12));
        let mut reference = Viewport::new((100, 12));
        let mut tty = Terminal::with_size(100, 12);
        let mut full_tty = Terminal::with_size(100, 12);
        view.badge = " session:host ".into();
        reference.badge.clone_from(&view.badge);
        for target in [&mut view, &mut reference] {
            target.feed(b"\x1b[?1049h\x1b[?1000h\x1b[?1006h");
        }
        let mut previous = None;
        for first in [100, 101, 100] {
            let frame = tui_frame(first, 12);
            view.feed(&frame);
            reference.feed(&frame);
            reference.invalidate();
            let delta = view.draw();
            let full = reference.draw();
            tty.feed(&delta);
            full_tty.feed(&full);
            assert!(tty.same_screen(&full_tty));
            assert_eq!(
                view.terminal.display_offset(),
                0,
                "movement is inferred from frames, not history position"
            );
            assert!(tty.screen_line(0).ends_with("session:host"));
            assert_eq!(tty.screen_line(10), "> pending input stays fixed");
            if let Some(old) = previous {
                let output = String::from_utf8_lossy(&delta);
                assert!(
                    output.contains("\x1b[3;10r"),
                    "only the moving transcript belongs to the scroll region"
                );
                assert!(output.contains(if first > old { "\x1b[1S" } else { "\x1b[1T" }));
                assert!(!output.contains("pending input stays fixed"));
                assert!(tty.screen_line(0).starts_with("Coding assistant"));
                assert!(
                    delta.len() * 2 < full.len(),
                    "delta {} versus full {}",
                    delta.len(),
                    full.len()
                );
            }
            previous = Some(first);
        }
    }

    #[test]
    fn frame_diff_matches_full_render_across_redraws_resizes_and_repeated_rows() {
        let mut view = Viewport::new((100, 16));
        let mut reference = Viewport::new((100, 16));
        let mut tty = Terminal::with_size(100, 16);
        let mut full_tty = Terminal::with_size(100, 16);
        view.feed(b"\x1b[?1049h");
        reference.feed(b"\x1b[?1049h");
        let mut size = (100, 16);
        for step in 0..240 {
            if step == 100 || step == 180 {
                size = if step == 100 { (80, 10) } else { (100, 16) };
                view.resize(size);
                reference.resize(size);
                tty.resize(size.0, size.1);
                full_tty.resize(size.0, size.1);
            }
            let frame = tui_frame(100 + step * 7 % 31, size.1);
            view.feed(&frame);
            reference.feed(&frame);
            if step % 3 == 0 {
                // Repeated lines and a changed row within a moving block
                // must not turn a candidate row match into a stale screen.
                let edit = b"\x1b[5;1H\x1b[2Ksame repeated row\x1b[6;1H\x1b[2Ksame repeated row";
                view.feed(edit);
                reference.feed(edit);
            }
            reference.invalidate();
            tty.feed(&view.draw());
            full_tty.feed(&reference.draw());
            assert!(tty.same_screen(&full_tty), "frame {step}");
        }
        view.invalidate();
        let output = view.draw();
        assert!(!String::from_utf8_lossy(&output).contains("\x1b[1S"));
        tty.feed(&output);
        assert!(tty.same_screen(&full_tty));
    }

    #[test]
    fn row_movement_is_skipped_when_control_bytes_cost_more() {
        let mut view = Viewport::new((80, 3));
        view.rows = vec![b"x".to_vec(), b"a".to_vec(), b"b".to_vec()];
        let previous = view.rows.clone();
        let next = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        assert!(view.shift_rows(&next).is_none());
        assert_eq!(view.rows, previous);
    }

    fn populated() -> Viewport {
        let mut view = Viewport::new((80, 5));
        view.badge = " shell:host ".into();
        for n in 0..30 {
            view.feed(format!("line {n:02}\r\n").as_bytes());
        }
        view
    }

    #[test]
    fn scrolling_requests_server_frames_without_retaining_history() {
        let mut view = populated();
        let mut server = Terminal::with_size(80, 5);
        for n in 0..30 {
            server.feed(format!("line {n:02}\r\n").as_bytes());
        }
        let mut tty = Terminal::with_size(80, 5);
        tty.feed(&view.draw());
        assert_eq!(view.terminal.scrollback_len(), 0);
        assert!(tty.screen_line(0).starts_with("line 26"));
        let now = Instant::now();
        assert!(view.input(b"\x1b[<64;3;4M", false, now).is_empty());
        assert_eq!(view.request.position, 3);
        tty.feed(&view.draw());
        assert!(
            tty.screen_line(0).starts_with("line 26"),
            "only the server can select history"
        );
        server.scroll_display(3);
        view.feed(&server.render_view());
        view.status(ScrollStatus {
            serial: view.request.serial,
            offset: 3,
        });
        tty.feed(&view.draw());
        assert!(tty.screen_line(0).contains("[scroll +3]"));
        assert!(tty.screen_line(0).starts_with("line 23"));
        let body: Vec<_> = (1..5).map(|row| tty.screen_line(row)).collect();
        server.feed(b"new output\r\n");
        view.feed(&server.render_view());
        view.status(ScrollStatus {
            serial: view.request.serial,
            offset: server.display_offset() as u32,
        });
        tty.feed(&view.draw());
        assert_eq!(
            body,
            (1..5).map(|row| tty.screen_line(row)).collect::<Vec<_>>()
        );
        assert_eq!(view.input(b"x", false, now), b"x");
        assert_eq!(view.request.reset, view.request.serial);
        assert_eq!(view.request.position, 0);
        assert_eq!(view.terminal.scrollback_len(), 0);
        assert_eq!(tty.scrollback_len(), 0);
    }

    #[test]
    fn fragmented_wheel_pages_escape_and_paste_are_local_only_where_expected() {
        let mut view = populated();
        let now = Instant::now();
        assert!(view.input(b"\x1b[<64;", false, now).is_empty());
        assert!(view.input(b"3;4M", false, now).is_empty());
        assert_eq!(view.request.position, 3);
        assert!(view.input(b"\x1b[5;2~", false, now).is_empty());
        assert_eq!(view.request.position, 7);
        assert!(view.input(b"\x1b[6~", false, now).is_empty());
        assert_eq!(view.request.position, 3);
        assert!(view.input(b"\x1b", false, now).is_empty());
        assert!(view.input(b"", false, now + PARTIAL_WAIT).is_empty());
        assert_eq!(view.request.position, 0);
        let paste = b"\x1b[200~literal\x1b[5;2~\x1b[<64;3;4M\x1b[201~";
        assert_eq!(view.input(paste, false, now), paste);
        assert_eq!(view.request.position, 0);
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
    fn legacy_and_delayed_mouse_reports_never_become_shell_input() {
        let mut view = populated();
        let now = Instant::now();
        let up = [0x1b, b'[', b'M', 96, 42, 37];
        assert!(
            view.input(&up, false, now).is_empty(),
            "X10 wheel must request server scrolling"
        );
        assert_eq!(view.request.position, 3);
        assert!(
            view.input(b"\x1b[97;10;5M", false, now).is_empty(),
            "urxvt wheel must request server scrolling"
        );
        assert_eq!(view.request.position, 0);
        assert!(view.input(b"\x1b[<64;", false, now).is_empty());
        assert!(
            view.input(b"", false, now + PARTIAL_WAIT).is_empty(),
            "a delayed mouse tail must not type its prefix into the shell"
        );
        assert!(view
            .input(b"10;5M", false, now + PARTIAL_WAIT * 4)
            .is_empty());
        assert_eq!(view.request.position, 3);
        assert!(view.input(&up[..4], false, now).is_empty());
        assert!(view.input(b"", false, now + PARTIAL_WAIT).is_empty());
        assert!(view
            .input(&up[4..], false, now + PARTIAL_WAIT * 4)
            .is_empty());
        assert_eq!(view.request.position, 6);
        assert_eq!(view.input(b"echo ok\r", false, now), b"echo ok\r");
    }

    #[test]
    fn physical_mouse_encoding_survives_ghostty_style_resets() {
        // Ghostty 1.4.0 stream_terminal.zig resets the active encoding to
        // X10 on *any* of 1005/1006/1015/1016 being disabled.
        #[derive(Default)]
        struct Encoding(u16);
        impl Perform for Encoding {
            fn csi_dispatch(
                &mut self,
                params: &Params,
                intermediates: &[u8],
                _: bool,
                action: char,
            ) {
                if intermediates != b"?" {
                    return;
                }
                for param in params.iter() {
                    if let [mode @ (1005 | 1006 | 1015 | 1016)] = param {
                        match action {
                            'h' => self.0 = *mode,
                            'l' => self.0 = 0,
                            _ => {}
                        }
                    }
                }
            }
        }
        let terminal = Terminal::with_size(80, 5);
        for (pixel, expected) in [(false, 1006), (true, 1016)] {
            let mut encoding = Encoding::default();
            Parser::new().advance(&mut encoding, &terminal.display_modes(pixel));
            assert_eq!(encoding.0, expected);
        }
    }

    #[test]
    fn interrupted_mouse_reports_preserve_the_following_key() {
        let now = Instant::now();
        for prefix in [b"\x1b[<64;".as_slice(), b"\x1b[96;10;".as_slice()] {
            for key in [
                b"echo ok\r".as_slice(),
                b"cat\r",
                b"tree\r",
                b"npm\r",
                b"uname\r",
                b"yes\r",
                b"Rscript\r",
                b"~",
                b"\r",
                b"\x1b[A",
            ] {
                let mut view = Viewport::new((80, 5));
                assert!(view.input(prefix, false, now).is_empty());
                assert!(view.input_deadline().is_none());
                assert_eq!(view.input(key, false, now + PARTIAL_WAIT * 40), key);
            }
        }
        let mut view = Viewport::new((80, 5));
        assert!(view.input(b"\x1b[97;", false, now).is_empty());
        assert_eq!(
            view.input(b"1:2u", false, now + PARTIAL_WAIT * 4),
            b"\x1b[97;1:2u"
        );
        assert!(view.input(b"\x1b[96;", false, now).is_empty());
        assert_eq!(
            view.input(b"10R", false, now + PARTIAL_WAIT * 4),
            b"\x1b[96;10R"
        );
        assert!(view.input(b"\x1b[96;10;", false, now).is_empty());
        assert!(view.input(b"", false, now + PARTIAL_WAIT * 4).is_empty());
        assert!(view.input(b"5M", false, now + PARTIAL_WAIT * 5).is_empty());
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
        assert_eq!(view.input(b"\x1b[<64;10;3M", false, now), b"\x1b[<64;10;3M");
        assert_eq!(view.input(b"\x1b[<0;10;3m", false, now), b"\x1b[<0;10;3m");
        assert_eq!(view.input(b"\x1b[<0;10;1M", false, now), b"\x1b[<0;10;1M");
        let mut tty = Terminal::with_size(80, 5);
        tty.feed(&view.draw());
        assert_eq!(tty.cursor(), (2, 3));
        assert!(String::from_utf8_lossy(&tty.render(0)).contains("?1002h"));
        view.feed(b"\x1b[?1049h\x1b[H\x1b[?1006l\x1b[?1000l\x1b[?1002lFULL SCREEN");
        tty.feed(&view.draw());
        assert!(tty.screen_line(0).contains("FULL SCREEN"));
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
        assert_eq!(tty.cursor(), (2, 3));
    }

    #[test]
    fn resize_and_remote_reset_cannot_put_the_header_into_history() {
        let mut view = populated();
        let mut tty = Terminal::with_size(80, 5);
        tty.feed(&view.draw());
        tty.resize(40, 2);
        view.resize((40, 2));
        tty.feed(&view.draw());
        assert!(tty.screen_line(0).ends_with("shell:host"));
        view.feed(b"\x1bc\x1b[2Jreset\r\n");
        tty.feed(&view.draw());
        assert!(tty.alt_screen());
        assert!(tty.screen_line(0).ends_with("shell:host"));
        assert!(tty.screen_line(0).contains("reset"));
        tty.resize(40, 1);
        view.resize((40, 1));
        tty.feed(&view.draw());
        assert!(tty.screen_line(0).contains("shell:host"));
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
    fn pixel_mouse_keeps_full_screen_coordinates_after_a_font_resize() {
        let mut view = Viewport::new((80, 5));
        view.feed(b"\x1b[?1000h\x1b[?1016h");
        let now = Instant::now();
        view.draw();
        assert!(view.input(b"\x1b[6;20;10t", false, now).is_empty());
        view.draw();
        assert_eq!(
            view.input(b"\x1b[<0;100;61M", false, now),
            b"\x1b[<0;100;61M"
        );
        view.resize((70, 4));
        assert!(String::from_utf8_lossy(&view.draw()).contains("\x1b[16t"));
        assert!(view.input(b"\x1b[<0;100;61M", false, now).is_empty());
        assert!(view.input(b"\x1b[6;30;10t", false, now).is_empty());
        view.draw();
        assert_eq!(
            view.input(b"\x1b[<0;100;61M", false, now),
            b"\x1b[<0;100;61M"
        );
        assert_eq!(
            view.input(b"\x1b[<0;100;29M", false, now),
            b"\x1b[<0;100;29M"
        );
        view.feed(b"\x1b[14t");
        assert!(String::from_utf8_lossy(&view.replies).contains("\x1b[4;120;700t"));
        assert!(String::from_utf8_lossy(&view.finish()).contains("\x1b[?1016l"));
    }
}
