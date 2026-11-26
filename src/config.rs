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
//! # The key before rsh's commands (`~.` detaches): a printable character
//! # (quoted, used at the start of a line), ctrl-<key> (used anywhere) or
//! # none. RSH_ESCAPE_KEY takes precedence.
//! escape_key: "~"
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
    /// The key before rsh's own commands; `None` turns them off.
    pub escape_key: Option<u8>,
}

/// The keys that follow the escape key as commands: as the escape key
/// itself, one of them could never be typed through to the session.
const COMMAND_KEYS: &[u8] = b".h?ripml,";

/// Whether `key` may be the escape key: printable ASCII but space and the
/// command keys, or a control character that no session needs for itself
/// (not NUL, Ctrl-C, Ctrl-D, Backspace, Tab, Enter, Ctrl-Z or Esc).
pub fn check_escape_key(key: u8) -> Result<(), String> {
    match key {
        b'!'..=b'~' if !COMMAND_KEYS.contains(&key) => Ok(()),
        0x01..=0x1f if ![0x03, 0x04, 0x08, 0x09, 0x0a, 0x0d, 0x1a, 0x1b].contains(&key) => Ok(()),
        _ => Err(format!("{} cannot be the escape key", crate::client::escape_name(key))),
    }
}

/// `escape_key`'s value for `key`: `none`, `ctrl-a`, `ctrl-^`, or the
/// character quoted (so `#` is not taken for a comment).
pub fn escape_key_value(key: Option<u8>) -> String {
    match key {
        None => "none".into(),
        Some(key) if key < 0x20 => format!("ctrl-{}", char::from(key + 0x40).to_ascii_lowercase()),
        Some(key) => format!("\"{}\"", char::from(key)),
    }
}

/// `RSH_ESCAPE_KEY` when it sets the escape key: empty turns it off, one
/// ASCII character is the key; any other value leaves `escape_key`.
pub fn escape_key_from_env() -> Option<Option<u8>> {
    match std::env::var("RSH_ESCAPE_KEY") {
        Ok(value) if value.is_empty() => Some(None),
        Ok(value) if value.len() == 1 && value.as_bytes()[0] < 0x80 => Some(Some(value.as_bytes()[0])),
        _ => None,
    }
}

fn parse_escape_key(value: &str) -> Result<Option<u8>, String> {
    if value.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    let unquoted = value
        .strip_prefix('"').and_then(|v| v.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(value);
    let key = match (unquoted.as_bytes(), unquoted.get(..5).map(str::to_ascii_lowercase).as_deref()) {
        ([key], _) => *key,
        (_, Some("ctrl-")) if unquoted.len() == 6 => match unquoted.as_bytes()[5].to_ascii_uppercase() {
            ch @ (b'@'..=b'_') => ch - 0x40,
            _ => return Err(format!("escape_key: no control character for `{value}`")),
        },
        _ => return Err(format!("escape_key must be a character, ctrl-<key> or none, not `{value}`")),
    };
    check_escape_key(key).map(|()| Some(key)).map_err(|err| format!("escape_key: {err}"))
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
        Config { clipboard_read: true, compress: Compress::Always, show_stats: false, inspect: false, escape_key: Some(crate::client::DEFAULT_ESCAPE) }
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
            // A comment starts with a `#` at the start or after a space, so
            // `escape_key: "#" # note` keeps its quoted `#`.
            let line = match line.match_indices('#').find(|(at, _)| *at == 0 || line[..*at].ends_with(char::is_whitespace)) {
                Some((at, _)) => &line[..at],
                None => line,
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
                "escape_key" => config.escape_key = parse_escape_key(value).map_err(at)?,
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
    fn escape_key_reads_characters_control_keys_and_none() {
        assert_eq!(Config::default().escape_key, Some(b'~'));
        for (value, key) in [("\"~\"", Some(b'~')), ("'#'", Some(b'#')), ("\"#\" # note", Some(b'#')), ("`", Some(b'`')), ("ctrl-^", Some(0x1e)), ("Ctrl-]", Some(0x1d)), ("\"ctrl-a\"", Some(0x01)), ("None", None)] {
            assert_eq!(Config::parse(&format!("escape_key: {value}")).unwrap().escape_key, key, "{value}");
            assert_eq!(Config::parse(&format!("escape_key: {}", escape_key_value(key))).unwrap().escape_key, key);
        }
        assert_eq!(escape_key_value(Some(0x1e)), "ctrl-^");
        // A saved key keeps the comment after it.
        let dir = std::env::temp_dir().join(format!("rsh-config-escape-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.yaml");
        std::fs::write(&path, "escape_key: \"~\" # my key\n").unwrap();
        Config::save_setting(&path, "escape_key", &escape_key_value(Some(b'#'))).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "escape_key: \"#\" # my key\n");
        assert_eq!(Config::load(&path).unwrap().escape_key, Some(b'#'));
        let _ = std::fs::remove_dir_all(dir);
        // Keys a session needs, command keys and what is not one key.
        for value in ["ctrl-c", "ctrl-m", "ctrl-[", "\".\"", "l", "\" \"", "ab", "ctrl-", "ctrl-1", "#"] {
            assert!(Config::parse(&format!("escape_key: {value}")).is_err(), "{value}");
        }
    }

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
