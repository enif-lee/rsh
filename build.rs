// Build id: a hash of the sources, so two binaries built from the same
// tree on different machines (or for different targets) report the same id
// and any source change gives a new one. RSH_BUILD_SALT changes it on demand.

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn main() {
    let mut paths = vec![PathBuf::from("Cargo.toml"), PathBuf::from("Cargo.lock"), PathBuf::from("build.rs")];
    files(Path::new("src"), &mut paths);
    paths.sort();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for path in &paths {
        eat(path.to_string_lossy().as_bytes());
        eat(&std::fs::read(path).unwrap_or_default());
    }
    eat(std::env::var("RSH_BUILD_SALT").unwrap_or_default().as_bytes());
    println!("cargo:rustc-env=RSH_BUILD={:012x}", hash >> 16);
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-env-changed=RSH_BUILD_SALT");
}
