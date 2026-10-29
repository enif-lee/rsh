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
//! ```

use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// OSC 52 clipboard reads reach the local terminal, which answers
    /// with the clipboard to the host.
    pub clipboard_read: bool,
    pub compress: Compress,
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
        Config { clipboard_read: true, compress: Compress::Always }
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
    fn settings_parse_and_mistakes_are_errors() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
        assert!(Config::default().clipboard_read);
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
