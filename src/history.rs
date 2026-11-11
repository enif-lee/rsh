//! Local successful attachments, isolated alongside the selected auth store.
use crate::client::Target;
use crate::proto::{Reader, Writer};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct History(BTreeMap<String, u64>);

fn key(target: &Target) -> String {
    match target {
        Target::Rsh(host, id) => format!("{host}\nrsh\n{id}"),
        Target::Tmux(host, name) => format!("{host}\ntmux\n{name}"),
    }
}

impl History {
    pub fn path(auth: &Path) -> PathBuf {
        auth.with_extension("recent")
    }

    pub fn load(path: &Path) -> Self {
        let Ok(bytes) = std::fs::read(path) else {
            return Self::default();
        };
        let mut r = Reader::new(&bytes);
        let mut entries = BTreeMap::new();
        while let Some(key) = r.str() {
            let Some(time) = r.u64() else { break };
            entries.insert(key, time);
        }
        Self(entries)
    }

    pub fn get(&self, target: &Target) -> u64 {
        self.0.get(&key(target)).copied().unwrap_or(0)
    }

    pub fn record(path: &Path, target: &Target) -> io::Result<()> {
        use std::os::unix::fs::OpenOptionsExt;
        if let Some(parent) = path.parent() {
            crate::net::private_dir(parent)?;
        }
        let lock = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(path.with_extension("recent.lock"))?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut history = Self::load(path);
        history.0.insert(key(target), crate::server::now_ms());
        let mut w = Writer::new();
        for (key, time) in history.0 {
            w.str(&key).u64(time);
        }
        let tmp = path.with_extension("recent.tmp");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)?;
        file.write_all(&w.finish())?;
        std::fs::rename(tmp, path)
    }
}
