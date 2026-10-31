#![cfg(target_os = "linux")]

use rsh::pty::{Pty, SpawnOptions};
use std::ffi::CStr;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

// Own each child until waitpid has reaped it, including error/unwind paths.
struct Child {
    pty: Pty,
    status: Option<i32>,
    reaped: bool,
}
impl Child {
    fn reap(&mut self, flags: i32) -> io::Result<Option<i32>> {
        if self.reaped { return Ok(self.status); }
        loop {
            let mut status = 0;
            let pid = unsafe { libc::waitpid(self.pty.pid(), &mut status, flags) };
            if pid == self.pty.pid() {
                self.reaped = true;
                self.status = Some(status);
                return Ok(self.status);
            }
            if pid == 0 { return Ok(None); }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted { continue; }
            if error.raw_os_error() == Some(libc::ECHILD) { self.reaped = true; }
            return Err(error);
        }
    }
    fn finish(&mut self) -> io::Result<Option<i32>> {
        if !self.reaped { self.pty.kill(); }
        self.reap(0)
    }
}
impl Drop for Child {
    fn drop(&mut self) { let _ = self.finish(); }
}

fn one_spawn(worker: usize, round: usize) -> Result<(), String> {
    let marker = format!("PTY_{}_{}_{}", std::process::id(), worker, round);
    let command = vec![format!("printf '%s\\n' '{marker}'; tty")];
    let pty = Pty::spawn(&SpawnOptions {
        session_id: 900_000 + (worker * 8 + round) as u64,
        cols: 80, rows: 24, env: &[], command: &command,
    }).map_err(|e| format!("spawn {marker}: {e}"))?;
    let mut child = Child { pty, status: None, reaped: false };
    let observed = (|| -> Result<(String, Vec<u8>), String> {
        // This is the kernel's slave for the master we own. The child must
        // independently report that same terminal via its actual fd 0.
        let mut name = [0 as libc::c_char; libc::PATH_MAX as usize];
        let error = unsafe { libc::ptsname_r(child.pty.fd(), name.as_mut_ptr(), name.len()) };
        if error != 0 { return Err(format!("expected slave {marker}: errno {error}")); }
        let expected = unsafe { CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned();
        let until = Instant::now() + Duration::from_secs(5);
        let mut output = Vec::new();
        let mut buf = [0u8; 512];
        loop {
            if Instant::now() >= until { return Err(format!("timeout {marker}")); }
            child.reap(libc::WNOHANG).map_err(|e| format!("wait {marker}: {e}"))?;
            loop {
                match child.pty.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        output.extend_from_slice(&buf[..n]);
                        if output.len() > 4096 { return Err(format!("excess output {marker}")); }
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(format!("read {marker}: {e}")),
                }
            }
            // Reap precedes the drain, so an exited child's final bytes
            // have already been written before the last drain above.
            if child.reaped { break; }
            let wait = until.saturating_duration_since(Instant::now()).as_millis().min(50).max(1) as i32;
            let mut fd = libc::pollfd { fd: child.pty.fd(), events: libc::POLLIN, revents: 0 };
            let rc = unsafe { libc::poll(&mut fd, 1, wait) };
            if rc < 0 {
                let e = io::Error::last_os_error();
                if e.kind() != io::ErrorKind::Interrupted { return Err(format!("poll {marker}: {e}")); }
            }
        }
        Ok((expected, output))
    })();
    // Reap/kill and close every owned child before any final assertion.
    let status = child.finish().map_err(|e| format!("cleanup {marker}: {e}"));
    drop(child);
    let status = status?.ok_or_else(|| format!("no exit status {marker}"))?;
    let (expected, output) = observed?;
    let text = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = text.lines().map(|s| s.trim_end_matches('\r')).collect();
    if !libc::WIFEXITED(status) || libc::WEXITSTATUS(status) != 0
        || lines != [marker.as_str(), expected.as_str()] {
        return Err(format!("{marker}: expected tty {expected}, status {status}, output {text:?}"));
    }
    Ok(())
}

#[test]
fn concurrent_pty_children_keep_their_own_marker_and_slave() {
    let mut workers = Vec::new();
    let mut failures = Vec::new();
    // No test sleep/hooks or global serialization around the production call.
    for worker in 0..8 {
        match thread::Builder::new().spawn(move || {
            let mut failures = Vec::new();
            for round in 0..8 {
                if let Err(error) = one_spawn(worker, round) { failures.push(error); }
            }
            failures
        }) {
            Ok(handle) => workers.push(handle),
            Err(error) => failures.push(format!("worker {worker}: {error}")),
        }
    }
    for worker in workers {
        match worker.join() {
            Ok(errors) => failures.extend(errors),
            Err(_) => failures.push("worker panicked (owned child cleanup runs on unwind)".into()),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
