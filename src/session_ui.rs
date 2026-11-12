//! State of local settings and rename windows. Network work stays off the
//! terminal loop so keepalives and remote output continue while editing.
use crate::client::{Target, WindowKey};
use crate::config::{Compress, Config};
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
    pub status: String,
    pub busy: bool,
}

impl Modal {
    pub fn settings(controls: &Controls, clipboard: bool, prediction: Mode, show_stats: bool) -> Self {
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
            WindowKey::Down | WindowKey::Char('j') => self.selected = (self.selected + 1).min(6),
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
                                self.status = "Saved for new attachments".into();
                            }
                            Err(err) => self.status = err,
                        }
                    }
                    5 => {
                        self.show_stats = !self.show_stats;
                        return Effect::ShowStats(self.show_stats);
                    }
                    6 if key == WindowKey::Enter => {
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
        let mut window = Modal::settings(&controls, true, Mode::Adaptive, false);
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
            }
        );
        let _ = std::fs::remove_dir_all(controls.config_path.parent().unwrap());
    }

    #[test]
    fn header_stats_can_change_live_or_be_saved_for_new_attachments() {
        let controls = controls();
        let target = Target::Rsh("h".into(), 1);
        let mut window = Modal::settings(&controls, true, Mode::Adaptive, false);
        window.selected = 5;
        assert!(matches!(window.handle(WindowKey::Right, &controls, &target, "old"), Effect::ShowStats(true)));
        assert!(!controls.config_path.exists());
        window.selected = 2;
        window.handle(WindowKey::Right, &controls, &target, "old");
        window.selected = 5;
        assert!(matches!(window.handle(WindowKey::Enter, &controls, &target, "old"), Effect::None));
        assert!(Config::load(&controls.config_path).unwrap().show_stats);
        // A saved global change does not override the current session's choice.
        window.selected = 2;
        window.handle(WindowKey::Left, &controls, &target, "old");
        assert!(window.lines(&controls, "h", "peer", "old", false)[5].ends_with("true"));
        window.selected = 5;
        assert!(matches!(window.handle(WindowKey::Left, &controls, &target, "old"), Effect::ShowStats(false)));
        assert!(Config::load(&controls.config_path).unwrap().show_stats);
        window.handle(WindowKey::Down, &controls, &target, "old");
        assert!(matches!(window.handle(WindowKey::Enter, &controls, &target, "old"), Effect::None));
        assert!(window.lines(&controls, "h", "peer", "old", false)[0].contains("Rename"));
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
