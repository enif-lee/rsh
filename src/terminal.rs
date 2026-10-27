// Screen state of a session, kept by alacritty_terminal. The server feeds it
// the PTY output so a client that attaches later gets the screen, its
// history and the modes the program turned on (`render`); the client keeps
// one as a mirror of its own terminal, to put back what rsh draws over it
// (`render_span`, `render_cell`, `cursor_sequence`) and to hand the terminal
// back clean (`reset_sequence`).
//
// alacritty does the parsing and the grid. This file adds what alacritty
// keeps private or ignores but rsh must reproduce: the scroll region, the
// character set in use, modifyOtherKeys, the cursor style, a few private
// modes, the window title, and screen's/tmux's `ESC k … ST` title string,
// which alacritty would print as text.

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::{Dimensions, Grid, Row};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::{Cell as GridCell, Flags};
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{
    self, CharsetIndex, ClearMode, Color as AnsiColor, CursorShape, CursorStyle, Handler, Hyperlink, KeyboardModes,
    KeyboardModesApplyBehavior, LineClearMode, Mode, ModifyOtherKeys, NamedColor, NamedPrivateMode, PrivateMode, Processor, Rgb,
    ScpCharPath, ScpUpdateMode, StandardCharset, TabulationClearMode,
};
use alacritty_terminal::vte::{Params, Parser, Perform};
use std::cell::RefCell;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

/// Lines of history a session keeps: as many as an attaching client gets
/// (`server::ATTACH_HISTORY`); more would only cost memory.
const SCROLLBACK_LIMIT: usize = 1000;
/// An escape sequence or string longer than this is cut off and the rest of
/// it skipped, so program output cannot grow the parsers' buffers.
const MAX_SEQUENCE: usize = 256 * 1024;
/// Kitty keyboard stack entries kept per screen.
const KITTY_DEPTH: u16 = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Color {
    #[default]
    Default,
    /// One of the 16 colors set with SGR 30–37, 40–47, 90–97 or 100–107.
    Named(u8),
    /// A 256-color palette entry set with `38;5;n`.
    Indexed(u8),
    Rgb(u8, u8, u8),
}

pub const BOLD: u16 = 1;
pub const DIM: u16 = 1 << 1;
pub const ITALIC: u16 = 1 << 2;
pub const REVERSE: u16 = 1 << 4;
pub const HIDDEN: u16 = 1 << 5;
pub const STRIKE: u16 = 1 << 6;
const UNDERLINE_SHIFT: u16 = 8;
const UNDERLINE_MASK: u16 = 0b111 << UNDERLINE_SHIFT;

/// A cell's colors and attributes, in the form rsh writes them back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Attr {
    pub fg: Color,
    pub bg: Color,
    pub underline_color: Color,
    pub flags: u16,
}

impl Attr {
    /// 0 none, 1 single, 2 double, 3 curly, 4 dotted, 5 dashed.
    pub fn underline(&self) -> u8 {
        ((self.flags & UNDERLINE_MASK) >> UNDERLINE_SHIFT) as u8
    }

    fn of(cell: &GridCell) -> Self {
        let mut flags = 0;
        for (bit, ours) in [
            (Flags::BOLD, BOLD),
            (Flags::DIM, DIM),
            (Flags::ITALIC, ITALIC),
            (Flags::INVERSE, REVERSE),
            (Flags::HIDDEN, HIDDEN),
            (Flags::STRIKEOUT, STRIKE),
        ] {
            if cell.flags.contains(bit) {
                flags |= ours;
            }
        }
        let underline = [
            (Flags::UNDERLINE, 1),
            (Flags::DOUBLE_UNDERLINE, 2),
            (Flags::UNDERCURL, 3),
            (Flags::DOTTED_UNDERLINE, 4),
            (Flags::DASHED_UNDERLINE, 5),
        ]
        .into_iter()
        .find(|(bit, _)| cell.flags.contains(*bit))
        .map_or(0, |(_, style)| style);
        flags |= underline << UNDERLINE_SHIFT;
        Attr {
            fg: color(cell.fg),
            bg: color(cell.bg),
            underline_color: cell.underline_color().map_or(Color::Default, color),
            flags,
        }
    }
}

fn color(color: AnsiColor) -> Color {
    match color {
        AnsiColor::Named(named) if (named as usize) < 16 => Color::Named(named as u8),
        AnsiColor::Named(_) => Color::Default,
        AnsiColor::Indexed(index) => Color::Indexed(index),
        AnsiColor::Spec(Rgb { r, g, b }) => Color::Rgb(r, g, b),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub attr: Attr,
    /// Combining marks and the hyperlink, hashed: 0 when there are none.
    pub extra: u64,
    pub link: bool,
}

const SPACER: char = '\0';

impl Cell {
    /// The right half of a wide character.
    pub fn is_spacer(&self) -> bool {
        self.ch == SPACER
    }

    fn of(cell: &GridCell) -> Self {
        let ch = if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
            SPACER
        } else if cell.flags.contains(Flags::LEADING_WIDE_CHAR_SPACER) || cell.c == '\t' {
            // alacritty keeps a tab's `\t` in the blank cells it skipped.
            ' '
        } else {
            cell.c
        };
        let link = cell.hyperlink();
        let marks = cell.zerowidth().unwrap_or_default();
        let extra = if link.is_none() && marks.is_empty() {
            0
        } else {
            use std::hash::{Hash, Hasher};
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            marks.hash(&mut hash);
            if let Some(link) = &link {
                (link.id(), link.uri()).hash(&mut hash);
            }
            hash.finish() | 1
        };
        Cell { ch, attr: Attr::of(cell), extra, link: link.is_some() }
    }
}

/// What alacritty reports through events: only the title matters here;
/// replies to queries are the user's terminal's to send.
#[derive(Clone, Default)]
struct Events {
    title: Arc<Mutex<String>>,
}

impl EventListener for Events {
    fn send_event(&self, event: Event) {
        let title = match event {
            Event::Title(title) => title,
            Event::ResetTitle => String::new(),
            _ => return,
        };
        if let Ok(mut slot) = self.title.lock() {
            *slot = title;
        }
    }
}

/// State alacritty keeps private or does not keep.
#[derive(Clone, Debug)]
struct Extra {
    /// Scroll region, `top..bottom`.
    region: (usize, usize),
    /// G1 shifted in (SO).
    shifted: bool,
    modify_other_keys: Option<u8>,
    /// DECSCUSR number, 0 for the default.
    cursor_style: u8,
    /// alacritty starts with alternate scroll on; a program turning it on
    /// is what counts.
    alternate_scroll: bool,
    /// Private modes alacritty does not know but rsh restores.
    modes: Vec<u16>,
    /// The program used the kitty keyboard protocol. Each screen has its own
    /// stack and only the active one is readable, so both are emptied when
    /// the terminal is handed back.
    kitty_used: bool,
    /// The alternate screen as `?47l`/`?1047l`-free exits left it: xterm
    /// keeps it for the next `?47h`/`?1047h`, alacritty would clear it.
    kept_alt: Option<Grid<GridCell>>,
    /// Kitty keyboard stack depth of the primary and alternate screens.
    kitty_depth: [u16; 2],
}

impl Extra {
    fn new(rows: usize) -> Self {
        Extra {
            region: (0, rows),
            shifted: false,
            modify_other_keys: None,
            cursor_style: 0,
            alternate_scroll: false,
            modes: Vec::new(),
            kitty_used: false,
            kept_alt: None,
            kitty_depth: [0; 2],
        }
    }
}

/// Reverse video, highlight mouse tracking, urxvt and SGR-pixel mouse.
const KEPT_MODES: [u16; 4] = [5, 1001, 1015, 1016];

/// Bytes kept from the parsers: a screen/tmux title string `ESC k … ST`,
/// and the rest of a sequence longer than `MAX_SEQUENCE`. Both end at BEL
/// or ST, and CAN or SUB cancels them as any string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Skip {
    Off,
    /// An ESC at ground, held to see whether `k` follows.
    Esc,
    In,
    InEsc,
}

/// What one byte did in the probe parser.
#[derive(Default)]
struct Probe {
    /// A character, sequence or string was finished.
    ended: bool,
    /// A C0 control was executed; inside a CSI or ESC sequence that does
    /// not end it.
    executed: bool,
    /// DECSTR (`CSI ! p`), which alacritty does not do.
    soft_reset: bool,
    /// Synchronized output (`CSI ? 2026 h`/`l`) begun or ended.
    sync: Option<bool>,
}

impl Perform for Probe {
    fn print(&mut self, _: char) {
        self.ended = true;
    }

    fn execute(&mut self, _: u8) {
        self.executed = true;
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _: bool, action: char) {
        self.ended = true;
        self.soft_reset = intermediates == b"!" && action == 'p';
        if intermediates == b"?" && matches!(action, 'h' | 'l') && params.iter().any(|param| param == [2026]) {
            self.sync = Some(action == 'h');
        }
    }

    fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {
        self.ended = true;
    }

    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {
        // Ended by BEL here; one ended by ESC is finished by the `\` after.
        self.ended = true;
    }

    fn unhook(&mut self) {
        self.ended = true;
    }
}

pub struct Terminal {
    /// In a cell because rendering an alternate screen briefly swaps to the
    /// primary one, which alacritty offers no other way to read.
    term: RefCell<Term<Events>>,
    parser: Processor,
    events: Events,
    extra: Extra,
    probe: Parser,
    ground: bool,
    /// An ESC started a sequence that has not ended.
    in_sequence: bool,
    /// The program is inside synchronized output (it began a frame and has
    /// not ended it).
    synchronized: bool,
    probe_soft_reset: bool,
    /// Bytes since the last ground point.
    run: usize,
    skip: Skip,
    /// The size asked for; the grid has at least 2 columns, as alacritty
    /// panics or hangs on wide characters in 1.
    size: (u16, u16),
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Terminal {
    pub fn new() -> Self {
        Self::with_size(80, 24)
    }

    pub fn with_size(cols: u16, rows: u16) -> Self {
        Self::with_history(cols, rows, SCROLLBACK_LIMIT)
    }

    /// `history` lines of scrollback; a client's mirror needs none.
    pub fn with_history(cols: u16, rows: u16, history: usize) -> Self {
        let size = (cols.max(1), rows.max(1));
        let (cols, rows) = (usize::from(size.0.max(2)), usize::from(size.1));
        let events = Events::default();
        let config = Config {
            scrolling_history: history,
            kitty_keyboard: true,
            ..Config::default()
        };
        let term = Term::new(config, &TermSize::new(cols, rows), events.clone());
        Terminal {
            term: RefCell::new(term),
            parser: Processor::new(),
            events,
            extra: Extra::new(rows),
            probe: Parser::new(),
            ground: true,
            in_sequence: false,
            synchronized: false,
            probe_soft_reset: false,
            run: 0,
            skip: Skip::Off,
            size,
        }
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        let (cols, rows) = (cols.max(1), rows.max(1));
        if (cols, rows) == self.size {
            return;
        }
        self.size = (cols, rows);
        self.term.get_mut().resize(TermSize::new(usize::from(cols.max(2)), usize::from(rows)));
        self.extra.region = (0, usize::from(rows));
        self.extra.kept_alt = None;
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.feed_to_ground(bytes);
    }

    /// Feeds `bytes` and returns how many of them end at a point where no
    /// escape sequence or UTF-8 character is half done.
    pub fn feed_to_ground(&mut self, bytes: &[u8]) -> usize {
        let mut pass = Vec::with_capacity(bytes.len());
        let mut cut = 0;
        let mut skip_to = 0;
        for (index, &byte) in bytes.iter().enumerate() {
            if index < skip_to {
                continue;
            }
            match self.skip {
                Skip::Off if byte == 0x1b && self.ground => {
                    self.skip = Skip::Esc;
                    self.ground = false;
                    continue;
                }
                // Printable ASCII at ground leaves the probe at ground: no
                // need to run it (most output is this).
                Skip::Off if self.ground && (0x20..0x7f).contains(&byte) => {
                    pass.push(byte);
                    self.run = 0;
                    cut = index + 1;
                    continue;
                }
                // So is a whole printable UTF-8 character.
                Skip::Off if self.ground && byte >= 0xc2 && printable_char(&bytes[index..]).is_some() => {
                    let end = index + printable_char(&bytes[index..]).expect("printable char");
                    pass.extend_from_slice(&bytes[index..end]);
                    self.run = 0;
                    cut = end;
                    skip_to = end;
                    continue;
                }
                Skip::Off => self.step(byte, &mut pass),
                Skip::Esc if byte == b'k' => {
                    self.skip = Skip::In;
                    continue;
                }
                // ESC ESC: the second restarts the sequence and the first
                // does nothing, so only the second is held.
                Skip::Esc if byte == 0x1b => continue,
                Skip::Esc => {
                    self.skip = Skip::Off;
                    self.step(0x1b, &mut pass);
                    self.step(byte, &mut pass);
                }
                Skip::In | Skip::InEsc if matches!(byte, 0x07 | 0x18 | 0x1a) || (self.skip == Skip::InEsc && byte == b'\\') => {
                    self.skip = Skip::Off;
                    self.ground = true;
                }
                Skip::In | Skip::InEsc => {
                    self.skip = if byte == 0x1b { Skip::InEsc } else { Skip::In };
                    continue;
                }
            }
            if self.ground {
                self.run = 0;
                cut = index + 1;
            } else {
                self.run += 1;
                if self.run > MAX_SEQUENCE {
                    // Too long: cancel it in both parsers and skip the rest.
                    self.step(0x18, &mut pass);
                    self.skip = Skip::In;
                    self.ground = false;
                    self.run = 0;
                }
            }
            if self.probe_soft_reset {
                self.probe_soft_reset = false;
                self.flush(&mut pass);
                let mut handler = Tracked {
                    term: self.term.get_mut(),
                    extra: &mut self.extra,
                };
                handler.soft_reset();
            }
        }
        self.flush(&mut pass);
        cut
    }

    fn flush(&mut self, pass: &mut Vec<u8>) {
        let mut handler = Tracked {
            term: self.term.get_mut(),
            extra: &mut self.extra,
        };
        self.parser.advance(&mut handler, pass);
        // Synchronized output is the user's terminal's to hold; this copy
        // applies everything at once.
        if self.parser.sync_timeout().sync_timeout().is_some() {
            self.parser.stop_sync(&mut handler);
        }
        pass.clear();
    }

    fn step(&mut self, byte: u8, pass: &mut Vec<u8>) {
        let mut probe = Probe::default();
        self.probe.advance(&mut probe, &[byte]);
        pass.push(byte);
        let cancel = matches!(byte, 0x18 | 0x1a);
        if byte == 0x1b {
            self.in_sequence = true;
            self.ground = false;
        } else if probe.ended || (probe.executed && (cancel || !self.in_sequence)) {
            self.in_sequence = false;
            self.ground = true;
            self.probe_soft_reset |= probe.soft_reset;
            if let Some(sync) = probe.sync {
                self.synchronized = sync;
            }
        } else {
            self.ground = false;
        }
    }

    /// The program began synchronized output and has not ended it.
    pub fn synchronized(&self) -> bool {
        self.synchronized
    }

    /// No escape sequence or UTF-8 character is in progress.
    pub fn at_ground(&self) -> bool {
        self.ground
    }

    pub fn title(&self) -> String {
        self.events.title.lock().map(|title| title.clone()).unwrap_or_default()
    }

    fn mode(&self) -> TermMode {
        *self.term.borrow().mode()
    }

    pub fn alt_screen(&self) -> bool {
        self.mode().contains(TermMode::ALT_SCREEN)
    }

    pub fn cursor_visible(&self) -> bool {
        self.mode().contains(TermMode::SHOW_CURSOR)
    }

    pub fn origin_mode(&self) -> bool {
        self.mode().contains(TermMode::ORIGIN)
    }

    pub fn insert_mode(&self) -> bool {
        self.mode().contains(TermMode::INSERT)
    }

    /// Kitty keyboard flags in effect.
    pub fn kitty_flags(&self) -> u8 {
        let shift = TermMode::DISAMBIGUATE_ESC_CODES.bits().trailing_zeros();
        ((self.mode() & TermMode::KITTY_KEYBOARD_PROTOCOL).bits() >> shift) as u8
    }

    pub fn scroll_region(&self) -> (usize, usize) {
        self.extra.region
    }

    pub fn cursor(&self) -> (usize, usize) {
        let point = self.term.borrow().grid().cursor.point;
        (point.line.0.max(0) as usize, point.column.0)
    }

    pub fn pending_wrap(&self) -> bool {
        self.term.borrow().grid().cursor.input_needs_wrap
    }

    pub fn pen(&self) -> Attr {
        Attr::of(&self.term.borrow().grid().cursor.template)
    }

    pub fn scrollback_len(&self) -> usize {
        self.term.borrow().grid().history_size()
    }

    /// Text of a history line, oldest first.
    pub fn scrollback_line(&self, index: usize) -> String {
        let term = self.term.borrow();
        let history = term.grid().history_size();
        if index >= history {
            return String::new();
        }
        row_text(&term.grid()[Line(index as i32 - history as i32)], term.columns())
    }

    pub fn screen_line(&self, row: usize) -> String {
        let term = self.term.borrow();
        if row >= term.screen_lines() {
            return String::new();
        }
        row_text(&term.grid()[Line(row as i32)], term.columns())
    }

    pub fn cell(&self, row: usize, col: usize) -> Option<Cell> {
        let term = self.term.borrow();
        (row < term.screen_lines() && col < term.columns()).then(|| Cell::of(&term.grid()[Line(row as i32)][Column(col)]))
    }

    /// True when the row holds nothing but spaces at or right of `col`.
    pub fn blank_from(&self, row: usize, col: usize) -> bool {
        let term = self.term.borrow();
        row < term.screen_lines() && (col..term.columns()).all(|c| matches!(term.grid()[Line(row as i32)][Column(c)].c, ' ' | '\t'))
    }

    /// The whole state as bytes for a fresh terminal of the same size: up
    /// to `scrollback` lines of history, the primary screen, the alternate
    /// screen if in use, the scroll region, the modes and the cursor.
    pub fn render(&self, scrollback: usize) -> Vec<u8> {
        let mut out = String::new();
        out.push_str("\x18\x1b[?2026h\x1b[0m\x1b(B\x0f\x1b[r\x1b[?6l\x1b[?7h\x1b[4l\x1b[?1049l\x1b[H\x1b[2J");
        let flags = KeyboardModes::from_bits_truncate(self.kitty_flags());
        let rows = {
            let mut term = self.term.borrow_mut();
            // The primary screen is only readable while it is active: swap
            // to it and back, then put the alternate screen back as it was
            // (entering it again clears it).
            let alt = if term.mode().contains(TermMode::ALT_SCREEN) {
                // Swapping re-applies the stack's top, which `CSI = … u`
                // may have changed; the flags are put back too.
                let grid = term.grid().clone();
                term.swap_alt();
                Some((grid, flags))
            } else {
                None
            };
            let (cols, rows) = (term.columns(), term.screen_lines());
            let history = scrollback.min(term.grid().history_size());
            if history > 0 {
                for back in (1..=history).rev() {
                    render_row(&mut out, &term.grid()[Line(-(back as i32))], cols);
                    out.push_str("\r\n");
                }
                for _ in 1..rows {
                    out.push('\n');
                }
            }
            render_screen(&mut out, term.grid());
            if let Some((grid, flags)) = alt {
                let saved = term.grid().cursor.point;
                let _ = write!(out, "\x1b[{};{}H\x1b[?1049h\x1b[H\x1b[2J", saved.line.0 + 1, saved.column.0 + 1);
                term.swap_alt();
                *term.grid_mut() = grid;
                term.set_keyboard_mode(flags, KeyboardModesApplyBehavior::Replace);
                render_screen(&mut out, term.grid());
            }
            rows
        };
        let (top, bottom) = self.extra.region;
        if top != 0 || bottom != rows {
            let _ = write!(out, "\x1b[{};{}r", top + 1, bottom);
        }
        self.render_modes(&mut out);
        out.push_str(&String::from_utf8_lossy(&self.cursor_sequence()));
        out.push_str("\x1b[?2026l");
        out.into_bytes()
    }

    fn render_modes(&self, out: &mut String) {
        out.push_str(&self.modes_text());
        let title: String = self.title().chars().take(1024).collect();
        if !title.is_empty() {
            let _ = write!(out, "\x1b]2;{title}\x07");
        }
    }

    /// The modes as `render` sets them, without the title.
    fn modes_text(&self) -> String {
        let mut out = String::new();
        let mode = self.mode();
        for (flag, number) in [
            (TermMode::APP_CURSOR, 1),
            (TermMode::MOUSE_REPORT_CLICK, 1000),
            (TermMode::MOUSE_DRAG, 1002),
            (TermMode::MOUSE_MOTION, 1003),
            (TermMode::FOCUS_IN_OUT, 1004),
            (TermMode::UTF8_MOUSE, 1005),
            (TermMode::SGR_MOUSE, 1006),
            (TermMode::BRACKETED_PASTE, 2004),
        ] {
            if mode.contains(flag) {
                let _ = write!(out, "\x1b[?{number}h");
            }
        }
        for number in &self.extra.modes {
            let _ = write!(out, "\x1b[?{number}h");
        }
        if self.extra.alternate_scroll {
            out.push_str("\x1b[?1007h");
        }
        if !mode.contains(TermMode::LINE_WRAP) {
            out.push_str("\x1b[?7l");
        }
        if mode.contains(TermMode::INSERT) {
            out.push_str("\x1b[4h");
        }
        if mode.contains(TermMode::LINE_FEED_NEW_LINE) {
            out.push_str("\x1b[20h");
        }
        if mode.contains(TermMode::APP_KEYPAD) {
            out.push_str("\x1b=");
        }
        if mode.contains(TermMode::ORIGIN) {
            out.push_str("\x1b[?6h");
        }
        if !mode.contains(TermMode::SHOW_CURSOR) {
            out.push_str("\x1b[?25l");
        }
        if self.extra.cursor_style != 0 {
            let _ = write!(out, "\x1b[{} q", self.extra.cursor_style);
        }
        if self.kitty_flags() != 0 {
            let _ = write!(out, "\x1b[>{}u", self.kitty_flags());
        }
        if let Some(level) = self.extra.modify_other_keys {
            let _ = write!(out, "\x1b[>4;{level}m");
        }
        out
    }

    fn rows(&self) -> Vec<Vec<Cell>> {
        let term = self.term.borrow();
        let cols = term.columns();
        (0..term.screen_lines()).map(|row| (0..cols).map(|col| Cell::of(&term.grid()[Line(row as i32)][Column(col)])).collect()).collect()
    }

    /// What sets the modes of `prev` to these, other than origin and insert
    /// mode; None for a change only a repaint makes (the kitty keyboard
    /// stack, modifyOtherKeys).
    fn mode_changes(&self, prev: &Terminal) -> Option<String> {
        if self.kitty_flags() != prev.kitty_flags() || self.extra.modify_other_keys != prev.extra.modify_other_keys {
            return None;
        }
        let (now, was) = (self.mode(), prev.mode());
        let mut out = String::new();
        let mut private = |on: bool, before: bool, number: u16| {
            if on != before {
                let _ = write!(out, "\x1b[?{number}{}", if on { 'h' } else { 'l' });
            }
        };
        for (flag, number) in [
            (TermMode::APP_CURSOR, 1),
            (TermMode::LINE_WRAP, 7),
            (TermMode::SHOW_CURSOR, 25),
            (TermMode::MOUSE_REPORT_CLICK, 1000),
            (TermMode::MOUSE_DRAG, 1002),
            (TermMode::MOUSE_MOTION, 1003),
            (TermMode::FOCUS_IN_OUT, 1004),
            (TermMode::UTF8_MOUSE, 1005),
            (TermMode::SGR_MOUSE, 1006),
            (TermMode::BRACKETED_PASTE, 2004),
        ] {
            private(now.contains(flag), was.contains(flag), number);
        }
        for number in self.extra.modes.iter().chain(&prev.extra.modes) {
            private(self.extra.modes.contains(number), prev.extra.modes.contains(number), *number);
        }
        private(self.extra.alternate_scroll, prev.extra.alternate_scroll, 1007);
        if now.contains(TermMode::LINE_FEED_NEW_LINE) != was.contains(TermMode::LINE_FEED_NEW_LINE) {
            out.push_str(if now.contains(TermMode::LINE_FEED_NEW_LINE) { "\x1b[20h" } else { "\x1b[20l" });
        }
        if now.contains(TermMode::APP_KEYPAD) != was.contains(TermMode::APP_KEYPAD) {
            out.push_str(if now.contains(TermMode::APP_KEYPAD) { "\x1b=" } else { "\x1b>" });
        }
        if self.extra.cursor_style != prev.extra.cursor_style {
            let _ = write!(out, "\x1b[{} q", self.extra.cursor_style);
        }
        Some(out)
    }

    /// Whether `other` shows the same screen: cells, cursor, region, modes.
    pub fn same_screen(&self, other: &Terminal) -> bool {
        self.size() == other.size()
            && self.alt_screen() == other.alt_screen()
            && self.cursor() == other.cursor()
            && self.pending_wrap() == other.pending_wrap()
            && self.scroll_region() == other.scroll_region()
            && self.modes_text() == other.modes_text()
            && self.title().chars().take(1024).eq(other.title().chars().take(1024))
            && self.rows() == other.rows()
    }

    /// Bytes that turn `prev` (the screen a terminal shows) into this one:
    /// a block of rows scrolled where the screen moved, the cells that
    /// differ, then region, title and cursor. As mosh sends screens, for a
    /// link too slow for every byte. None when only a repaint will do (a
    /// size, alternate screen or mode change).
    pub fn diff_from(&self, prev: &Terminal) -> Option<Vec<u8>> {
        if self.size() != prev.size() || self.alt_screen() != prev.alt_screen() {
            return None;
        }
        let modes = self.mode_changes(prev)?;
        let (cur, mut old) = (self.rows(), prev.rows());
        let rows = cur.len();
        // Rows compared by hash in the search (a wrong match is caught by
        // the caller's `same_screen`): rows² hashes, not rows² × cols cells.
        let hash = |row: &Vec<Cell>| {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            for cell in row {
                (cell.ch, cell.attr, cell.extra).hash(&mut h);
            }
            h.finish()
        };
        let (hc, ho): (Vec<u64>, Vec<u64>) = (cur.iter().map(hash).collect(), old.iter().map(hash).collect());
        let blank = Cell::of(&GridCell::default());
        let mut out = String::from("\x1b[?2026h\x1b[0m\x1b(B\x0f\x1b[?6l\x1b[4l");
        // The longest run of rows that moved together by k (up when k > 0).
        let mut best: Option<(isize, usize, usize)> = None;
        for k in (1..rows as isize).flat_map(|k| [k, -k]) {
            let mut start = None;
            for i in 0..=rows {
                let j = i as isize + k;
                let same = i < rows && j >= 0 && (j as usize) < rows && hc[i] == ho[j as usize] && hc[i] != ho[i];
                match (same, start) {
                    (true, None) => start = Some(i),
                    (false, Some(a)) => {
                        if i - a >= 2 && best.is_none_or(|(_, ba, bb)| i - a > bb - ba) {
                            best = Some((k, a, i));
                        }
                        start = None;
                    }
                    _ => {}
                }
            }
        }
        if let Some((k, a, b)) = best {
            // Rows a..b take old rows a+k..b+k: scroll the region they span.
            let (top, bottom) = if k > 0 { (a, b - 1 + k as usize) } else { ((a as isize + k) as usize, b - 1) };
            let n = k.unsigned_abs();
            let _ = write!(out, "\x1b[{};{}r\x1b[{};1H\x1b[{}{}", top + 1, bottom + 1, top + 1, n, if k > 0 { 'S' } else { 'T' });
            let region: Vec<Vec<Cell>> = old[top..=bottom].to_vec();
            for (offset, row) in old[top..=bottom].iter_mut().enumerate() {
                let from = offset as isize + k;
                *row = if from >= 0 && (from as usize) < region.len() { region[from as usize].clone() } else { vec![blank; row.len()] };
            }
            out.push_str("\x1b[r");
        }
        for row in 0..rows {
            let Some(first) = (0..cur[row].len()).find(|&col| cur[row][col] != old[row][col]) else {
                continue;
            };
            // Spans do not carry hyperlinks; a repaint does.
            if cur[row].iter().zip(&old[row]).any(|(new, was)| new != was && new.link) {
                return None;
            }
            let last = (0..cur[row].len()).rev().find(|&col| cur[row][col] != old[row][col]).unwrap_or(first);
            out.push_str(&String::from_utf8_lossy(&self.span(row, first, last + 1, false, true)));
        }
        let (top, bottom) = self.extra.region;
        if top != 0 || bottom != rows {
            let _ = write!(out, "\x1b[{};{}r", top + 1, bottom);
        } else {
            out.push_str("\x1b[r");
        }
        // Origin and insert mode were turned off to draw; the rest only
        // where they changed (`CSI > u` pushes a kitty stack entry: never
        // sent again for the same flags).
        let mode = self.mode();
        if mode.contains(TermMode::ORIGIN) {
            out.push_str("\x1b[?6h");
        }
        if mode.contains(TermMode::INSERT) {
            out.push_str("\x1b[4h");
        }
        out.push_str(&modes);
        let title: String = self.title().chars().take(1024).collect();
        if !title.chars().eq(prev.title().chars().take(1024)) {
            let _ = write!(out, "\x1b]2;{title}\x07");
        }
        out.push_str(&String::from_utf8_lossy(&self.cursor_sequence()));
        out.push_str("\x1b[?2026l");
        Some(out.into_bytes())
    }

    /// Repaints columns `from..to` of one row as the program left them,
    /// without moving anything else; the caller restores the cursor.
    pub fn render_span(&self, row: usize, from: usize, to: usize) -> Vec<u8> {
        self.span(row, from, to, false, false)
    }

    /// The same span drawn faint, to show the screen is not live.
    pub fn render_span_dimmed(&self, row: usize, from: usize, to: usize) -> Vec<u8> {
        self.span(row, from, to, true, false)
    }

    /// `delta`: a change of pen sets only what changed when nothing needs
    /// turning off (a screen update's run of colours costs a colour each).
    fn span(&self, row: usize, from: usize, to: usize, dim: bool, delta: bool) -> Vec<u8> {
        let term = self.term.borrow();
        let cols = term.columns();
        if row >= term.screen_lines() {
            return Vec::new();
        }
        let line = &term.grid()[Line(row as i32)];
        let to = to.min(cols);
        let mut from = from.min(to);
        if from > 0 && from < cols && line[Column(from)].flags.contains(Flags::WIDE_CHAR_SPACER) {
            from -= 1;
        }
        let mut out = String::from("\x1b(B\x0f");
        let _ = write!(out, "\x1b[{};{}H", row + 1, from + 1);
        let mut attr = None;
        let mut col = from;
        while col < to {
            let cell = &line[Column(col)];
            let mut ours = Attr::of(cell);
            if dim {
                ours.flags = (ours.flags | DIM) & !BOLD;
            }
            if attr != Some(ours) {
                match attr {
                    Some(old) if delta => out.push_str(&sgr_from(&old, &ours)),
                    _ => {
                        out.push_str("\x1b[0m");
                        out.push_str(&sgr_string(&ours));
                    }
                }
                attr = Some(ours);
            }
            push_cell(&mut out, cell);
            col += if cell.flags.contains(Flags::WIDE_CHAR) { 2 } else { 1 };
            if !cell.c.is_ascii() {
                let _ = write!(out, "\x1b[{}G", col + 1);
            }
        }
        out.push_str("\x1b[0m");
        out.into_bytes()
    }

    /// Repaints one cell as the program left it; the pen is the caller's to
    /// set afterwards.
    pub fn render_cell(&self, row: usize, col: usize) -> Vec<u8> {
        let col = col.min(usize::from(self.size().0) - 1);
        self.render_span(row, col, col + 1)
    }

    /// Puts the pen, character sets and cursor back where the program left
    /// them, without DECSC/DECRC, which would clobber the program's own save.
    pub fn cursor_sequence(&self) -> Vec<u8> {
        let term = self.term.borrow();
        let grid = term.grid();
        let cursor = &grid.cursor;
        let mut row = cursor.point.line.0.max(0) as usize;
        if term.mode().contains(TermMode::ORIGIN) {
            row = row.saturating_sub(self.extra.region.0);
        }
        let mut out = String::from("\x1b(B\x0f");
        if cursor.input_needs_wrap && term.mode().contains(TermMode::LINE_WRAP) {
            // The cursor sits past the last column; the only way to get a
            // terminal there is to print that column's character again.
            let line = &grid[cursor.point.line];
            let mut col = term.columns() - 1;
            if col > 0 && line[Column(col)].flags.contains(Flags::WIDE_CHAR_SPACER) {
                col -= 1;
            }
            let cell = &line[Column(col)];
            let _ = write!(out, "\x1b[{};{}H\x1b[0m{}", row + 1, col + 1, sgr_string(&Attr::of(cell)));
            push_cell(&mut out, cell);
        } else {
            let _ = write!(out, "\x1b[{};{}H", row + 1, cursor.point.column.0 + 1);
        }
        out.push_str("\x1b[0m");
        out.push_str(&sgr_string(&Attr::of(&cursor.template)));
        let graphics = |index| cursor.charsets[index] == StandardCharset::SpecialCharacterAndLineDrawing;
        out.push_str(if graphics(CharsetIndex::G0) { "\x1b(0" } else { "\x1b(B" });
        out.push_str(if graphics(CharsetIndex::G1) { "\x1b)0" } else { "\x1b)B" });
        out.push(if self.extra.shifted { '\x0e' } else { '\x0f' });
        out.into_bytes()
    }

    /// Bytes that turn off every mode the program enabled, used when the
    /// client hands its tty back to the local shell.
    pub fn reset_sequence(&self) -> Vec<u8> {
        let mode = self.mode();
        let mut out = String::from("\x1b[?2026l\x1b[0m\x1b[r");
        // Pops more entries than any program pushes: the stack empties.
        let kitty = if self.extra.kitty_used { "\x1b[<99u" } else { "" };
        out.push_str(kitty);
        if mode.contains(TermMode::ALT_SCREEN) {
            out.push_str("\x1b[?1049l");
            out.push_str(kitty);
        }
        out.push_str("\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1005l\x1b[?1006l\x1b[?1015l\x1b[?1016l");
        out.push_str("\x1b[?1004l\x1b[?2004l\x1b[?1l\x1b>\x1b[?25h\x1b(B\x0f");
        if self.extra.alternate_scroll {
            out.push_str("\x1b[?1007l");
        }
        if self.extra.modify_other_keys.is_some() {
            out.push_str("\x1b[>4m");
        }
        if self.extra.cursor_style != 0 {
            out.push_str("\x1b[0 q");
        }
        for number in &self.extra.modes {
            if !matches!(number, 1015 | 1016) {
                let _ = write!(out, "\x1b[?{number}l");
            }
        }
        if !mode.contains(TermMode::LINE_WRAP) {
            out.push_str("\x1b[?7h");
        }
        out.into_bytes()
    }
}

fn push_cell(out: &mut String, cell: &GridCell) {
    let ch = Cell::of(cell).ch;
    out.push(if ch == SPACER { ' ' } else { ch });
    for mark in cell.zerowidth().unwrap_or_default() {
        out.push(*mark);
    }
}

fn row_text(row: &Row<GridCell>, cols: usize) -> String {
    let mut out = String::new();
    for col in 0..cols {
        let cell = &row[Column(col)];
        if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
            continue;
        }
        push_cell(&mut out, cell);
    }
    out.trim_end_matches(' ').to_string()
}

fn render_screen(out: &mut String, grid: &Grid<GridCell>) {
    for row in 0..grid.screen_lines() {
        let _ = write!(out, "\x1b[{};1H", row + 1);
        render_row(out, &grid[Line(row as i32)], grid.columns());
    }
}

fn is_default_blank(cell: &GridCell) -> bool {
    matches!(cell.c, ' ' | '\t')
        && cell.fg == AnsiColor::Named(NamedColor::Foreground)
        && cell.bg == AnsiColor::Named(NamedColor::Background)
        && (cell.flags - Flags::WRAPLINE).is_empty()
        && cell.zerowidth().is_none()
        && cell.hyperlink().is_none()
        && cell.underline_color().is_none()
}

/// One row, from column 1 where the cursor is: cells, links, the line's end.
fn render_row(out: &mut String, row: &Row<GridCell>, cols: usize) {
    let mut end = cols;
    while end > 0 && is_default_blank(&row[Column(end - 1)]) {
        end -= 1;
    }
    let mut attr = Attr::default();
    let mut link: Option<(String, String)> = None;
    out.push_str("\x1b[0m");
    let mut drift = false;
    for col in 0..end {
        let cell = &row[Column(col)];
        if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
            continue;
        }
        if drift {
            let _ = write!(out, "\x1b[{}G", col + 1);
            drift = false;
        }
        let cell_link = cell.hyperlink().map(|l| (l.id().to_string(), l.uri().to_string()));
        if cell_link != link {
            match &cell_link {
                Some((id, uri)) => {
                    let _ = write!(out, "\x1b]8;id={id};{uri}\x1b\\");
                }
                None => out.push_str("\x1b]8;;\x1b\\"),
            }
            link = cell_link;
        }
        let ours = Attr::of(cell);
        if ours != attr {
            out.push_str("\x1b[0m");
            out.push_str(&sgr_string(&ours));
            attr = ours;
        }
        push_cell(out, cell);
        if !cell.c.is_ascii() {
            drift = true;
        }
    }
    if link.is_some() {
        out.push_str("\x1b]8;;\x1b\\");
    }
    out.push_str("\x1b[0m");
    if end < cols {
        if drift {
            let _ = write!(out, "\x1b[{}G", end + 1);
        }
        out.push_str("\x1b[K");
    }
}

pub fn sgr_string(attr: &Attr) -> String {
    if *attr == Attr::default() {
        return String::new();
    }
    let mut out = String::from("\x1b[0");
    for (bit, code) in [(BOLD, 1), (DIM, 2), (ITALIC, 3), (REVERSE, 7), (HIDDEN, 8), (STRIKE, 9)] {
        if attr.flags & bit != 0 {
            let _ = write!(out, ";{code}");
        }
    }
    match attr.underline() {
        0 => {}
        1 => out.push_str(";4"),
        style => {
            let _ = write!(out, ";4:{style}");
        }
    }
    color_params(&mut out, 38, attr.fg);
    color_params(&mut out, 48, attr.bg);
    color_params(&mut out, 58, attr.underline_color);
    out.push('m');
    out
}

/// The SGR that takes the pen from `old` to `new`: only what changed when
/// nothing needs turning off, else a reset and all of `new`.
pub fn sgr_from(old: &Attr, new: &Attr) -> String {
    let plain = !UNDERLINE_MASK;
    if old.flags & plain & !new.flags != 0 || (old.underline() != 0 && new.underline() == 0) {
        return if *new == Attr::default() { "\x1b[0m".into() } else { sgr_string(new) };
    }
    let mut out = String::from("\x1b[");
    for (bit, code) in [(BOLD, 1), (DIM, 2), (ITALIC, 3), (REVERSE, 7), (HIDDEN, 8), (STRIKE, 9)] {
        if new.flags & bit != 0 && old.flags & bit == 0 {
            let _ = write!(out, ";{code}");
        }
    }
    match new.underline() {
        style if style == old.underline() => {}
        1 => out.push_str(";4"),
        style => {
            let _ = write!(out, ";4:{style}");
        }
    }
    for (base, was, now) in [(38, old.fg, new.fg), (48, old.bg, new.bg), (58, old.underline_color, new.underline_color)] {
        if was != now {
            if now == Color::Default {
                let _ = write!(out, ";{}", base + 1);
            } else {
                color_params(&mut out, base, now);
            }
        }
    }
    // The first parameter's separator.
    out.remove(2);
    out.push('m');
    out
}

fn color_params(out: &mut String, base: u16, color: Color) {
    match color {
        Color::Default => {}
        Color::Named(n) if base != 58 && n < 8 => {
            let _ = write!(out, ";{}", base - 8 + u16::from(n));
        }
        Color::Named(n) if base != 58 => {
            let _ = write!(out, ";{}", base + 52 + u16::from(n) - 8);
        }
        Color::Named(n) | Color::Indexed(n) => {
            let _ = write!(out, ";{base};5;{n}");
        }
        Color::Rgb(r, g, b) => {
            let _ = write!(out, ";{base};2;{r};{g};{b}");
        }
    }
}

/// alacritty's handler, watched for the state it keeps private.
struct Tracked<'a> {
    term: &'a mut Term<Events>,
    extra: &'a mut Extra,
}

impl Tracked<'_> {
    /// DECSTR, as xterm: insert, origin, application keys and keypad off,
    /// wrap and cursor on, full scroll region, default pen and character
    /// sets, no saved cursor; the cursor stays where it is.
    fn soft_reset(&mut self) {
        let point = self.term.grid().cursor.point;
        self.term.unset_mode(Mode::Named(ansi::NamedMode::Insert));
        self.unset_private_mode(PrivateMode::Named(NamedPrivateMode::Origin));
        self.term.unset_private_mode(PrivateMode::Named(NamedPrivateMode::CursorKeys));
        self.term.unset_keypad_application_mode();
        self.term.set_private_mode(PrivateMode::Named(NamedPrivateMode::LineWrap));
        self.term.set_private_mode(PrivateMode::Named(NamedPrivateMode::ShowCursor));
        self.set_scrolling_region(1, None);
        self.term.terminal_attribute(ansi::Attr::Reset);
        for index in [CharsetIndex::G0, CharsetIndex::G1, CharsetIndex::G2, CharsetIndex::G3] {
            self.term.configure_charset(index, StandardCharset::Ascii);
        }
        self.set_active_charset(CharsetIndex::G0);
        self.term.grid_mut().saved_cursor = Default::default();
        self.term.grid_mut().cursor.point = point;
    }

    /// CUU/CUD and CPL/CNL. alacritty adds the scroll region's top twice in
    /// origin mode and ignores the margins otherwise; as xterm, a cursor
    /// inside the region stops at its margins.
    fn vertical(&mut self, delta: i32, column: Option<usize>) {
        let (top, bottom) = (self.extra.region.0 as i32, self.extra.region.1 as i32);
        let rows = self.term.screen_lines() as i32;
        let line = self.term.grid().cursor.point.line.0;
        let inside = (top..bottom).contains(&line);
        let (low, high) = if inside { (top, bottom - 1) } else { (0, rows - 1) };
        let target = (line + delta).clamp(low, high.max(low));
        let column = column.unwrap_or(self.term.grid().cursor.point.column.0);
        if self.term.mode().contains(TermMode::ORIGIN) {
            self.term.goto(target - top, column);
        } else {
            self.term.goto(target, column);
        }
    }

    fn private_mode(&mut self, mode: &PrivateMode, on: bool) {
        match mode {
            PrivateMode::Named(NamedPrivateMode::AlternateScroll) => self.extra.alternate_scroll = on,
            // The older alternate screen modes, which alacritty does not
            // know: as xterm, both screens share the cursor, the alternate
            // screen keeps its text between visits, and `?1047l` clears it.
            PrivateMode::Unknown(number @ (47 | 1047)) => {
                if on == self.term.mode().contains(TermMode::ALT_SCREEN) {
                    return;
                }
                let point = self.term.grid().cursor.point;
                if on {
                    self.term.swap_alt();
                    if let Some(mut kept) = self.extra.kept_alt.take() {
                        kept.cursor = self.term.grid().cursor.clone();
                        *self.term.grid_mut() = kept;
                    }
                } else {
                    self.extra.kept_alt = (*number == 47).then(|| self.term.grid().clone());
                    self.term.swap_alt();
                }
                self.term.grid_mut().cursor.point = point;
            }
            PrivateMode::Named(NamedPrivateMode::SwapScreenAndSetRestoreCursor) => self.extra.kept_alt = None,
            // Save and restore the cursor, as DECSC/DECRC.
            PrivateMode::Unknown(1048) if on => self.term.save_cursor_position(),
            PrivateMode::Unknown(1048) => self.term.restore_cursor_position(),
            PrivateMode::Unknown(number) if KEPT_MODES.contains(number) => {
                self.extra.modes.retain(|m| m != number);
                if on {
                    self.extra.modes.push(*number);
                }
            }
            _ => {}
        }
    }
}

macro_rules! forward {
    ($( fn $name:ident(&mut self $(, $arg:ident: $ty:ty)*); )*) => {
        $( fn $name(&mut self $(, $arg: $ty)*) { self.term.$name($($arg),*) } )*
    };
}

impl Handler for Tracked<'_> {
    forward! {
        fn input(&mut self, c: char);
        fn goto(&mut self, line: i32, col: usize);
        fn goto_line(&mut self, line: i32);
        fn goto_col(&mut self, col: usize);
        fn insert_blank(&mut self, count: usize);
        fn identify_terminal(&mut self, intermediate: Option<char>);
        fn device_status(&mut self, arg: usize);
        fn move_forward(&mut self, col: usize);
        fn move_backward(&mut self, col: usize);
        fn put_tab(&mut self, count: u16);
        fn backspace(&mut self);
        fn carriage_return(&mut self);
        fn linefeed(&mut self);
        fn bell(&mut self);
        fn substitute(&mut self);
        fn newline(&mut self);
        fn set_horizontal_tabstop(&mut self);
        fn scroll_up(&mut self, count: usize);
        fn scroll_down(&mut self, count: usize);
        fn insert_blank_lines(&mut self, count: usize);
        fn delete_lines(&mut self, count: usize);
        fn erase_chars(&mut self, count: usize);
        fn delete_chars(&mut self, count: usize);
        fn move_backward_tabs(&mut self, count: u16);
        fn move_forward_tabs(&mut self, count: u16);
        fn save_cursor_position(&mut self);
        fn restore_cursor_position(&mut self);
        fn clear_line(&mut self, mode: LineClearMode);
        fn clear_screen(&mut self, mode: ClearMode);
        fn clear_tabs(&mut self, mode: TabulationClearMode);
        fn set_tabs(&mut self, interval: u16);
        fn reverse_index(&mut self);
        fn terminal_attribute(&mut self, attr: ansi::Attr);
        fn set_mode(&mut self, mode: Mode);
        fn unset_mode(&mut self, mode: Mode);
        fn report_mode(&mut self, mode: Mode);
        fn report_private_mode(&mut self, mode: PrivateMode);
        fn set_keypad_application_mode(&mut self);
        fn unset_keypad_application_mode(&mut self);
        fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset);
        fn set_color(&mut self, index: usize, color: Rgb);
        fn dynamic_color_sequence(&mut self, prefix: String, index: usize, terminator: &str);
        fn reset_color(&mut self, index: usize);
        fn clipboard_store(&mut self, clipboard: u8, data: &[u8]);
        fn clipboard_load(&mut self, clipboard: u8, terminator: &str);
        fn decaln(&mut self);
        fn push_title(&mut self);
        fn pop_title(&mut self);
        fn text_area_size_pixels(&mut self);
        fn text_area_size_chars(&mut self);
        fn set_hyperlink(&mut self, link: Option<Hyperlink>);
        fn report_keyboard_mode(&mut self);
        fn report_modify_other_keys(&mut self);
        fn set_scp(&mut self, char_path: ScpCharPath, update_mode: ScpUpdateMode);
        fn set_title(&mut self, title: Option<String>);
        fn set_cursor_shape(&mut self, shape: CursorShape);
    }

    fn move_up(&mut self, count: usize) {
        self.vertical(-(count.min(u16::MAX as usize) as i32), None);
    }

    fn move_down(&mut self, count: usize) {
        self.vertical(count.min(u16::MAX as usize) as i32, None);
    }

    fn move_up_and_cr(&mut self, count: usize) {
        self.vertical(-(count.min(u16::MAX as usize) as i32), Some(0));
    }

    fn move_down_and_cr(&mut self, count: usize) {
        self.vertical(count.min(u16::MAX as usize) as i32, Some(0));
    }

    /// Kept to `KITTY_DEPTH` per screen: alacritty's own limit is 4096 and
    /// trimming at it panics (it removes from the title stack).
    fn push_keyboard_mode(&mut self, mode: KeyboardModes) {
        self.extra.kitty_used = true;
        let depth = &mut self.extra.kitty_depth[usize::from(self.term.mode().contains(TermMode::ALT_SCREEN))];
        if *depth < KITTY_DEPTH {
            *depth += 1;
            self.term.push_keyboard_mode(mode);
        }
    }

    fn pop_keyboard_modes(&mut self, to_pop: u16) {
        let depth = &mut self.extra.kitty_depth[usize::from(self.term.mode().contains(TermMode::ALT_SCREEN))];
        *depth = depth.saturating_sub(to_pop);
        self.term.pop_keyboard_modes(to_pop);
    }

    fn set_keyboard_mode(&mut self, mode: KeyboardModes, behavior: KeyboardModesApplyBehavior) {
        self.extra.kitty_used = true;
        self.term.set_keyboard_mode(mode, behavior);
    }

    fn set_cursor_style(&mut self, style: Option<CursorStyle>) {
        self.extra.cursor_style = style.map_or(0, |style| {
            let base = match style.shape {
                CursorShape::Underline => 3,
                CursorShape::Beam => 5,
                _ => 1,
            };
            if style.blinking {
                base
            } else {
                base + 1
            }
        });
        self.term.set_cursor_style(style);
    }

    fn set_private_mode(&mut self, mode: PrivateMode) {
        self.private_mode(&mode, true);
        // DECCOLM would clear the screen and the region; xterm ignores it by
        // default, and so did the user's terminal.
        if mode != PrivateMode::Named(NamedPrivateMode::ColumnMode) {
            self.term.set_private_mode(mode);
        }
    }

    fn unset_private_mode(&mut self, mode: PrivateMode) {
        self.private_mode(&mode, false);
        if mode != PrivateMode::Named(NamedPrivateMode::ColumnMode) {
            self.term.unset_private_mode(mode);
        }
    }

    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        let rows = self.term.screen_lines();
        let end = bottom.unwrap_or(rows);
        if top < end {
            self.extra.region = ((top.max(1) - 1).min(rows), end.min(rows));
        }
        self.term.set_scrolling_region(top, bottom);
    }

    fn set_active_charset(&mut self, index: CharsetIndex) {
        self.extra.shifted = index == CharsetIndex::G1;
        self.term.set_active_charset(index);
    }

    fn set_modify_other_keys(&mut self, mode: ModifyOtherKeys) {
        self.extra.modify_other_keys = match mode {
            ModifyOtherKeys::Reset => None,
            ModifyOtherKeys::EnableExceptWellDefined => Some(1),
            ModifyOtherKeys::EnableAll => Some(2),
        };
        self.term.set_modify_other_keys(mode);
    }

    fn reset_state(&mut self) {
        *self.extra = Extra::new(self.term.screen_lines());
        self.term.reset_state();
    }
}

/// Length of the UTF-8 character `bytes` starts with, when it is complete
/// and printable (not a C1 control, which a parser executes).
fn printable_char(bytes: &[u8]) -> Option<usize> {
    let len = match bytes.first()? {
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return None,
    };
    let ch = std::str::from_utf8(bytes.get(..len)?).ok()?.chars().next()?;
    (ch >= '\u{a0}').then_some(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh terminal fed `t.render(..)` shows the same.
    fn copy_of(t: &Terminal, history: usize) -> Terminal {
        let (cols, rows) = t.size();
        let mut copy = Terminal::with_size(cols, rows);
        copy.feed(&t.render(history));
        for row in 0..usize::from(rows) {
            assert_eq!(copy.screen_line(row), t.screen_line(row), "row {row}");
            for col in 0..usize::from(cols) {
                assert_eq!(copy.cell(row, col), t.cell(row, col), "cell {row},{col}");
            }
        }
        assert_eq!(copy.cursor(), t.cursor());
        assert_eq!(copy.pen(), t.pen());
        assert_eq!(copy.mode(), t.mode(), "modes");
        assert_eq!(copy.scroll_region(), t.scroll_region());
        assert_eq!(copy.title(), t.title());
        copy
    }

    #[test]
    fn render_round_trips_screen_colors_and_modes() {
        let mut t = Terminal::with_size(20, 6);
        t.feed("plain \x1b[1;31mred\x1b[0m \x1b[4:3;38;5;200;48;2;1;2;3mfancy\x1b[0m\r\n한글 e\u{301} 🚀\r\n".as_bytes());
        t.feed(b"\x1b]8;;http://x\x1b\\link\x1b]8;;\x1b\\\r\n\x1b]2;the title\x07");
        t.feed(b"\x1b[?1h\x1b=\x1b[?1000;1002;1006;2004;1004h\x1b[?1016h\x1b[2;5r\x1b[3;4H\x1b[7mpen");
        let copy = copy_of(&t, 0);
        assert_eq!(copy.cell(0, 6).unwrap().attr.fg, Color::Named(1));
        assert_eq!(copy.cell(0, 10).unwrap().attr.underline(), 3);
        assert_eq!(copy.cell(0, 10).unwrap().attr.bg, Color::Rgb(1, 2, 3));
        assert_eq!(copy.screen_line(1), "한글 e\u{301} 🚀");
        assert_eq!(copy.title(), "the title");
        let rendered = String::from_utf8_lossy(&t.render(0)).into_owned();
        assert!(rendered.contains("\x1b[?1016h"));
        assert!(rendered.contains("\x1b]8;id="));
    }

    #[test]
    fn alternate_screen_keeps_the_primary_underneath() {
        let mut t = Terminal::with_size(10, 4);
        t.feed(b"shell$ ls\r\nfile\r\nshell$ ");
        t.feed(b"\x1b[?1049h\x1b[Hvim here\x1b[2;3H");
        assert!(t.alt_screen());
        let mut copy = copy_of(&t, 0);
        assert_eq!(t.screen_line(0), "vim here", "rendering does not disturb the alternate screen");
        assert_eq!(t.cursor(), (1, 2));
        copy.feed(b"\x1b[?1049l");
        t.feed(b"\x1b[?1049l");
        assert_eq!(copy.screen_line(0), "shell$ ls");
        assert_eq!(copy.screen_line(2), "shell$");
        assert_eq!(copy.cursor(), t.cursor());
    }

    #[test]
    fn older_alternate_screen_keeps_its_text_and_the_cursor() {
        let mut t = Terminal::with_size(10, 3);
        t.feed(b"\x1b[?47hKeep\x1b[?47l");
        assert_eq!(t.cursor(), (0, 4), "the cursor is shared");
        t.feed(b"\x1b[?1047hZ");
        assert_eq!(t.screen_line(0), "KeepZ", "47 left the text for the next visit");
        t.feed(b"\x1b[?1047l\x1b[?47h");
        assert_eq!(t.screen_line(0), "", "1047l clears it");
        let reset = String::from_utf8(t.reset_sequence()).unwrap();
        assert!(reset.contains("\x1b[?1049l"));
        let mut m = Terminal::new();
        m.feed(b"\x1b[?1001h\x1b[?5h");
        let reset = String::from_utf8(m.reset_sequence()).unwrap();
        assert!(reset.contains("\x1b[?1001l") && reset.contains("\x1b[?5l"), "{reset:?}");
    }

    #[test]
    fn click_mode_alone_is_restored() {
        let mut t = Terminal::new();
        t.feed(b"\x1b[?1000;1006h");
        let copy = copy_of(&t, 0);
        assert!(copy.mode().contains(TermMode::MOUSE_REPORT_CLICK | TermMode::SGR_MOUSE));
        assert!(String::from_utf8(t.reset_sequence()).unwrap().contains("\x1b[?1000l"));
    }

    #[test]
    fn older_alternate_screen_modes() {
        for mode in ["47", "1047"] {
            let mut t = Terminal::with_size(10, 3);
            t.feed(format!("main\x1b[?{mode}hALT").as_bytes());
            assert!(t.alt_screen(), "{mode}");
            assert!(t.screen_line(0).ends_with("ALT"));
            t.feed(format!("\x1b[?{mode}l").as_bytes());
            assert!(!t.alt_screen());
            assert_eq!(t.screen_line(0), "main");
        }
    }

    #[test]
    fn history_round_trips() {
        let mut t = Terminal::with_size(12, 3);
        t.feed("one\r\n\x1b[31m빨강\x1b[0m two\r\nthree\r\nfour\r\nfive".as_bytes());
        assert_eq!(t.scrollback_len(), 2);
        assert_eq!(t.scrollback_line(0), "one");
        let copy = copy_of(&t, 10);
        assert_eq!(copy.scrollback_line(copy.scrollback_len() - 1), "빨강 two");
        assert_eq!(copy.scrollback_line(copy.scrollback_len() - 2), "one");
    }

    #[test]
    fn ground_points() {
        let mut t = Terminal::new();
        assert_eq!(t.feed_to_ground(b"$ \x1b[3"), 2);
        assert!(!t.at_ground());
        assert_eq!(t.feed_to_ground(b"1mRED\xed\x95"), 5);
        assert_eq!(t.feed_to_ground(b"\x9c"), 1);
        assert_eq!(t.screen_line(0), "$ RED한");
        assert_eq!(t.feed_to_ground(b"\x1b]2;x\x1b"), 0);
        assert_eq!(t.feed_to_ground(b"\\ok"), 3);
        assert_eq!(t.title(), "x");
        // Whole characters take the fast path; a C1 control in UTF-8 does not.
        let mut t = Terminal::new();
        assert_eq!(t.feed_to_ground("가 😀 é".as_bytes()), "가 😀 é".len());
        assert_eq!(t.feed_to_ground("\u{9b}x".as_bytes()), 3);
        assert_eq!(t.screen_line(0), "가 😀 éx");
    }

    #[test]
    fn screen_title_string_is_not_text() {
        let mut t = Terminal::with_size(20, 2);
        assert_eq!(t.feed_to_ground(b"\x1bk~x\x1b\\zsh\x1bkvim"), 9);
        t.feed(b"\x07!\x1b[31mr");
        assert_eq!(t.screen_line(0), "zsh!r");
        assert_eq!(t.cursor(), (0, 5));
        assert_eq!(t.cell(0, 4).unwrap().attr.fg, Color::Named(1), "an ESC not followed by k still works");
    }

    #[test]
    fn spans_repaint_in_place() {
        let mut t = Terminal::with_size(12, 2);
        t.feed("ab\x1b[31m한글\x1b[0mxyz".as_bytes());
        let mut local = Terminal::with_size(12, 2);
        local.feed("ab\x1b[31m한글\x1b[0mxyz".as_bytes());
        local.feed(b"\x1b[1;2HZZZZZZZZ");
        local.feed(&t.render_span(0, 3, 10));
        local.feed(&t.render_span(0, 1, 3));
        assert_eq!(local.screen_line(0), t.screen_line(0));
        assert_eq!(local.cell(0, 2).unwrap().attr.fg, Color::Named(1));
        local.feed(b"\x1b[1;1HQ");
        local.feed(&t.render_cell(0, 0));
        assert_eq!(local.screen_line(0), t.screen_line(0));
    }

    #[test]
    fn cursor_sequence_restores_pen_charset_and_pending_wrap() {
        let mut t = Terminal::with_size(5, 3);
        t.feed(b"\x1b)0\x0e\x1b[1;32mabcde");
        assert!(t.pending_wrap());
        let mut local = Terminal::with_size(5, 3);
        local.feed(b"\x1b)0\x0eabcde\x0f\x1b[0m\x1b[3;1H");
        local.feed(&t.cursor_sequence());
        assert!(local.pending_wrap());
        assert_eq!(local.pen(), t.pen());
        local.feed(b"q");
        t.feed(b"q");
        assert_eq!(local.screen_line(1), t.screen_line(1), "same character set after the wrap");
        assert_eq!(t.screen_line(1), "─");
    }

    #[test]
    fn keyboard_modes_and_reset_sequence() {
        let mut t = Terminal::new();
        t.feed(b"\x1b[>1u\x1b[?1h\x1b=\x1b[>4;2m\x1b[?25l\x1b[5 q\x1b[?1049h\x1b[>5u\x1b[?1003;1006;2004;1007h");
        assert_eq!(t.kitty_flags(), 5, "the alternate screen has its own stack");
        let copy = copy_of(&t, 0);
        assert!(!copy.cursor_visible());
        let rendered = String::from_utf8_lossy(&t.render(0)).into_owned();
        for part in ["\x1b[>5u", "\x1b[>4;2m", "\x1b[5 q", "\x1b[?1007h"] {
            assert!(rendered.contains(part), "{part:?} in render");
        }
        let reset = String::from_utf8(t.reset_sequence()).unwrap();
        assert!(reset.contains("\x1b[<99u\x1b[?1049l\x1b[<99u"), "both screens' stacks are emptied");
        for part in ["\x1b[?1003l", "\x1b[?1006l", "\x1b[?2004l", "\x1b[>4m", "\x1b[?25h", "\x1b[?1l", "\x1b[0 q", "\x1b[?1007l"] {
            assert!(reset.contains(part), "{part:?} in {reset:?}");
        }
        let mut local = Terminal::new();
        local.feed(&t.render(0));
        local.feed(b"\x1b[?1049l\x1b[>1u\x1b[?1049h");
        local.feed(&t.reset_sequence());
        assert!(!local.alt_screen());
        assert_eq!(local.kitty_flags(), 0, "the primary screen's stack too");
        assert!(!local.mode().contains(TermMode::MOUSE_MOTION));
        let plain = Terminal::new();
        let reset = String::from_utf8(plain.reset_sequence()).unwrap();
        assert!(!reset.contains("1049l") && !reset.contains('<') && !reset.contains("1007"));
        assert!(!String::from_utf8_lossy(&plain.render(0)).contains("1007h"), "alacritty's alternate scroll default is not passed on");
    }

    #[test]
    fn resize_keeps_cursor_line_and_resets_region() {
        let mut t = Terminal::with_size(20, 6);
        t.feed(b"1\r\n2\r\n3\r\n4\r\n5\r\nprompt$ \x1b[2;4r\x1b[6;9H");
        t.resize(10, 3);
        assert_eq!(t.size(), (10, 3));
        assert_eq!(t.screen_line(2), "prompt$");
        assert_eq!(t.cursor().0, 2);
        assert_eq!(t.scroll_region(), (0, 3));
        assert_eq!(t.screen_line(0), "4", "the lines above go to history");
        assert_eq!(t.scrollback_line(t.scrollback_len() - 1), "3");
    }

    #[test]
    fn tabs_repaint_as_spaces() {
        let mut t = Terminal::with_size(30, 2);
        t.feed(b"a\tb\tc");
        assert_eq!(t.screen_line(0), "a       b       c");
        let mut local = Terminal::with_size(30, 2);
        local.feed(b"a\tb\tc\x1b[1;1HZZZZZZZZZZZZZZZZZZZZ");
        local.feed(&t.render_span(0, 0, 20));
        assert_eq!(local.screen_line(0), t.screen_line(0));
        copy_of(&t, 0);
        assert!(t.blank_from(0, 17));
    }

    #[test]
    fn hostile_output_is_bounded() {
        let mut t = Terminal::with_size(20, 3);
        t.feed(&b"\x1b[>1u".repeat(5000));
        assert_eq!(t.extra.kitty_depth[0], KITTY_DEPTH);
        let mut huge = b"\x1b]2;".to_vec();
        huge.extend(std::iter::repeat_n(b'x', MAX_SEQUENCE + 10));
        huge.extend_from_slice(b"\x07after");
        let cut = t.feed_to_ground(&huge);
        assert_eq!(cut, huge.len());
        assert!(t.title().len() <= MAX_SEQUENCE);
        assert_eq!(t.screen_line(0), "after", "the rest of the long string is skipped");
        // One column: alacritty would panic or hang on wide characters.
        let mut narrow = Terminal::with_size(1, 2);
        narrow.feed("\r\n한 hello 한글".as_bytes());
        narrow.resize(1, 3);
        narrow.resize(5, 3);
        assert_eq!(narrow.size(), (5, 3));
    }

    #[test]
    fn sequences_and_strings_end_where_they_should() {
        let mut t = Terminal::with_size(20, 3);
        assert_eq!(t.feed_to_ground(b"ab\x1b[3\r"), 2, "a C0 control inside a CSI does not end it");
        assert_eq!(t.feed_to_ground(b"1mX"), 3);
        assert_eq!(t.cell(0, 0).unwrap().ch, 'X');
        let mut k = Terminal::with_size(20, 2);
        k.feed(b"\x1bkabc\x18visible");
        assert_eq!(k.screen_line(0), "visible", "CAN cancels a title string");
        let mut kk = Terminal::with_size(20, 2);
        kk.feed(b"x\x1b\x1bkhidden\x1b\\y");
        assert_eq!(kk.screen_line(0), "xy");
    }

    #[test]
    fn soft_reset_column_mode_origin_moves_and_1048() {
        let mut t = Terminal::with_size(10, 6);
        t.feed(b"\x1b[4h\x1b[?25l\x1b[?6h\x1b[2;4r\x1b[1;31m\x1b[3;3H\x1b[!p");
        assert!(!t.insert_mode() && t.cursor_visible() && !t.origin_mode());
        assert_eq!(t.scroll_region(), (0, 6));
        assert_eq!(t.pen(), Attr::default());
        t.feed(b"\x1b[2;4rkeep\x1b[?3h");
        assert_eq!(t.scroll_region(), (1, 4), "DECCOLM is ignored");
        assert!(t.screen_line(0).contains("keep"));
        let mut o = Terminal::with_size(10, 6);
        o.feed(b"\x1b[2;4r\x1b[?6h\x1b[2;1H\x1b[5A");
        assert_eq!(o.cursor().0, 1, "CUU stops at the top margin");
        o.feed(b"\x1b[9B");
        assert_eq!(o.cursor().0, 3, "CUD stops at the bottom margin");
        let mut s = Terminal::with_size(10, 6);
        s.feed(b"\x1b[2;3H\x1b[?1048h\x1b[5;5H\x1b[?1048l");
        assert_eq!(s.cursor(), (1, 2));
    }

    #[test]
    fn render_keeps_kitty_flags_set_in_the_alternate_screen() {
        let mut t = Terminal::new();
        t.feed(b"\x1b[>1u\x1b[?1049h\x1b[>1u\x1b[=15;1u");
        assert_eq!(t.kitty_flags(), 15);
        let rendered = String::from_utf8_lossy(&t.render(0)).into_owned();
        assert_eq!(t.kitty_flags(), 15);
        assert!(rendered.contains("\x1b[>15u"));
    }

    #[test]
    fn synchronized_output_applies_at_once() {
        let mut t = Terminal::with_size(10, 2);
        t.feed(b"\x1b[?2026hinside");
        assert!(t.synchronized());
        assert_eq!(t.screen_line(0), "inside");
        t.feed(b"\x1b[?2026l");
        assert!(!t.synchronized());
        assert_eq!(t.screen_line(0), "inside");
    }

    #[test]
    fn diff_turns_the_old_screen_into_the_new() {
        // Each case: bytes for the screen a client has, then more output.
        let frame = |n: usize| {
            let mut out = format!("\x1b[H\x1b[1;31mFRAME {n}\x1b[0m");
            for row in 1..9 {
                let ch = (b'a' + ((n + row) % 26) as u8) as char;
                out.push_str(&format!("\r\n{}", ch.to_string().repeat(30)));
            }
            out
        };
        let cases: Vec<(String, String)> = vec![
            (frame(1), frame(2)),
            (frame(5), frame(3)),
            ("$ ls\r\none two\r\n$ ".into(), "x\r\n\x1b]2;title\x07한글 wide\r\n\x1b[7mrev\x1b[0m $ ".into()),
            ("a\r\nb\r\nc".into(), "\x1b[2;5r\x1b[5H\n\n\x1b[3;1Hmid".into()),
            ((1..30).map(|n| format!("line {n}\r\n")).collect(), (30..34).map(|n| format!("line {n}\r\n")).collect()),
        ];
        for (before, after) in cases {
            let make = || {
                let mut t = Terminal::with_history(40, 10, 0);
                t.feed(before.as_bytes());
                t
            };
            let (mut client, mut now) = (make(), make());
            now.feed(after.as_bytes());
            let diff = now.diff_from(&client).expect("a diff");
            client.feed(&diff);
            assert!(client.same_screen(&now), "{before:?} then {after:?}:\n{}\nvs\n{}", (0..10).map(|r| client.screen_line(r)).collect::<Vec<_>>().join("|"), (0..10).map(|r| now.screen_line(r)).collect::<Vec<_>>().join("|"));
        }
        // A scrolled block costs a scroll and the new rows, not a repaint.
        let (mut a, mut b) = (Terminal::with_history(40, 10, 0), Terminal::with_history(40, 10, 0));
        a.feed(frame(1).as_bytes());
        b.feed(frame(2).as_bytes());
        let diff = b.diff_from(&a).unwrap();
        assert!(diff.len() < 300, "{} bytes", diff.len());
        // A switch to the alternate screen is not bridged.
        b.feed(b"\x1b[?1049h");
        assert!(b.diff_from(&a).is_none());
    }
}
