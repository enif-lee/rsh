//! The client's settings, `~/.rsh/config.yaml`: flat `key: value` lines
//! (a subset of YAML), `#` comments. A missing file means the defaults;
//! an unknown key or a bad value is an error, so a setting meant to turn
//! something off is never silently ignored.
//!
//! ```yaml
//! # Programs on the host may read the local clipboard (OSC 52 `?`).
//! clipboard_read: true
//! # Compress output: on any link (always), on slow or far ones (adaptive),
//! # or not at all (none).
//! compress: always
//! show_stats: false
//! inspect: false
//! ```

use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// OSC 52 clipboard reads reach the local terminal, which answers
    /// with the clipboard to the host.
    pub clipboard_read: bool,
    pub compress: Compress,
    /// Show frame rate, round-trip latency and packet traffic in the header.
    pub show_stats: bool,
    /// Highlight moved, cell-patched, rewritten and unchanged rows in the viewport.
    pub inspect: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compress {
    /// When the link is slow, far or behind (the server decides).
    Adaptive,
    Always,
    /// Not at all (the setting's `none`).
    Off,
}

impl Compress {
    pub fn name(self) -> &'static str {
        match self { Self::Adaptive => "adaptive", Self::Always => "always", Self::Off => "none" }
    }

    /// From the setting's (and `--compress`'s) name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "adaptive" => Some(Compress::Adaptive),
            "always" => Some(Compress::Always),
            "none" => Some(Compress::Off),
            _ => None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config { clipboard_read: true, compress: Compress::Always, show_stats: false, inspect: false }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text).map_err(|err| format!("{}: {err}", path.display())),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(format!("{}: {err}", path.display())),
        }
    }

    /// Change one known setting, keeping comments and unrelated values.
    pub fn save_setting(path: &Path, key: &str, value: &str) -> Result<(), String> {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::fd::AsRawFd;
        if let Some(parent) = path.parent() { crate::net::private_dir(parent).map_err(|err| err.to_string())?; }
        let lock = std::fs::OpenOptions::new().write(true).create(true).truncate(false).mode(0o600).open(path.with_extension("lock")).map_err(|err| err.to_string())?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 { return Err(std::io::Error::last_os_error().to_string()); }
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(err) => return Err(err.to_string()),
        };
        Self::parse(&text)?;
        let mut found = false;
        let mut lines: Vec<String> = text.lines().map(|line| {
            if line.split_once(':').is_some_and(|(name, _)| name.trim().trim_start_matches('\u{feff}') == key) {
                found = true;
                let comment = line.find(" #").map(|at| &line[at..]).unwrap_or("");
                format!("{key}: {value}{comment}")
            } else { line.to_string() }
        }).collect();
        if !found { lines.push(format!("{key}: {value}")); }
        let text = lines.join("\n") + "\n";
        Self::parse(&text)?;
        let save = || -> std::io::Result<()> {
            if let Some(parent) = path.parent() { crate::net::private_dir(parent)?; }
            let tmp = path.with_extension(format!("yaml.{}-{}.tmp", std::process::id(), crate::server::now_ms()));
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&tmp)?;
            file.write_all(text.as_bytes())?;
            std::fs::rename(tmp, path)
        };
        save().map_err(|err| format!("{}: {err}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let mut config = Self::default();
        let mut seen = Vec::new();
        for (number, line) in text.strip_prefix('\u{feff}').unwrap_or(text).lines().enumerate() {
            let line = match line.find('#') {
                Some(at) if at == 0 || line[..at].ends_with(char::is_whitespace) => &line[..at],
                _ => line,
            };
            if line.trim().is_empty() {
                continue;
            }
            let at = |what: String| format!("line {}: {what}", number + 1);
            let (key, value) = line.split_once(':').ok_or_else(|| at("expected `key: value`".to_string()))?;
            let value = value.trim();
            let flag = || match value {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(at(format!("{} must be true or false, not `{value}`", key.trim()))),
            };
            let key = key.trim();
            if seen.contains(&key) {
                return Err(at(format!("{key} is set twice")));
            }
            seen.push(key);
            match key {
                "clipboard_read" => config.clipboard_read = flag()?,
                "show_stats" => config.show_stats = flag()?,
                "inspect" => config.inspect = flag()?,
                "compress" => {
                    config.compress = Compress::from_name(value).ok_or_else(|| at(format!("compress must be adaptive, always or none, not `{value}`")))?;
                }
                other => return Err(at(format!("unknown setting `{other}`"))),
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_preserves_comments_and_other_settings() {
        let dir = std::env::temp_dir().join(format!("rsh-config-save-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.yaml");
        std::fs::write(&path, "\u{feff}compress: none # keep\n# clipboard policy\nclipboard_read: false\n").unwrap();
        Config::save_setting(&path, "compress", "adaptive").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "compress: adaptive # keep\n# clipboard policy\nclipboard_read: false\n");
        let before = std::fs::read(&path).unwrap();
        assert!(Config::save_setting(&path, "compress", "bad").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn settings_parse_and_mistakes_are_errors() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
        assert!(Config::default().clipboard_read);
        assert!(!Config::default().show_stats);
        assert!(!Config::default().inspect);
        assert!(Config::parse("show_stats: true").unwrap().show_stats);
        assert!(Config::parse("show_stats: yes").is_err());
        assert!(Config::parse("inspect: true").unwrap().inspect);
        assert!(Config::parse("inspect: yes").is_err());
        let off = Config::parse("# mine\n\nclipboard_read: false   # no reads\n").unwrap();
        assert!(!off.clipboard_read);
        assert!(Config::parse("clipboard_read: true").unwrap().clipboard_read);
        assert_eq!(Config::parse("clipboard_read: no").unwrap_err(), "line 1: clipboard_read must be true or false, not `no`");
        assert_eq!(Config::parse("\nclipbaord_read: false").unwrap_err(), "line 2: unknown setting `clipbaord_read`");
        assert!(Config::parse("clipboard_read").is_err());
        assert_eq!(Config::parse("clipboard_read: false\nclipboard_read: true").unwrap_err(), "line 2: clipboard_read is set twice");
        assert!(!Config::parse("\u{feff}clipboard_read: false\r\n").unwrap().clipboard_read);
        assert_eq!(Config::default().compress, Compress::Always);
        assert_eq!(Config::parse("compress: adaptive\nclipboard_read: false").unwrap().compress, Compress::Adaptive);
        assert_eq!(Config::parse("compress: none").unwrap().compress, Compress::Off);
        assert_eq!(Config::parse("compress: yes").unwrap_err(), "line 1: compress must be adaptive, always or none, not `yes`");
        let missing = std::env::temp_dir().join("rsh-no-such-config.yaml");
        assert_eq!(Config::load(&missing).unwrap(), Config::default());
    }
}
