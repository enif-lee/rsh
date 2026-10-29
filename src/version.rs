// What this binary is, and whether a remote one must be replaced. The
// client's build is the reference: a server reporting any other build is
// brought to the client's, newer or older.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Hash of the sources this binary was built from (see build.rs), kept in
/// the binary as one string so a file can be checked without running it.
const MARK: &str = concat!("rsh-build-mark:", env!("RSH_BUILD"), ":end");

pub fn build() -> &'static str {
    &MARK["rsh-build-mark:".len()..MARK.len() - ":end".len()]
}
/// Wire format of packets; peers with another number cannot talk at all.
pub const PROTOCOL: u32 = 3;

/// `rsh --version`: `rsh 0.1.0 build 1a2b3c4d5e6f protocol 3 handoff 3 platform linux-x86_64`.
pub fn describe() -> String {
    format!(
        "rsh {VERSION} build {} protocol {PROTOCOL} handoff {} platform {}",
        build(),
        crate::server::HANDOFF_FORMAT,
        local_platform()
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remote {
    pub version: String,
    pub build: Option<String>,
    pub protocol: Option<u32>,
    pub handoff: Option<u32>,
    pub platform: Option<String>,
}

/// Reads a `describe()` line; older binaries print less and still parse.
pub fn parse(line: &str) -> Option<Remote> {
    let mut words = line.split_whitespace();
    if words.next()? != "rsh" {
        return None;
    }
    let mut remote = Remote {
        version: words.next()?.to_string(),
        build: None,
        protocol: None,
        handoff: None,
        platform: None,
    };
    while let Some(key) = words.next() {
        let value = words.next();
        match key {
            "build" => remote.build = value.map(str::to_string),
            "protocol" => remote.protocol = value.and_then(|v| v.parse().ok()),
            "handoff" => remote.handoff = value.and_then(|v| v.parse().ok()),
            "platform" => remote.platform = value.map(str::to_string),
            _ => {}
        }
    }
    Some(remote)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    /// The remote runs exactly this build.
    Nothing,
    /// Another build that speaks this protocol: replace it, keeping sessions
    /// when the server can hand them over.
    Update,
    /// Missing, unreadable, or another protocol: install this build whatever
    /// is there.
    Force,
}

pub fn need(remote: Option<&Remote>) -> Need {
    let Some(remote) = remote else {
        return Need::Force;
    };
    if remote.protocol != Some(PROTOCOL) {
        return Need::Force;
    }
    if remote.version == VERSION && remote.build.as_deref() == Some(build()) {
        Need::Nothing
    } else {
        Need::Update
    }
}

/// `linux-x86_64`, `darwin-aarch64`: the name a binary for a platform has.
pub fn platform(os: &str, arch: &str) -> String {
    let os = match os.to_ascii_lowercase().as_str() {
        "macos" | "darwin" => "darwin".to_string(),
        other => other.to_string(),
    };
    let arch = match arch {
        "amd64" | "x86_64" => "x86_64",
        "arm64" | "aarch64" => "aarch64",
        other => other,
    };
    format!("{os}-{arch}")
}

pub fn local_platform() -> String {
    platform(std::env::consts::OS, std::env::consts::ARCH)
}

/// True if the file at `bytes` was built from these sources: the build id
/// is embedded as text, so this works for binaries of any platform.
pub fn is_this_build(bytes: &[u8]) -> bool {
    bytes.windows(MARK.len()).any(|window| window == MARK.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_parse_and_need() {
        let me = parse(&describe()).unwrap();
        assert_eq!(me.build.as_deref(), Some(build()));
        assert_eq!(me.platform.as_deref(), Some(local_platform().as_str()));
        assert_eq!(need(Some(&me)), Need::Nothing);
        assert_eq!(need(None), Need::Force);
        assert_eq!(need(parse("rsh 0.1.0").as_ref()), Need::Force, "no protocol: an old build");
        let mut other = me.clone();
        other.build = Some("000000000000".into());
        assert_eq!(need(Some(&other)), Need::Update);
        other.version = "9.9.9".into();
        assert_eq!(need(Some(&other)), Need::Update, "the client's build wins, newer or older");
        other.protocol = Some(PROTOCOL + 1);
        assert_eq!(need(Some(&other)), Need::Force);
        assert!(parse("usage: rsh").is_none());
        assert_eq!(platform("Linux", "x86_64"), "linux-x86_64");
        assert_eq!(platform("Darwin", "arm64"), "darwin-aarch64");
        assert_eq!(platform("macos", "aarch64"), "darwin-aarch64");
        let exe = std::fs::read(std::env::current_exe().unwrap()).unwrap();
        assert!(is_this_build(&exe), "a binary built from these sources carries the id");
        assert!(!is_this_build(b"rsh-build-mark:000000000000:end"));
    }
}
