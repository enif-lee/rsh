//! State of local settings and rename windows. Network work stays off the
//! terminal loop so keepalives and remote output continue while editing.
use crate::client::{escape_name, Target, WindowKey};
use crate::config::{check_escape_key, escape_key_value, Compress, Config};
use crate::predict::Mode;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type Renamer = Arc<dyn Fn(&Target, &str) -> Result<String, String> + Send + Sync>;

#[derive(Clone)]
pub struct Controls {
    pub rename: Renamer,
    pub config_path: PathBuf,
    /// Updated on successful rename, also used by detach/resume messages.
    pub label: Arc<Mutex<String>>,
}

pub enum Effect {
    None,
    Close,
    Rename(Target, String),
    Clipboard(bool),
    Prediction(Mode),
    ShowStats(bool),
    Inspect(bool),
    /// The new escape key, saved, now also this attachment's.
    EscapeKey(u8),
}

enum Page {
    Settings,
    Rename { target: Target, value: String },
}

pub struct Modal {
    page: Page,
    pub selected: usize,
    global: bool,
    config: Config,
    clipboard: bool,
    prediction: Mode,
    show_stats: bool,
    inspect: Option<bool>,
    /// This attachment's escape key; a captured one replaces it unless
    /// `RSH_ESCAPE_KEY` sets it.
    escape: Option<u8>,
    /// `RSH_ESCAPE_KEY` sets the key of every attachment of this process.
    escape_from_env: bool,
    /// The next key typed becomes the saved escape key.
    capturing: bool,
    pub status: String,
    pub busy: bool,
}

/// How the settings show an escape key.
fn key_name(key: Option<u8>) -> String {
    key.map_or_else(|| "off".into(), escape_name)
}

/// Rows of the settings page that can be selected.
const PREFIX_ROW: usize = 7;
const RENAME_ROW: usize = 8;

impl Modal {
    pub fn settings(
        controls: &Controls,
        clipboard: bool,
        prediction: Mode,
        show_stats: bool,
        inspect: Option<bool>,
        escape: Option<u8>,
    ) -> Self {
        let loaded = Config::load(&controls.config_path);
        let status = loaded.as_ref().err().cloned().unwrap_or_default();
        Self {
            page: Page::Settings,
            selected: 2,
            global: false,
            config: loaded.unwrap_or_default(),
            clipboard,
            prediction,
            show_stats,
            inspect,
            escape,
            escape_from_env: crate::config::escape_key_from_env().is_some(),
            capturing: false,
            status,
            busy: false,
        }
    }

    pub fn rename(target: Target, name: String) -> Self {
        Self {
            page: Page::Rename {
                target,
                value: name,
            },
            selected: 2,
            global: false,
            config: Config::default(),
            clipboard: false,
            prediction: Mode::Adaptive,
            show_stats: false,
            inspect: None,
            escape: None,
            escape_from_env: false,
            capturing: false,
            status: String::new(),
            busy: false,
        }
    }

    pub(crate) fn retarget(&mut self, old: &Target, updated: &Target) {
        if let Page::Rename { target, .. } = &mut self.page {
            if target == old {
                *target = updated.clone();
            }
        }
    }

    pub fn lines(
        &self,
        controls: &Controls,
        host: &str,
        peer: &str,
        label: &str,
        tmux: bool,
    ) -> Vec<String> {
        let clean = crate::cli::printable;
        match &self.page {
            Page::Rename { value, .. } => vec![
                "Rename session".into(),
                String::new(),
                format!("Name: {}_", clean(value)),
                clean(&self.status),
                "Type a name; Enter saves; Esc cancels".into(),
            ],
            Page::Settings => vec![
                format!("rsh settings — {}", env!("CARGO_PKG_VERSION")),
                String::new(),
                format!(
                    "Scope: {}",
                    if self.global {
                        "Global (saved defaults)"
                    } else {
                        "Session (this connection)"
                    }
                ),
                format!(
                    "Clipboard read: {}",
                    if self.global {
                        self.config.clipboard_read
                    } else {
                        self.clipboard
                    }
                ),
                if self.global {
                    format!(
                        "Compression: {} (new attachments)",
                        self.config.compress.name()
                    )
                } else {
                    format!("Prediction: {}", mode_name(self.prediction))
                },
                format!("Header stats (fps, RTT, KB/s): {}", if self.global { self.config.show_stats } else { self.show_stats }),
                if self.global {
                    format!("Inspect (frame diff): {}", self.config.inspect)
                } else if let Some(inspect) = self.inspect {
                    format!("Inspect (frame diff): {inspect}")
                } else {
                    "Inspect (frame diff): unavailable in tmux".into()
                },
                if self.capturing {
                    "Prefix key: press the new key (Esc cancels)".into()
                } else if self.escape == self.config.escape_key {
                    format!("Prefix key: {} (saved; Enter changes)", key_name(self.config.escape_key))
                } else {
                    format!("Prefix key: {} (saved; this session {})", key_name(self.config.escape_key), key_name(self.escape))
                },
                "Rename current session (Enter)".into(),
                String::new(),
                format!(
                    "{}:{}{}",
                    clean(host),
                    clean(label),
                    if tmux { " (tmux)" } else { "" }
                ),
                format!("Peer: {peer}"),
                format!(
                    "Config: {}",
                    clean(&controls.config_path.display().to_string())
                ),
                clean(&self.status),
                "↑/↓ move; ←/→ or Enter change; Esc closes".into(),
            ],
        }
    }

    pub(crate) fn handle(
        &mut self,
        key: WindowKey,
        controls: &Controls,
        here: &Target,
        label: &str,
    ) -> Effect {
        if self.capturing {
            return self.capture(key, controls);
        }
        if key == WindowKey::Close {
            return Effect::Close;
        }
        if self.busy {
            return Effect::None;
        }
        if let Page::Rename { target, value } = &mut self.page {
            return match key {
                WindowKey::Char(ch) if !ch.is_control() && value.chars().count() < 64 => {
                    value.push(ch);
                    Effect::None
                }
                WindowKey::Backspace => {
                    value.pop();
                    Effect::None
                }
                WindowKey::Enter if !value.is_empty() => {
                    self.busy = true;
                    self.status = "Renaming...".into();
                    Effect::Rename(target.clone(), value.clone())
                }
                _ => Effect::None,
            };
        }
        match key {
            WindowKey::Char('q') => return Effect::Close,
            WindowKey::Up | WindowKey::Char('k') => {
                self.selected = self.selected.saturating_sub(1).max(2)
            }
            WindowKey::Down | WindowKey::Char('j') => self.selected = (self.selected + 1).min(RENAME_ROW),
            WindowKey::Left | WindowKey::Right | WindowKey::Enter => {
                let backwards = key == WindowKey::Left;
                match self.selected {
                    2 => self.global = !self.global,
                    3 if self.global => {
                        let value = !self.config.clipboard_read;
                        match Config::save_setting(
                            &controls.config_path,
                            "clipboard_read",
                            if value { "true" } else { "false" },
                        ) {
                            Ok(()) => {
                                self.config.clipboard_read = value;
                                self.status = "Saved for new attachments".into();
                            }
                            Err(err) => self.status = err,
                        }
                    }
                    3 => {
                        self.clipboard = !self.clipboard;
                        return Effect::Clipboard(self.clipboard);
                    }
                    4 if self.global => {
                        let choices = [Compress::Always, Compress::Adaptive, Compress::Off];
                        let at = choices
                            .iter()
                            .position(|v| *v == self.config.compress)
                            .unwrap_or(0);
                        let value = choices[(at + if backwards { 2 } else { 1 }) % 3];
                        match Config::save_setting(&controls.config_path, "compress", value.name())
                        {
                            Ok(()) => {
                                self.config.compress = value;
                                self.status =
                                    "Saved for new attachments (--compress takes precedence)"
                                        .into();
                            }
                            Err(err) => self.status = err,
                        }
                    }
                    4 => {
                        let choices = [Mode::Adaptive, Mode::Always, Mode::Never];
                        let at = choices
                            .iter()
                            .position(|v| *v == self.prediction)
                            .unwrap_or(0);
                        self.prediction = choices[(at + if backwards { 2 } else { 1 }) % 3];
                        return Effect::Prediction(self.prediction);
                    }
                    5 if self.global => {
                        let value = !self.config.show_stats;
                        match Config::save_setting(&controls.config_path, "show_stats", if value { "true" } else { "false" }) {
                            Ok(()) => {
                                self.config.show_stats = value;
                                self.status = "Saved for new attachments (--inspect takes precedence)".into();
                            }
                            Err(err) => self.status = err,
                        }
                    }
                    5 => {
                        self.show_stats = !self.show_stats;
                        return Effect::ShowStats(self.show_stats);
                    }
                    6 if self.global => {
                        let value = !self.config.inspect;
                        match Config::save_setting(&controls.config_path, "inspect", if value { "true" } else { "false" }) {
                            Ok(()) => {
                                self.config.inspect = value;
                                self.status = "Saved for new attachments (--inspect takes precedence)".into();
                            }
                            Err(err) => self.status = err,
                        }
                    }
                    6 => {
                        if let Some(inspect) = &mut self.inspect {
                            *inspect = !*inspect;
                            return Effect::Inspect(*inspect);
                        }
                        self.status = "Inspect mode is available in ordinary rsh sessions".into();
                    }
                    PREFIX_ROW => {
                        // Saved and applied at once in either scope.
                        self.capturing = true;
                        self.status.clear();
                    }
                    RENAME_ROW if key == WindowKey::Enter => {
                        self.page = Page::Rename {
                            target: here.clone(),
                            value: label.into(),
                        };
                        self.selected = 2;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Effect::None
    }

    /// The next key typed becomes the saved escape key.
    pub fn capturing(&self) -> bool {
        self.capturing
    }

    /// The key typed while waiting for a new escape key.
    /// It is saved and, unless `RSH_ESCAPE_KEY` sets this attachment's
    /// key, used from the next key on.
    fn capture(&mut self, key: WindowKey, controls: &Controls) -> Effect {
        let byte = match key {
            WindowKey::Close => {
                self.capturing = false;
                self.status = "Prefix key unchanged".into();
                return Effect::None;
            }
            WindowKey::Char(ch) if ch.is_ascii() => ch as u8,
            _ => {
                self.status = "That key cannot be the prefix key; press another or Esc".into();
                return Effect::None;
            }
        };
        if let Err(err) = check_escape_key(byte) {
            self.status = format!("{err}; press another or Esc");
            return Effect::None;
        }
        self.capturing = false;
        if let Err(err) = Config::save_setting(&controls.config_path, "escape_key", &escape_key_value(Some(byte))) {
            self.status = err;
            return Effect::None;
        }
        self.config.escape_key = Some(byte);
        if self.escape_from_env {
            self.status = "Saved; RSH_ESCAPE_KEY sets this rsh's key".into();
            return Effect::None;
        }
        self.escape = Some(byte);
        let at = if byte < 0x20 { "" } else { " at the start of a line" };
        self.status = format!("Saved; {}. now detaches{at}", escape_name(byte));
        Effect::EscapeKey(byte)
    }
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Adaptive => "adaptive",
        Mode::Always => "always",
        Mode::Never => "never",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn controls() -> Controls {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        Controls {
            rename: Arc::new(|_, name| Ok(name.into())),
            config_path: std::env::temp_dir()
                .join(format!(
                    "rsh-settings-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                ))
                .join("config.yaml"),
            label: Arc::new(Mutex::new("old".into())),
        }
    }

    #[test]
    fn settings_distinguish_live_session_changes_and_saved_defaults() {
        let controls = controls();
        let target = Target::Rsh("h".into(), 1);
        let mut window = Modal::settings(&controls, true, Mode::Adaptive, false, Some(false), Some(b'~'));
        window.handle(WindowKey::Down, &controls, &target, "old");
        assert!(matches!(
            window.handle(WindowKey::Right, &controls, &target, "old"),
            Effect::Clipboard(false)
        ));
        assert!(!controls.config_path.exists());
        window.handle(WindowKey::Down, &controls, &target, "old");
        assert!(matches!(
            window.handle(WindowKey::Left, &controls, &target, "old"),
            Effect::Prediction(Mode::Never)
        ));
        window.handle(WindowKey::Up, &controls, &target, "old");
        window.handle(WindowKey::Up, &controls, &target, "old");
        window.handle(WindowKey::Right, &controls, &target, "old");
        window.handle(WindowKey::Down, &controls, &target, "old");
        window.handle(WindowKey::Enter, &controls, &target, "old");
        window.handle(WindowKey::Down, &controls, &target, "old");
        window.handle(WindowKey::Right, &controls, &target, "old");
        let saved = Config::load(&controls.config_path).unwrap();
        assert_eq!(
            saved,
            Config {
                clipboard_read: false,
                compress: Compress::Adaptive,
                show_stats: false,
                inspect: false,
                escape_key: Some(b'~'),
            }
        );
        let _ = std::fs::remove_dir_all(controls.config_path.parent().unwrap());
    }

    #[test]
    fn header_stats_and_inspect_can_change_live_or_be_saved_for_new_attachments() {
        let controls = controls();
        let target = Target::Rsh("h".into(), 1);
        let mut window = Modal::settings(&controls, true, Mode::Adaptive, false, Some(false), Some(b'~'));
        window.selected = 5;
        assert!(matches!(window.handle(WindowKey::Right, &controls, &target, "old"), Effect::ShowStats(true)));
        window.selected = 6;
        assert!(matches!(window.handle(WindowKey::Right, &controls, &target, "old"), Effect::Inspect(true)));
        assert!(!controls.config_path.exists());
        window.selected = 2;
        window.handle(WindowKey::Right, &controls, &target, "old");
        window.selected = 5;
        assert!(matches!(window.handle(WindowKey::Enter, &controls, &target, "old"), Effect::None));
        window.selected = 6;
        assert!(matches!(window.handle(WindowKey::Enter, &controls, &target, "old"), Effect::None));
        let saved = Config::load(&controls.config_path).unwrap();
        assert!(saved.show_stats && saved.inspect);
        // A saved global change does not override the current session's choice.
        window.selected = 2;
        window.handle(WindowKey::Left, &controls, &target, "old");
        assert!(window.lines(&controls, "h", "peer", "old", false)[5].ends_with("true"));
        assert!(window.lines(&controls, "h", "peer", "old", false)[6].ends_with("true"));
        window.selected = 5;
        assert!(matches!(window.handle(WindowKey::Left, &controls, &target, "old"), Effect::ShowStats(false)));
        window.selected = 6;
        assert!(matches!(window.handle(WindowKey::Left, &controls, &target, "old"), Effect::Inspect(false)));
        let saved = Config::load(&controls.config_path).unwrap();
        assert!(saved.show_stats && saved.inspect);
        window.selected = RENAME_ROW - 1;
        window.handle(WindowKey::Down, &controls, &target, "old");
        assert!(matches!(window.handle(WindowKey::Enter, &controls, &target, "old"), Effect::None));
        assert!(window.lines(&controls, "h", "peer", "old", false)[0].contains("Rename"));

        let mut tmux_window = Modal::settings(&controls, true, Mode::Adaptive, false, None, Some(b'~'));
        assert!(tmux_window.lines(&controls, "h", "peer", "old", true)[6].contains("unavailable in tmux"));
        tmux_window.selected = 6;
        assert!(matches!(tmux_window.handle(WindowKey::Enter, &controls, &target, "old"), Effect::None));
        assert!(tmux_window.status.contains("ordinary rsh sessions"));
        let _ = std::fs::remove_dir_all(controls.config_path.parent().unwrap());
    }

    #[test]
    fn prefix_key_is_captured_checked_saved_and_applied_at_once() {
        let controls = controls();
        let target = Target::Rsh("h".into(), 1);
        let mut window = Modal::settings(&controls, true, Mode::Adaptive, false, Some(false), Some(b'~'));
        window.escape_from_env = false;
        let line = |window: &Modal| window.lines(&controls, "h", "peer", "old", false)[PREFIX_ROW].clone();
        assert_eq!(line(&window), "Prefix key: ~ (saved; Enter changes)");
        window.selected = PREFIX_ROW;
        window.handle(WindowKey::Enter, &controls, &target, "old");
        assert!(line(&window).contains("press the new key"));
        // Keys a session needs and command keys are refused; `q` and Enter
        // do not leave the capture either.
        for key in [WindowKey::Char('\x04'), WindowKey::Char('.'), WindowKey::Char(' '), WindowKey::Enter, WindowKey::Char('한')] {
            assert!(matches!(window.handle(key, &controls, &target, "old"), Effect::None));
            assert!(line(&window).contains("press the new key"), "{key:?}");
        }
        assert!(!controls.config_path.exists());
        assert!(matches!(window.handle(WindowKey::Char('\x1e'), &controls, &target, "old"), Effect::EscapeKey(0x1e)));
        assert_eq!(Config::load(&controls.config_path).unwrap().escape_key, Some(0x1e));
        assert_eq!(std::fs::read_to_string(&controls.config_path).unwrap(), "escape_key: ctrl-^\n");
        assert_eq!(line(&window), "Prefix key: Ctrl-^ (saved; Enter changes)");
        assert_eq!(window.status, "Saved; Ctrl-^. now detaches");
        // Esc leaves the capture, not the window.
        window.handle(WindowKey::Right, &controls, &target, "old");
        assert!(matches!(window.handle(WindowKey::Close, &controls, &target, "old"), Effect::None));
        assert_eq!(window.status, "Prefix key unchanged");
        assert!(matches!(window.handle(WindowKey::Close, &controls, &target, "old"), Effect::Close));
        window.handle(WindowKey::Enter, &controls, &target, "old");
        assert!(matches!(window.handle(WindowKey::Char('`'), &controls, &target, "old"), Effect::EscapeKey(b'`')));
        assert_eq!(std::fs::read_to_string(&controls.config_path).unwrap(), "escape_key: \"`\"\n");
        assert_eq!(window.status, "Saved; `. now detaches at the start of a line");
        // RSH_ESCAPE_KEY keeps this process's key; the choice is still saved.
        window.escape_from_env = true;
        window.handle(WindowKey::Enter, &controls, &target, "old");
        assert!(matches!(window.handle(WindowKey::Char('!'), &controls, &target, "old"), Effect::None));
        assert_eq!(Config::load(&controls.config_path).unwrap().escape_key, Some(b'!'));
        assert_eq!(line(&window), "Prefix key: ! (saved; this session `)");
        assert_eq!(window.status, "Saved; RSH_ESCAPE_KEY sets this rsh's key");
        let _ = std::fs::remove_dir_all(controls.config_path.parent().unwrap());
    }

    #[test]
    fn rename_accepts_unicode_and_navigation_letters_as_text() {
        let controls = controls();
        let target = Target::Tmux("h".into(), "old".into());
        let mut window = Modal::rename(target.clone(), String::new());
        for ch in "한글 qjk".chars() {
            window.handle(WindowKey::Char(ch), &controls, &target, "old");
        }
        window.handle(WindowKey::Backspace, &controls, &target, "old");
        assert!(
            matches!(window.handle(WindowKey::Enter, &controls, &target, "old"), Effect::Rename(t, value) if t == target && value == "한글 qj")
        );
        assert!(matches!(
            window.handle(WindowKey::Char('x'), &controls, &target, "old"),
            Effect::None
        ));
        assert!(matches!(
            window.handle(WindowKey::Close, &controls, &target, "old"),
            Effect::Close
        ));
    }
}
